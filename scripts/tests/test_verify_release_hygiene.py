from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch
from urllib.error import URLError

SCRIPT_ROOT = Path(__file__).resolve().parents[1]
if str(SCRIPT_ROOT) not in sys.path:
    sys.path.insert(0, str(SCRIPT_ROOT))

from scripts import verify_release_hygiene as hygiene


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
    def verify_workflow(self, workflow_text: str) -> list[str]:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            workflow_path = root / "crates-publish.yml"
            main_workflow_path = root / "main.yml"
            workflow_path.write_text(workflow_text, encoding="utf-8")
            main_workflow_path.write_text(
                hygiene.DEFAULT_MAIN_WORKFLOW.read_text(encoding="utf-8"),
                encoding="utf-8",
            )
            return hygiene.verify(workflow_path, main_workflow_path)

    def test_repository_workflow_passes(self) -> None:
        violations = hygiene.verify(
            hygiene.DEFAULT_WORKFLOW,
            hygiene.DEFAULT_MAIN_WORKFLOW,
        )

        self.assertEqual(violations, [])

    def test_contents_write_permission_is_required(self) -> None:
        workflow = hygiene.DEFAULT_WORKFLOW.read_text(encoding="utf-8").replace(
            "  contents: write", "  contents: read", 1
        )

        violations = self.verify_workflow(workflow)

        self.assertTrue(
            any("contents: write permission" in violation for violation in violations)
        )

    def test_explicit_gh_token_wiring_is_required(self) -> None:
        workflow = hygiene.DEFAULT_WORKFLOW.read_text(encoding="utf-8").replace(
            "          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}\n", "", 1
        )

        violations = self.verify_workflow(workflow)

        self.assertTrue(any("GH_TOKEN" in violation for violation in violations))

    def test_release_race_recheck_and_post_release_gate_are_required(self) -> None:
        workflow = hygiene.DEFAULT_WORKFLOW.read_text(encoding="utf-8")
        workflow = workflow.replace("post_create_view_output", "post_create_output")
        workflow = workflow.replace("--phase post-release", "--phase post-release-removed", 1)

        violations = self.verify_workflow(workflow)

        self.assertTrue(
            any("re-check for an already-existing release" in violation for violation in violations)
        )
        self.assertTrue(
            any("post-release verification" in violation for violation in violations)
        )

    def test_publish_commands_require_locked_resolutions(self) -> None:
        workflow = hygiene.DEFAULT_WORKFLOW.read_text(encoding="utf-8").replace(
            "cargo publish --dry-run --locked", "cargo publish --dry-run"
        )

        violations = self.verify_workflow(workflow)

        self.assertTrue(
            any("cargo publish --dry-run --locked" in violation for violation in violations)
        )

        workflow = hygiene.DEFAULT_WORKFLOW.read_text(encoding="utf-8").replace(
            "cargo publish --locked", "cargo publish"
        )

        violations = self.verify_workflow(workflow)

        self.assertTrue(
            any("cargo publish --locked" in violation for violation in violations)
        )

    def test_publish_commands_require_explicit_workspace_packages(self) -> None:
        workflow = hygiene.DEFAULT_WORKFLOW.read_text(encoding="utf-8").replace(
            "cargo publish --locked -p lib-conxian-core;",
            "cargo publish --locked;",
            1,
        )
        violations = self.verify_workflow(workflow)
        self.assertTrue(any("lib-conxian-core" in violation for violation in violations))

        workflow = hygiene.DEFAULT_WORKFLOW.read_text(encoding="utf-8").replace(
            "cargo publish --locked -p lib-conxian-core-enclave;",
            "cargo publish --locked;",
            1,
        )
        violations = self.verify_workflow(workflow)
        self.assertTrue(
            any("lib-conxian-core-enclave" in violation for violation in violations)
        )


