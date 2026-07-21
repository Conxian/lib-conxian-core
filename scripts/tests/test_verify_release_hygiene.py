from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import verify_release_hygiene as hygiene


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


if __name__ == "__main__":
    unittest.main()
