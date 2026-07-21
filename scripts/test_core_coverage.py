#!/usr/bin/env python3
"""Unit tests for scripts/core_coverage.py using only the Python standard library."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from core_coverage import evaluate, parse_report, render_markdown


FIXTURE = Path(__file__).parent / "testdata" / "core_coverage_sample.json"
POLICY = {
    "schema_version": 1,
    "tool": {"name": "cargo-llvm-cov", "version": "0.8.7"},
    "denominator": {
        "include": ["src/**/*.rs"],
        "exclude": ["src/tests.rs", "tests/**", "fuzz/**", "examples/**"],
    },
    "targets": {
        "overall": {"scope": "denominator", "metric": "lines", "minimum_percent": 80},
        "signing": {"files": ["src/signing.rs"], "metric": "lines", "minimum_percent": 95},
        "trust_policy": {
            "files": ["src/control_model/trust.rs"],
            "metric": "functions",
            "minimum_percent": 100,
        },
        "branch_probe": {
            "files": ["src/signing.rs"],
            "metric": "branches",
            "minimum_percent": 100,
        },
    },
}


class CoreCoverageTests(unittest.TestCase):
    def setUp(self) -> None:
        self.report = json.loads(FIXTURE.read_text(encoding="utf-8"))
        self.repo_root = Path("/repo")

    def test_parser_normalizes_paths_and_policy_excludes_non_production(self) -> None:
        files, metadata = parse_report(self.report, self.repo_root)
        self.assertEqual(
            [file.path for file in files],
            ["fuzz/ignored.rs", "src/control_model/trust.rs", "src/signing.rs", "tests/ignored.rs"],
        )
        self.assertEqual(metadata["tool"]["version"], "0.8.7")
        result = evaluate(self.report, POLICY, self.repo_root)
        self.assertEqual(
            result["denominator"]["files"],
            ["src/control_model/trust.rs", "src/signing.rs"],
        )

    def test_report_only_surfaces_but_does_not_fail_thresholds(self) -> None:
        result = evaluate(
            self.report,
            POLICY,
            self.repo_root,
            mode="report-only",
            commit_sha="abc123",
        )
        self.assertEqual(result["enforce_exit_code"], 0)
        self.assertEqual(result["status"], "thresholds_below_target")
        self.assertIn("signing", {issue["target"] for issue in result["issues"]})

    def test_enforce_mode_is_actionable(self) -> None:
        result = evaluate(self.report, POLICY, self.repo_root, mode="enforce")
        self.assertEqual(result["enforce_exit_code"], 1)
        below = next(issue for issue in result["issues"] if issue["target"] == "signing")
        self.assertEqual(below["kind"], "below_threshold")
        self.assertIn("add focused tests", below["action"])

    def test_zero_branch_dimension_is_reported_as_unsupported(self) -> None:
        result = evaluate(self.report, POLICY, self.repo_root, mode="enforce")
        branch = result["targets"]["branch_probe"]
        self.assertEqual(branch["status"], "unsupported")
        self.assertFalse(branch["metrics"]["branches"]["supported"])
        self.assertNotIn(
            "branch_probe",
            {issue["target"] for issue in result["issues"] if issue["kind"] == "below_threshold"},
        )

    def test_missing_scope_is_reported(self) -> None:
        policy = json.loads(json.dumps(POLICY))
        policy["targets"]["missing"] = {
            "files": ["src/verifier.rs"],
            "metric": "lines",
            "minimum_percent": 95,
        }
        result = evaluate(self.report, policy, self.repo_root)
        issue = next(issue for issue in result["issues"] if issue["target"] == "missing")
        self.assertEqual(issue["kind"], "missing_scope")
        self.assertEqual(result["targets"]["missing"]["status"], "missing")

    def test_markdown_is_stable_and_contains_targets(self) -> None:
        result = evaluate(self.report, POLICY, self.repo_root, commit_sha="abc123")
        markdown = render_markdown(result)
        self.assertIn("# Core coverage report", markdown)
        self.assertIn("| signing | lines |", markdown)
        self.assertTrue(markdown.endswith("\n"))

    def test_unreported_source_path_remains_in_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src" / "unmeasured.rs").parent.mkdir(parents=True)
            (root / "src" / "unmeasured.rs").write_text("pub struct Marker;\n", encoding="utf-8")
            result = evaluate(self.report, POLICY, root)
        self.assertIn("src/unmeasured.rs", result["denominator"]["files"])
        self.assertEqual(result["denominator"]["file_count"], 3)


if __name__ == "__main__":
    unittest.main()
