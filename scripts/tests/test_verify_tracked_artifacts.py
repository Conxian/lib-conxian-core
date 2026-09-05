from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch, MagicMock
from pathlib import Path
import sys

# Repo root directory
REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts import verify_tracked_artifacts as tracker


class VerifyTrackedArtifactsTests(unittest.TestCase):
    @patch("subprocess.run")
    def test_all_patterns_clean(self, mock_run: MagicMock) -> None:
        # Simulate git ls-files returning empty output for all patterns
        mock_run.return_value = MagicMock(stdout="", stderr="")

        result = tracker.verify_patterns()

        self.assertTrue(result)
        # Ensure it ran for all patterns
        self.assertEqual(mock_run.call_count, len(tracker.FORBIDDEN_PATTERNS))

    @patch("subprocess.run")
    def test_forbidden_pattern_matched(self, mock_run: MagicMock) -> None:
        # Simulate finding a tracked .env file on the matching pattern
        def side_effect(cmd: list[str], *args: any, **kwargs: any) -> MagicMock:
            pattern = cmd[-1]
            if ".env" in pattern:
                return MagicMock(stdout=".env.production\n", stderr="")
            return MagicMock(stdout="", stderr="")

        mock_run.side_effect = side_effect

        # We capture stdout to avoid cluttering the test runner
        with patch("sys.stdout") as mock_stdout:
            result = tracker.verify_patterns()

        self.assertFalse(result)

    @patch("subprocess.run")
    def test_expanded_forbidden_patterns_coverage(self, mock_run: MagicMock) -> None:
        # Verify that new secret/artifact patterns exist in FORBIDDEN_PATTERNS
        expected_patterns = [
            "credentials.json",
            "*.pfx",
            "*.p12",
            "*.jks",
            "*.keystore",
            "id_rsa*",
            "id_ed25519*",
            ".aws/",
            ".gcloud/",
            ".terraform/",
            ".vault/",
            "gateway/target/",
            "target-install/",
            ".pytest_cache/",
            "htmlcov/",
        ]
        for pattern in expected_patterns:
            self.assertIn(pattern, tracker.FORBIDDEN_PATTERNS)

    @patch("subprocess.run")
    def test_git_ls_files_error(self, mock_run: MagicMock) -> None:
        # Simulate subprocess.CalledProcessError for a pattern (should be handled gracefully)
        mock_run.side_effect = subprocess.CalledProcessError(1, ["git", "ls-files"])

        result = tracker.verify_patterns()

        # When git fails, the checker ignores it/doesn't flag it as a verified tracked file violation
        self.assertTrue(result)


if __name__ == "__main__":
    unittest.main()
