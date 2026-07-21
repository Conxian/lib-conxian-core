from __future__ import annotations

import json
import os
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch
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


class StatuslessResponse:
    def __init__(self, payload: object):
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
                "## Status",
                "**v1.2.3 Stable.**",
                "## Usage",
                "```toml",
                'demo-crate = "1.2.3"',
                'demo-crate = { version = "1.2.3", features = ["enclave"] }',
                "```",
                "## Historical examples",
                "> **v0.9.0 Breaking Change**: historical documentation is allowed.",
                'demo-crate = "0.9.0"',
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
    def _verify_workflow_variant(self, workflow: str) -> list[str]:
        with TemporaryDirectory() as directory:
            workflow_path = Path(directory) / "crates-publish.yml"
            workflow_path.write_text(workflow, encoding="utf-8")
            return guard.verify(workflow_path)

    def test_positive_local_parity_allows_historical_references(self) -> None:
        with fixture_root() as directory:
            self.assertEqual(guard.check_local_parity(Path(directory)), [])

    def test_readme_checks_current_usage_but_ignores_historical_examples(self) -> None:
        readme = "\n".join(
            [
                "[![Version](https://img.shields.io/badge/version-1.2.3-blue.svg)](CHANGELOG.md)",
                "## Status",
                "**v1.2.3 Stable.**",
                "## Usage",
                'lib-conxian-core = "1.2.3"',
                "## Historical examples",
                'lib-conxian-core = "old"',
                'lib-conxian-core = "0.2.11"',
            ]
        )
        self.assertEqual(
            guard._check_readme_versions(readme, "lib-conxian-core", "1.2.3"),
            [],
        )

        mismatched_current = readme.replace(
            'lib-conxian-core = "1.2.3"',
            'lib-conxian-core = "1.2.2"',
        )
        violations = guard._check_readme_versions(
            mismatched_current,
            "lib-conxian-core",
            "1.2.3",
        )
        self.assertTrue(any("dependency examples" in item for item in violations))

    def test_negative_local_parity_reports_lock_readme_and_changelog_drift(self) -> None:
        with fixture_root() as directory:
            root = Path(directory)
            (root / "Cargo.lock").write_text(
                'version = 4\n\n[[package]]\nname = "demo-crate"\nversion = "1.2.2"\n',
                encoding="utf-8",
            )
            (root / "README.md").write_text(
                "[![Version](https://img.shields.io/badge/version-1.2.2-blue.svg)](CHANGELOG.md)\n"
                "## Status\n"
                "**v1.2.2 Stable.**\n"
                "## Usage\n"
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

    def test_missing_http_status_fails_closed_for_crates_io_and_github(self) -> None:
        crates_response = StatuslessResponse(
            {"crate": {"name": "demo-crate"}, "version": {"num": "1.2.3"}}
        )
        crates_result = guard.fetch_crates_io_state(
            "demo-crate",
            "1.2.3",
            opener=lambda request, timeout: crates_response,
        )
        self.assertEqual(crates_result.state, guard.REMOTE_ERROR)
        self.assertIn("unverifiable HTTP status", crates_result.detail)
        self.assertTrue(crates_response.closed)

        github_response = StatuslessResponse({"tag_name": "v1.2.3"})
        github_result = guard.fetch_github_release_state(
            "Conxian/lib-conxian-core",
            "v1.2.3",
            "fixture-token",
            opener=lambda request, timeout: github_response,
        )
        self.assertEqual(github_result.state, guard.REMOTE_ERROR)
        self.assertIn("unverifiable HTTP status", github_result.detail)
        self.assertTrue(github_response.closed)

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

    def test_workflow_guards_reject_fail_open_condition_drift(self) -> None:
        workflow = guard.DEFAULT_WORKFLOW.read_text(encoding="utf-8")
        real_event_if = (
            "if: ${{ github.event_name == 'push' || "
            "(github.event_name == 'workflow_dispatch' && inputs.dry_run == false) }}"
        )
        publish_if = (
            "if: ${{ (github.event_name == 'push' || "
            "(github.event_name == 'workflow_dispatch' && inputs.dry_run == false)) && "
            "steps.crates_state.outputs.already_published != 'true' }}"
        )
        dry_run_if = "if: ${{ github.event_name == 'workflow_dispatch' && inputs.dry_run == true }}"

        variants = (
            (
                "crates.io preflight event gate",
                workflow.replace(real_event_if, "if: ${{ github.event_name == 'push' }}", 1),
                "Check crates.io version must run",
            ),
            (
                "real publish publication gate",
                workflow.replace(
                    publish_if,
                    "if: ${{ github.event_name == 'push' || "
                    "(github.event_name == 'workflow_dispatch' && inputs.dry_run == false) }}",
                    1,
                ),
                "Publish to crates.io must require",
            ),
            (
                "dry-run event gate",
                workflow.replace(dry_run_if, "if: ${{ github.event_name == 'push' }}", 1),
                "Publish to crates.io (dry run) must be limited",
            ),
            (
                "tag ref forwarding",
                workflow.replace('--tag "$GITHUB_REF_NAME"', '--tag "$GITHUB_REF"', 1),
                "GITHUB_REF_NAME",
            ),
        )
        for label, variant, expected in variants:
            with self.subTest(label=label):
                violations = self._verify_workflow_variant(variant)
                self.assertTrue(any(expected in item for item in violations), violations)

    def test_workflow_concurrency_is_ref_scoped_and_non_cancelling(self) -> None:
        workflow = guard.DEFAULT_WORKFLOW.read_text(encoding="utf-8")
        self.assertEqual(guard.verify(guard.DEFAULT_WORKFLOW), [])

        missing_concurrency = workflow.replace(
            "concurrency:\n"
            "  group: ${{ github.workflow }}-${{ github.ref }}\n"
            "  cancel-in-progress: false\n\n",
            "",
            1,
        )
        violations = self._verify_workflow_variant(missing_concurrency)
        self.assertTrue(any("concurrency protection" in item for item in violations), violations)

        cancelling_concurrency = workflow.replace(
            "cancel-in-progress: false",
            "cancel-in-progress: true",
            1,
        )
        violations = self._verify_workflow_variant(cancelling_concurrency)
        self.assertTrue(any("cancel-in-progress: false" in item for item in violations), violations)

        unscoped_concurrency = workflow.replace(
            "group: ${{ github.workflow }}-${{ github.ref }}",
            "group: release",
            1,
        )
        violations = self._verify_workflow_variant(unscoped_concurrency)
        self.assertTrue(any("scoped to github.workflow and github.ref" in item for item in violations), violations)

    def test_cli_tag_mismatch_returns_failure_without_network(self) -> None:
        with fixture_root() as directory:
            status = guard.main(["--root", directory, "--tag", "v1.2.4"])
            self.assertEqual(status, 1)

    def test_cli_crates_io_exit_semantics(self) -> None:
        with fixture_root() as directory:
            for state, expected_status in (
                (guard.REMOTE_PRESENT, 0),
                (guard.REMOTE_MISSING, guard.REMOTE_STATE_MISSING_EXIT),
                (guard.REMOTE_ERROR, 1),
            ):
                with self.subTest(state=state), patch.object(
                    guard,
                    "fetch_crates_io_state",
                    return_value=guard.RemoteCheck(state, "fixture crates.io result"),
                ):
                    status = guard.main(["--root", directory, "--crates-io-state"])
                    self.assertEqual(status, expected_status)

    def test_cli_wait_timeout_returns_failure(self) -> None:
        with fixture_root() as directory, patch.object(
            guard,
            "wait_for_crates_io",
            return_value=guard.RemoteCheck(guard.REMOTE_ERROR, "bounded polling timeout"),
        ):
            status = guard.main(
                [
                    "--root",
                    directory,
                    "--wait-for-crates-io",
                    "--poll-attempts",
                    "2",
                    "--poll-delay-seconds",
                    "0",
                ]
            )
            self.assertEqual(status, 1)

    def test_cli_github_release_exit_semantics(self) -> None:
        with fixture_root() as directory, patch.dict(
            os.environ,
            {
                "GITHUB_REPOSITORY": "Conxian/lib-conxian-core",
                "GITHUB_TOKEN": "fixture-token",
            },
            clear=False,
        ):
            for state, expected_status in (
                (guard.REMOTE_PRESENT, 0),
                (guard.REMOTE_MISSING, guard.REMOTE_STATE_MISSING_EXIT),
                (guard.REMOTE_ERROR, 1),
            ):
                with self.subTest(state=state), patch.object(
                    guard,
                    "fetch_github_release_state",
                    return_value=guard.RemoteCheck(state, "fixture GitHub result"),
                ):
                    status = guard.main(
                        [
                            "--root",
                            directory,
                            "--github-release-state",
                            "--tag",
                            "v1.2.3",
                        ]
                    )
                    self.assertEqual(status, expected_status)

    def test_current_workflow_contains_release_gates(self) -> None:
        self.assertEqual(guard.verify(guard.DEFAULT_WORKFLOW), [])


if __name__ == "__main__":
    unittest.main()
