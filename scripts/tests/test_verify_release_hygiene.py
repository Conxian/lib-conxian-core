from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from urllib.error import URLError


SCRIPT_ROOT = Path(__file__).resolve().parents[1]
if str(SCRIPT_ROOT) not in sys.path:
    sys.path.insert(0, str(SCRIPT_ROOT))

import verify_release_hygiene as guard  # noqa: E402


class FakeResponse:
    def __init__(self, status: int, payload: object):
        self.status = status
        self._body = json.dumps(payload).encode("utf-8")
        self.closed = False

    def read(self) -> bytes:
        return self._body

    def close(self) -> None:
        self.closed = True


def fixture_root() -> TemporaryDirectory[str]:
    directory = TemporaryDirectory()
    root = Path(directory.name)
    (root / "Cargo.toml").write_text(
        '[package]\nname = "demo-crate"\nversion = "1.2.3"\n',
        encoding="utf-8",
    )
    (root / "Cargo.lock").write_text(
        'version = 4\n\n[[package]]\nname = "demo-crate"\nversion = "1.2.3"\n',
        encoding="utf-8",
    )
    (root / "README.md").write_text(
        "\n".join(
            [
                "[![Version](https://img.shields.io/badge/version-1.2.3-blue.svg)](CHANGELOG.md)",
                "**v1.2.3 Stable.**",
                'demo-crate = "1.2.3"',
                'demo-crate = { version = "1.2.3", features = ["enclave"] }',
                "> **v0.9.0 Breaking Change**: historical documentation is allowed.",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    (root / "CHANGELOG.md").write_text(
        "\n".join(
            [
                "# Changelog",
                "## [Unreleased]",
                "- pending work",
                "## [v1.2.3] - 2026-07-21",
                "- current release",
                "## [v1.2.2] - 2026-07-01",
                "- historical release",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    return directory


class ReleaseHygieneTests(unittest.TestCase):
    def test_positive_local_parity_allows_historical_references(self) -> None:
        with fixture_root() as directory:
            self.assertEqual(guard.check_local_parity(Path(directory)), [])

    def test_negative_local_parity_reports_lock_readme_and_changelog_drift(self) -> None:
        with fixture_root() as directory:
            root = Path(directory)
            (root / "Cargo.lock").write_text(
                'version = 4\n\n[[package]]\nname = "demo-crate"\nversion = "1.2.2"\n',
                encoding="utf-8",
            )
            (root / "README.md").write_text(
                "[![Version](https://img.shields.io/badge/version-1.2.2-blue.svg)](CHANGELOG.md)\n"
                "**v1.2.2 Stable.**\n"
                'demo-crate = "1.2.2"\n',
                encoding="utf-8",
            )
            (root / "CHANGELOG.md").write_text(
                "## [Unreleased]\n## [v1.2.2] - 2026-07-21\n",
                encoding="utf-8",
            )

            violations = guard.check_local_parity(root)
            self.assertTrue(any("Cargo.lock root package version" in item for item in violations))
            self.assertTrue(any("README.md" in item for item in violations))
            self.assertTrue(any("CHANGELOG.md" in item for item in violations))

    def test_mismatched_tag_is_rejected(self) -> None:
        with fixture_root() as directory:
            violations = guard.check_tag("v1.2.4", Path(directory))
            self.assertEqual(len(violations), 1)
            self.assertIn("v1.2.3", violations[0])

    def test_matching_crates_io_state_is_safe_to_reuse(self) -> None:
        response = FakeResponse(
            200,
            {"crate": {"name": "demo-crate"}, "version": {"num": "1.2.3"}},
        )
        result = guard.fetch_crates_io_state(
            "demo-crate", "1.2.3", opener=lambda request, timeout: response
        )
        self.assertEqual(result.state, guard.REMOTE_PRESENT)
        self.assertEqual(guard.publication_decision(result), "skip-republish")
        self.assertTrue(response.closed)

    def test_missing_and_failed_remote_state_are_distinct(self) -> None:
        missing = guard.fetch_crates_io_state(
            "demo-crate",
            "1.2.3",
            opener=lambda request, timeout: FakeResponse(404, {"errors": []}),
        )
        self.assertEqual(missing.state, guard.REMOTE_MISSING)
        self.assertEqual(guard.publication_decision(missing), "publish")

        failed = guard.fetch_crates_io_state(
            "demo-crate",
            "1.2.3",
            opener=lambda request, timeout: FakeResponse(503, {"message": "unavailable"}),
        )
        self.assertEqual(failed.state, guard.REMOTE_ERROR)
        self.assertEqual(guard.publication_decision(failed), "fail-closed")

        network_failed = guard.fetch_crates_io_state(
            "demo-crate",
            "1.2.3",
            opener=lambda request, timeout: (_ for _ in ()).throw(URLError("offline")),
        )
        self.assertEqual(network_failed.state, guard.REMOTE_ERROR)
        self.assertEqual(guard.publication_decision(network_failed), "fail-closed")

    def test_crates_io_identity_mismatch_is_not_treated_as_published(self) -> None:
        result = guard.fetch_crates_io_state(
            "demo-crate",
            "1.2.3",
            opener=lambda request, timeout: FakeResponse(
                200,
                {"crate": {"name": "other-crate"}, "version": {"num": "1.2.3"}},
            ),
        )
        self.assertEqual(result.state, guard.REMOTE_MISMATCH)
        self.assertEqual(guard.publication_decision(result), "fail-closed")

    def test_bounded_polling_handles_delayed_propagation(self) -> None:
        responses = iter(
            [
                FakeResponse(404, {}),
                FakeResponse(404, {}),
                FakeResponse(
                    200,
                    {"crate": {"name": "demo-crate"}, "version": {"num": "1.2.3"}},
                ),
            ]
        )
        sleeps: list[float] = []
        result = guard.wait_for_crates_io(
            "demo-crate",
            "1.2.3",
            attempts=3,
            delay_seconds=0.25,
            opener=lambda request, timeout: next(responses),
            sleep=sleeps.append,
        )
        self.assertEqual(result.state, guard.REMOTE_PRESENT)
        self.assertEqual(sleeps, [0.25, 0.25])

    def test_github_release_state_and_creation_gate_are_fail_closed(self) -> None:
        missing = guard.fetch_github_release_state(
            "Conxian/lib-conxian-core",
            "v1.2.3",
            "fixture-token",
            opener=lambda request, timeout: FakeResponse(404, {}),
        )
        self.assertEqual(missing.state, guard.REMOTE_MISSING)
        self.assertTrue(
            guard.release_creation_allowed(
                local_parity_ok=True,
                publication_confirmed=True,
                github_release_state=missing,
            )
        )

        existing = guard.fetch_github_release_state(
            "Conxian/lib-conxian-core",
            "v1.2.3",
            "fixture-token",
            opener=lambda request, timeout: FakeResponse(200, {"tag_name": "v1.2.3"}),
        )
        self.assertEqual(existing.state, guard.REMOTE_PRESENT)
        self.assertFalse(
            guard.release_creation_allowed(
                local_parity_ok=True,
                publication_confirmed=True,
                github_release_state=existing,
            )
        )
        mismatched = guard.fetch_github_release_state(
            "Conxian/lib-conxian-core",
            "v1.2.3",
            "fixture-token",
            opener=lambda request, timeout: FakeResponse(200, {"tag_name": "v1.2.2"}),
        )
        self.assertEqual(mismatched.state, guard.REMOTE_MISMATCH)
        self.assertEqual(
            guard.fetch_github_release_state("Conxian/lib-conxian-core", "v1.2.3", None).state,
            guard.REMOTE_ERROR,
        )
        self.assertFalse(
            guard.release_creation_allowed(
                local_parity_ok=False,
                publication_confirmed=True,
                github_release_state=missing,
            )
        )

    def test_current_workflow_contains_release_gates(self) -> None:
        self.assertEqual(guard.verify(guard.DEFAULT_WORKFLOW), [])


if __name__ == "__main__":
    unittest.main()
