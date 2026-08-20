#!/usr/bin/env python3
"""Unit tests for the 5 implemented verification scripts.

Covers:
- verify_submodule_secret_filenames
- verify_compose_env_templates
- verify_bos_production_boundary
- verify_knowledge_retention
- verify_pr_bos_classification
"""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock

import sys
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

import verify_submodule_secret_filenames as submodule_checker
import verify_compose_env_templates as compose_checker
import verify_bos_production_boundary as bos_checker
import verify_knowledge_retention as retention_checker
import verify_pr_bos_classification as commit_checker


class TestVerifySubmoduleSecretFilenames(unittest.TestCase):
    def test_is_exempt(self):
        self.assertTrue(submodule_checker.is_exempt(".env.example"))
        self.assertTrue(submodule_checker.is_exempt("config.env.template"))
        self.assertFalse(submodule_checker.is_exempt(".env"))
        self.assertFalse(submodule_checker.is_exempt("private.key"))

    @patch("subprocess.run")
    def test_check_git_files_clean(self, mock_run):
        mock_run.return_value = MagicMock(
            stdout="src/lib.rs\nREADME.md\n.env.example\n", returncode=0
        )
        violations = submodule_checker.check_git_files()
        self.assertEqual(violations, [])

    @patch("subprocess.run")
    def test_check_git_files_violation(self, mock_run):
        mock_run.return_value = MagicMock(
            stdout="src/lib.rs\nsecret_key.pem\n.env\n", returncode=0
        )
        violations = submodule_checker.check_git_files()
        self.assertEqual(len(violations), 2)


class TestVerifyComposeEnvTemplates(unittest.TestCase):
    def test_is_placeholder(self):
        self.assertTrue(compose_checker.is_placeholder("CHANGE_ME"))
        self.assertTrue(compose_checker.is_placeholder("your_api_key_here"))
        self.assertTrue(compose_checker.is_placeholder("${SOME_ENV_VAR}"))
        self.assertFalse(compose_checker.is_placeholder("super_secret_live_token_1234567890"))

    def test_scan_file_clean(self):
        with tempfile.NamedTemporaryFile("w+", suffix=".env.example", delete=False) as f:
            f.write("# Sample env\nAPI_KEY=your_api_key_here\n")
            f.flush()
            temp_path = Path(f.name)

        try:
            with patch.object(compose_checker, "ROOT", temp_path.parent):
                violations = compose_checker.scan_file(temp_path)
                self.assertEqual(violations, [])
        finally:
            temp_path.unlink()


class TestVerifyBosProductionBoundary(unittest.TestCase):
    def test_is_test_file(self):
        self.assertTrue(bos_checker.is_test_file(Path("src/tests.rs")))
        self.assertTrue(bos_checker.is_test_file(Path("src/unit_test.rs")))
        self.assertFalse(bos_checker.is_test_file(Path("src/lib.rs")))

    def test_scan_src_file_clean(self):
        with tempfile.NamedTemporaryFile("w+", suffix=".rs", delete=False) as f:
            f.write("pub fn process_data() -> u32 { 42 }\n")
            f.flush()
            temp_path = Path(f.name)

        try:
            with patch.object(bos_checker, "ROOT", temp_path.parent):
                violations = bos_checker.scan_src_file(temp_path)
                self.assertEqual(violations, [])
        finally:
            temp_path.unlink()

    def test_scan_src_file_violation(self):
        with tempfile.NamedTemporaryFile("w+", suffix=".rs", delete=False) as f:
            f.write("use postgres::Client;\n")
            f.flush()
            temp_path = Path(f.name)

        try:
            with patch.object(bos_checker, "ROOT", temp_path.parent):
                violations = bos_checker.scan_src_file(temp_path)
                self.assertEqual(len(violations), 1)
        finally:
            temp_path.unlink()


class TestVerifyKnowledgeRetention(unittest.TestCase):
    def test_verify_doc_valid(self):
        with tempfile.NamedTemporaryFile("w+", suffix=".md", delete=False) as f:
            f.write("# Scorecard\n\n| Item | Status |\n| :--- | :--- |\n| Core | Active |\n" * 5)
            f.flush()
            temp_path = Path(f.name)

        try:
            with patch.object(retention_checker, "ROOT", temp_path.parent):
                violations = retention_checker.verify_doc(temp_path)
                self.assertEqual(violations, [])
        finally:
            temp_path.unlink()


class TestVerifyPrBosClassification(unittest.TestCase):
    def test_verify_commit_message(self):
        self.assertTrue(commit_checker.verify_commit_message("feat: add new feature"))
        self.assertTrue(commit_checker.verify_commit_message("fix(core): resolve race condition"))
        self.assertTrue(commit_checker.verify_commit_message("Merge pull request #123 from dev"))
        self.assertFalse(commit_checker.verify_commit_message("bad commit message without conventional prefix"))


if __name__ == "__main__":
    unittest.main()
