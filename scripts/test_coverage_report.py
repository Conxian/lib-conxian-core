#!/usr/bin/env python3
"""Focused stdlib tests for coverage report aggregation and policy guards."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from coverage_report import (
    CoverageError,
    HISTORICAL_PHASE1_REF,
    aggregate_file_summaries,
    aggregate_metrics,
    load_policy,
    repo_relative_path,
    summarize_report,
    summary_markdown,
    validate_mode,
)


def _summary(lines: tuple[int, int], regions: tuple[int, int], functions: tuple[int, int]) -> dict:
    def metric(counts: tuple[int, int]) -> dict[str, int]:
        return {"count": counts[0], "covered": counts[1]}

    return {
        "lines": metric(lines),
        "regions": metric(regions),
        "functions": metric(functions),
    }


class CoverageReportTests(unittest.TestCase):
    def test_aggregates_counts_instead_of_averaging_file_percentages(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = {
                "data": [
                    {
                        "files": [
                            {"filename": str(root / "src" / "a.rs"), "summary": _summary((1, 1), (2, 2), (1, 1))},
                            {"filename": str(root / "src" / "b.rs"), "summary": _summary((4, 3), (4, 2), (3, 2))},
                        ]
                    }
                ]
            }
            files = aggregate_file_summaries(report, root)
            metrics = aggregate_metrics(files)
            self.assertEqual(metrics["lines"], {"count": 5, "covered": 4, "percent": 80.0})
            self.assertEqual(metrics["regions"], {"count": 6, "covered": 4, "percent": 66.67})

    def test_normalizes_absolute_paths_and_merges_duplicate_entries(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            filename = root / "src" / "signing.rs"
            report = {
                "data": [
                    {"files": [{"filename": str(filename), "summary": _summary((2, 1), (2, 1), (2, 1))}]},
                    {"files": [{"filename": "src/signing.rs", "summary": _summary((3, 3), (3, 2), (3, 2))}]},
                ]
            }
            files = aggregate_file_summaries(report, root)
            self.assertEqual(sorted(files), ["src/signing.rs"])
            self.assertEqual(files["src/signing.rs"]["lines"], (5, 4))

    def test_rejects_traversal_and_symlink_escape_but_accepts_in_repo_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "repo"
            root.mkdir()
            (root / "src").mkdir()
            outside = Path(directory) / "outside"
            outside.mkdir()
            (outside / "outside.rs").write_text("fn outside() {}\n", encoding="utf-8")
            escape = root / "linked"
            try:
                escape.symlink_to(outside, target_is_directory=True)
            except (OSError, NotImplementedError) as error:
                self.skipTest(f"symlinks unavailable: {error}")

            in_repo_relative = "src/signing.rs"
            in_repo_absolute = str(root / in_repo_relative)
            self.assertEqual(repo_relative_path(in_repo_relative, root), in_repo_relative)
            self.assertEqual(repo_relative_path(in_repo_absolute, root), in_repo_relative)

            self.assertIsNone(repo_relative_path("../outside.rs", root))
            self.assertIsNone(repo_relative_path(str(root / ".." / "outside.rs"), root))
            self.assertIsNone(repo_relative_path("linked/outside.rs", root))

            report = {
                "data": [
                    {
                        "files": [
                            {"filename": in_repo_relative, "summary": _summary((2, 1), (2, 1), (1, 1))},
                            {"filename": "../outside.rs", "summary": _summary((9, 9), (9, 9), (9, 9))},
                            {
                                "filename": str(root / ".." / "outside.rs"),
                                "summary": _summary((9, 9), (9, 9), (9, 9)),
                            },
                            {
                                "filename": "linked/outside.rs",
                                "summary": _summary((9, 9), (9, 9), (9, 9)),
                            },
                        ]
                    }
                ]
            }
            self.assertEqual(sorted(aggregate_file_summaries(report, root)), [in_repo_relative])

    def test_markdown_method_matches_pinned_measurement_policy(self) -> None:
        summary = {
            "baseline_label": "current-implementation",
            "source_commit": "a" * 40,
            "tool": {"name": "cargo-llvm-cov", "version": "0.8.6", "rust_toolchain": "1.89.0"},
            "method": {
                "package": "lib-conxian-core",
                "locked": True,
                "all_targets": True,
                "default_features": False,
            },
            "targets": {},
        }
        markdown = summary_markdown(summary)
        self.assertIn(
            "cargo llvm-cov --package lib-conxian-core --locked --all-targets --no-default-features",
            markdown,
        )

    def test_policy_contains_required_targets_and_pins(self) -> None:
        policy_path = Path(__file__).resolve().parents[1] / "docs" / "coverage" / "policy.json"
        policy = load_policy(policy_path)
        self.assertEqual(policy["tool"]["version"], "0.8.6")
        self.assertEqual(policy["tool"]["rust_toolchain"], "1.89.0")
        targets = {target["name"]: target for target in policy["targets"]}
        self.assertEqual(targets["overall"]["eventual_line_percent"], 85.0)
        self.assertEqual(targets["bip110-policy"]["paths"], [
            "src/control_model/bip110.rs",
            "src/control_model/bip110_preflight.rs",
        ])

    def test_historical_label_requires_exact_commit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = {
                "data": [
                    {"files": [{"filename": str(root / "src" / "trust.rs"), "summary": _summary((1, 1), (1, 1), (1, 1))}]}
                ]
            }
            policy = {
                "measurement": {
                    "package": "lib-conxian-core",
                    "locked": True,
                    "all_targets": True,
                    "default_features": False,
                    "gate_metric": "lines",
                    "reported_metrics": ["lines", "regions", "functions"],
                    "branch_coverage": {"enabled": False, "reason": "test"},
                },
                "targets": [
                    {"name": "overall", "paths": [], "eventual_line_percent": 85.0},
                    {"name": "trust", "paths": ["src/trust.rs"], "eventual_line_percent": 95.0},
                ],
            }
            with self.assertRaises(CoverageError):
                summarize_report(
                    report,
                    root,
                    policy,
                    "not-the-historical-ref",
                    [],
                    "historical-phase1",
                    {},
                    [],
                    root / "coverage",
                )
            self.assertEqual(HISTORICAL_PHASE1_REF, "4065271bf6d9b035aa17f1c454f6a1db0c54754c")

    def test_report_only_is_advisory_but_future_modes_check_floors(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            baseline_path = root / "baseline.json"
            baseline_path.write_text(
                json.dumps(
                    {
                        "targets": {
                            "overall": {
                                "metrics": {"lines": {"percent": 80.0}}
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )
            policy = {
                "baseline": {"current_artifact": "baseline.json"},
                "targets": [{"name": "overall", "eventual_line_percent": 85.0}],
            }
            summary = {
                "targets": {
                    "overall": {
                        "status": "measured",
                        "metrics": {"lines": {"percent": 80.0}},
                        "eventual_line_percent": 85.0,
                    }
                }
            }

            validate_mode(summary, root, policy, "report-only")
            validate_mode(summary, root, policy, "no-regression")
            with self.assertRaises(CoverageError):
                validate_mode(summary, root, policy, "enforce")

            baseline_path.write_text(
                json.dumps(
                    {
                        "targets": {
                            "overall": {
                                "metrics": {"lines": {"percent": 81.0}}
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )
            with self.assertRaises(CoverageError):
                validate_mode(summary, root, policy, "no-regression")


if __name__ == "__main__":
    unittest.main()