class ParityAndRemoteReleaseHygieneTests(unittest.TestCase):
    def _verify_workflow_variant(self, workflow: str) -> list[str]:
        with TemporaryDirectory() as directory:
            workflow_path = Path(directory) / "crates-publish.yml"
            workflow_path.write_text(workflow, encoding="utf-8")
            return hygiene.verify(workflow_path)

    def test_positive_local_parity_allows_historical_references(self) -> None:
        with fixture_root() as directory:
            self.assertEqual(hygiene.check_local_parity(Path(directory)), [])

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
            hygiene._check_readme_versions(readme, "lib-conxian-core", "1.2.3"),
            [],
        )

        mismatched_current = readme.replace(
            'lib-conxian-core = "1.2.3"',
            'lib-conxian-core = "1.2.2"',
        )
        violations = hygiene._check_readme_versions(
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

            violations = hygiene.check_local_parity(root)
            self.assertTrue(any("Cargo.lock root package version" in item for item in violations))
            self.assertTrue(any("README.md" in item for item in violations))
            self.assertTrue(any("CHANGELOG.md" in item for item in violations))

    def test_mismatched_tag_is_rejected(self) -> None:
        with fixture_root() as directory:
            violations = hygiene.check_tag("v1.2.4", Path(directory))
            self.assertEqual(len(violations), 1)
            self.assertIn("v1.2.3", violations[0])

    def test_matching_crates_io_state_is_safe_to_reuse(self) -> None:
        response = FakeResponse(
            200,
            {"crate": {"name": "demo-crate"}, "version": {"num": "1.2.3"}},
        )
        result = hygiene.fetch_crates_io_state(
            "demo-crate", "1.2.3", opener=lambda request, timeout: response
        )
        self.assertEqual(result.state, hygiene.REMOTE_PRESENT)
        self.assertEqual(hygiene.publication_decision(result), "skip-republish")
        self.assertTrue(response.closed)

    def test_missing_and_failed_remote_state_are_distinct(self) -> None:
        missing = hygiene.fetch_crates_io_state(
            "demo-crate",
            "1.2.3",
            opener=lambda request, timeout: FakeResponse(404, {"errors": []}),
        )
        self.assertEqual(missing.state, hygiene.REMOTE_MISSING)
        self.assertEqual(hygiene.publication_decision(missing), "publish")

        failed = hygiene.fetch_crates_io_state(
            "demo-crate",
            "1.2.3",
            opener=lambda request, timeout: FakeResponse(503, {"message": "unavailable"}),
        )
        self.assertEqual(failed.state, hygiene.REMOTE_ERROR)
        self.assertEqual(hygiene.publication_decision(failed), "fail-closed")

        network_failed = hygiene.fetch_crates_io_state(
            "demo-crate",
            "1.2.3",
            opener=lambda request, timeout: (_ for _ in ()).throw(URLError("offline")),
        )
        self.assertEqual(network_failed.state, hygiene.REMOTE_ERROR)
        self.assertEqual(hygiene.publication_decision(network_failed), "fail-closed")

    def test_missing_http_status_fails_closed_for_crates_io_and_github(self) -> None:
        crates_response = StatuslessResponse(
            {"crate": {"name": "demo-crate"}, "version": {"num": "1.2.3"}}
        )
        crates_result = hygiene.fetch_crates_io_state(
            "demo-crate",
            "1.2.3",
            opener=lambda request, timeout: crates_response,
        )
        self.assertEqual(crates_result.state, hygiene.REMOTE_ERROR)
        self.assertIn("unverifiable HTTP status", crates_result.detail)
        self.assertTrue(crates_response.closed)

        github_response = StatuslessResponse({"tag_name": "v1.2.3"})
        github_result = hygiene.fetch_github_release_state(
            "Conxian/lib-conxian-core",
            "v1.2.3",
            "fixture-token",
            opener=lambda request, timeout: github_response,
        )
        self.assertEqual(github_result.state, hygiene.REMOTE_ERROR)
        self.assertIn("unverifiable HTTP status", github_result.detail)
        self.assertTrue(github_response.closed)

    def test_crates_io_identity_mismatch_is_not_treated_as_published(self) -> None:
        result = hygiene.fetch_crates_io_state(
            "demo-crate",
            "1.2.3",
            opener=lambda request, timeout: FakeResponse(
                200,
                {"crate": {"name": "other-crate"}, "version": {"num": "1.2.3"}},
            ),
        )
        self.assertEqual(result.state, hygiene.REMOTE_MISMATCH)
        self.assertEqual(hygiene.publication_decision(result), "fail-closed")

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
        result = hygiene.wait_for_crates_io(
            "demo-crate",
            "1.2.3",
            attempts=3,
            delay_seconds=0.25,
            opener=lambda request, timeout: next(responses),
            sleep=sleeps.append,
        )
        self.assertEqual(result.state, hygiene.REMOTE_PRESENT)
        self.assertEqual(sleeps, [0.25, 0.25])

    def test_github_release_state_and_creation_gate_are_fail_closed(self) -> None:
        missing = hygiene.fetch_github_release_state(
            "Conxian/lib-conxian-core",
            "v1.2.3",
            "fixture-token",
            opener=lambda request, timeout: FakeResponse(404, {}),
        )
        self.assertEqual(missing.state, hygiene.REMOTE_MISSING)
        self.assertTrue(
            hygiene.release_creation_allowed(
                local_parity_ok=True,
                publication_confirmed=True,
                github_release_state=missing,
            )
        )

        existing = hygiene.fetch_github_release_state(
            "Conxian/lib-conxian-core",
            "v1.2.3",
            "fixture-token",
            opener=lambda request, timeout: FakeResponse(200, {"tag_name": "v1.2.3"}),
        )
        self.assertEqual(existing.state, hygiene.REMOTE_PRESENT)
        self.assertFalse(
            hygiene.release_creation_allowed(
                local_parity_ok=True,
                publication_confirmed=True,
                github_release_state=existing,
            )
        )
        mismatched = hygiene.fetch_github_release_state(
            "Conxian/lib-conxian-core",
            "v1.2.3",
            "fixture-token",
            opener=lambda request, timeout: FakeResponse(200, {"tag_name": "v1.2.2"}),
        )
        self.assertEqual(mismatched.state, hygiene.REMOTE_MISMATCH)
        self.assertEqual(
            hygiene.fetch_github_release_state("Conxian/lib-conxian-core", "v1.2.3", None).state,
            hygiene.REMOTE_ERROR,
        )
        self.assertFalse(
            hygiene.release_creation_allowed(
                local_parity_ok=False,
                publication_confirmed=True,
                github_release_state=missing,
            )
        )

    def test_workflow_guards_reject_fail_open_condition_drift(self) -> None:
        workflow = hygiene.DEFAULT_WORKFLOW.read_text(encoding="utf-8")
        pre_publish_if = (
            "if: ${{ github.event_name == 'push' || "
            "(github.event_name == 'workflow_dispatch' && inputs.mode == 'publish') }}"
        )
        publish_if = pre_publish_if
        dry_run_if = "if: ${{ github.event_name == 'workflow_dispatch' && inputs.mode == 'dry-run' }}"

        variants = (
            (
                "pre-publish manual event gate",
                workflow.replace(pre_publish_if, "if: ${{ github.event_name == 'push' }}", 1),
                "pre-publish guard must run for manual publish mode",
            ),
            (
                "real publish publication gate",
                workflow.replace(
                    publish_if + "\n        env:\n          CARGO_REGISTRY_TOKEN:",
                    "if: ${{ github.event_name == 'push' }}\n        env:\n          CARGO_REGISTRY_TOKEN:",
                    1,
                ),
                "manual real publication must require mode == publish",
            ),
            (
                "dry-run event gate",
                workflow.replace(
                    dry_run_if + "\n        run: cargo publish --dry-run --locked",
                    "if: ${{ github.event_name == 'push' }}\n        run: cargo publish --dry-run --locked",
                    1,
                ),
                "dry-run publication must require mode == dry-run",
            ),
            (
                "tag ref forwarding",
                workflow.replace("github.ref_name", "github.ref", 1),
                "job must derive RELEASE_TAG",
            ),
        )
        for label, variant, expected in variants:
            with self.subTest(label=label):
                violations = self._verify_workflow_variant(variant)
                self.assertTrue(any(expected in item for item in violations), violations)

    def test_workflow_concurrency_is_ref_scoped_and_non_cancelling(self) -> None:
        workflow = hygiene.DEFAULT_WORKFLOW.read_text(encoding="utf-8")
        self.assertEqual(hygiene.verify(hygiene.DEFAULT_WORKFLOW), [])

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
            status = hygiene.main(["--root", directory, "--tag", "v1.2.4"])
            self.assertEqual(status, 1)

    def test_cli_crates_io_exit_semantics(self) -> None:
        with fixture_root() as directory:
            for state, expected_status in (
                (hygiene.REMOTE_PRESENT, 0),
                (hygiene.REMOTE_MISSING, hygiene.REMOTE_STATE_MISSING_EXIT),
                (hygiene.REMOTE_ERROR, 1),
            ):
                with self.subTest(state=state), patch.object(
                    hygiene,
                    "fetch_crates_io_state",
                    return_value=hygiene.RemoteCheck(state, "fixture crates.io result"),
                ):
                    status = hygiene.main(["--root", directory, "--crates-io-state"])
                    self.assertEqual(status, expected_status)

    def test_cli_wait_timeout_returns_failure(self) -> None:
        with fixture_root() as directory, patch.object(
            hygiene,
            "wait_for_crates_io",
            return_value=hygiene.RemoteCheck(hygiene.REMOTE_ERROR, "bounded polling timeout"),
        ):
            status = hygiene.main(
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
                "GITHUB_TOKEN": "[REDACTED_GITHUB_TOKEN]",
            },
            clear=False,
        ):
            for state, expected_status in (
                (hygiene.REMOTE_PRESENT, 0),
                (hygiene.REMOTE_MISSING, hygiene.REMOTE_STATE_MISSING_EXIT),
                (hygiene.REMOTE_ERROR, 1),
            ):
                with self.subTest(state=state), patch.object(
                    hygiene,
                    "fetch_github_release_state",
                    return_value=hygiene.RemoteCheck(state, "fixture GitHub result"),
                ):
                    status = hygiene.main(
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
        self.assertEqual(hygiene.verify(hygiene.DEFAULT_WORKFLOW), [])


if __name__ == "__main__":
    unittest.main()
