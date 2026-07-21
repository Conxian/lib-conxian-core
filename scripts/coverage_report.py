#!/usr/bin/env python3
"""Run and summarize the repository's pinned, report-first Rust coverage method."""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any


EXPECTED_TOOL_NAME = "cargo-llvm-cov"
EXPECTED_TOOL_VERSION = "0.8.6"
EXPECTED_RUST_TOOLCHAIN = "1.89.0"
HISTORICAL_PHASE1_REF = "4065271bf6d9b035aa17f1c454f6a1db0c54754c"
DEFAULT_POLICY = Path(__file__).resolve().parents[1] / "docs" / "coverage" / "policy.json"
DEFAULT_OUTPUT_DIR = Path(__file__).resolve().parents[1] / "coverage" / "llvm-cov"
TARGET_DISPLAY_ORDER = (
    "overall",
    "universal-signing",
    "protocol-verification",
    "trust-policy",
    "bip110-policy",
)


class CoverageError(RuntimeError):
    """A reproducibility, parsing, or threshold validation failure."""


def _run_capture(command: list[str], cwd: Path) -> str:
    result = subprocess.run(
        command,
        cwd=cwd,
        check=True,
        capture_output=True,
        text=True,
    )
    return (result.stdout + result.stderr).rstrip()


def _run(command: list[str], cwd: Path) -> None:
    print(f"$ {shlex.join(command)}")
    subprocess.run(command, cwd=cwd, check=True)


def _read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise CoverageError(f"unable to read JSON {path}: {error}") from error
    if not isinstance(value, dict):
        raise CoverageError(f"JSON root must be an object: {path}")
    return value


def load_policy(path: Path) -> dict[str, Any]:
    """Load and validate the small, dependency-free coverage policy."""

    policy = _read_json(path)
    if policy.get("schema_version") != 1:
        raise CoverageError("coverage policy schema_version must be 1")

    tool = policy.get("tool")
    if not isinstance(tool, dict):
        raise CoverageError("coverage policy must define a tool object")
    if tool.get("name") != EXPECTED_TOOL_NAME:
        raise CoverageError(f"coverage tool must be {EXPECTED_TOOL_NAME}")
    if tool.get("version") != EXPECTED_TOOL_VERSION:
        raise CoverageError(f"coverage tool must be pinned to {EXPECTED_TOOL_VERSION}")
    if tool.get("rust_toolchain") != EXPECTED_RUST_TOOLCHAIN:
        raise CoverageError(f"coverage Rust toolchain must be pinned to {EXPECTED_RUST_TOOLCHAIN}")

    measurement = policy.get("measurement")
    if not isinstance(measurement, dict):
        raise CoverageError("coverage policy must define measurement settings")
    if measurement.get("package") != "lib-conxian-core":
        raise CoverageError("coverage must measure package lib-conxian-core")
    if measurement.get("locked") is not True or measurement.get("all_targets") is not True:
        raise CoverageError("coverage must use locked dependencies and all targets")
    if measurement.get("gate_metric") != "lines":
        raise CoverageError("line coverage must remain the canonical gate metric")
    if measurement.get("branch_coverage", {}).get("enabled") is not False:
        raise CoverageError("branch coverage must remain disabled for the initial gate")
    if measurement.get("historical_phase1_ref") != HISTORICAL_PHASE1_REF:
        raise CoverageError("coverage policy historical ref does not match the required Phase 1 ref")

    targets = policy.get("targets")
    if not isinstance(targets, list) or not targets:
        raise CoverageError("coverage policy must define at least one target")
    names: set[str] = set()
    for target in targets:
        if not isinstance(target, dict):
            raise CoverageError("each coverage target must be an object")
        name = target.get("name")
        paths = target.get("paths")
        eventual = target.get("eventual_line_percent")
        if not isinstance(name, str) or not name or name in names:
            raise CoverageError(f"invalid or duplicate coverage target name: {name!r}")
        if not isinstance(paths, list) or not all(isinstance(path, str) for path in paths):
            raise CoverageError(f"coverage target {name!r} must define string paths")
        if not isinstance(eventual, (int, float)) or not 0 <= eventual <= 100:
            raise CoverageError(f"coverage target {name!r} has an invalid eventual line target")
        names.add(name)

    if "overall" not in names:
        raise CoverageError("coverage policy must define an overall target")
    return policy


def verify_tool_versions(repo_root: Path, policy: dict[str, Any]) -> dict[str, str]:
    """Verify the executable and independent coverage toolchain pins."""

    tool_output = _run_capture(["cargo", "llvm-cov", "--version"], repo_root)
    tool_match = re.search(r"\bcargo-llvm-cov\s+([0-9]+\.[0-9]+\.[0-9]+)\b", tool_output)
    expected_tool = policy["tool"]["version"]
    if not tool_match or tool_match.group(1) != expected_tool:
        raise CoverageError(
            f"expected cargo-llvm-cov {expected_tool}, observed: {tool_output or '<empty>'}"
        )

    rust_output = _run_capture(["rustc", "--version"], repo_root)
    rust_match = re.search(r"\brustc\s+([0-9]+\.[0-9]+\.[0-9]+)\b", rust_output)
    expected_rust = policy["tool"]["rust_toolchain"]
    if not rust_match or rust_match.group(1) != expected_rust:
        raise CoverageError(f"expected rustc {expected_rust}, observed: {rust_output or '<empty>'}")

    cargo_output = _run_capture(["cargo", "--version"], repo_root)
    return {
        "cargo_llvm_cov": tool_output.splitlines()[0] if tool_output else "",
        "rustc": rust_output.splitlines()[0] if rust_output else "",
        "cargo": cargo_output.splitlines()[0] if cargo_output else "",
    }


def git_metadata(repo_root: Path) -> tuple[str, list[str]]:
    commit = _run_capture(["git", "rev-parse", "HEAD"], repo_root)
    status = _run_capture(["git", "status", "--porcelain", "--untracked-files=all"], repo_root)
    dirty_paths: list[str] = []
    for line in status.splitlines():
        if len(line) < 4:
            continue
        path = line[3:]
        if " -> " in path:
            path = path.split(" -> ", 1)[1]
        dirty_paths.append(path)
    return commit, dirty_paths


def repo_relative_path(filename: str, repo_root: Path) -> str | None:
    """Normalize llvm-cov's absolute or relative filename to a repo path."""

    normalized = filename.replace("\\", "/")
    root = repo_root.resolve()
    path = Path(normalized)
    candidate = path if path.is_absolute() else root / path
    try:
        resolved = candidate.resolve()
        relative = resolved.relative_to(root)
    except (OSError, RuntimeError, ValueError):
        # A missing file is fine, but an unresolved path or a symlink that
        # escapes the repository is never safe to attribute to repository code.
        return None

    result = relative.as_posix()
    if not result or result == "." or result == ".." or result.startswith("../"):
        return None
    return result


def _metric_counts(summary: dict[str, Any], metric: str) -> tuple[int, int]:
    value = summary.get(metric)
    if not isinstance(value, dict):
        raise CoverageError(f"coverage file summary is missing {metric!r}")
    try:
        count = int(value["count"])
        covered = int(value["covered"])
    except (KeyError, TypeError, ValueError) as error:
        raise CoverageError(f"coverage file summary has invalid {metric!r}: {value!r}") from error
    if count < 0 or covered < 0 or covered > count:
        raise CoverageError(f"coverage file summary has invalid {metric!r}: {value!r}")
    return count, covered


def _merge_counts(left: tuple[int, int], right: tuple[int, int]) -> tuple[int, int]:
    return left[0] + right[0], left[1] + right[1]


def metric_dict(counts: tuple[int, int]) -> dict[str, int | float]:
    count, covered = counts
    percent = round((covered / count) * 100, 2) if count else 100.0
    return {"count": count, "covered": covered, "percent": percent}


def line_percent(metrics: dict[str, Any]) -> float | None:
    """Return an exact line percentage when covered/count totals are present."""

    lines = metrics.get("lines")
    if not isinstance(lines, dict):
        return None
    count = lines.get("count")
    covered = lines.get("covered")
    if isinstance(count, (int, float)) and isinstance(covered, (int, float)) and count:
        return (covered / count) * 100
    percent = lines.get("percent")
    return percent if isinstance(percent, (int, float)) else None


def aggregate_file_summaries(
    report: dict[str, Any], repo_root: Path
) -> dict[str, dict[str, tuple[int, int]]]:
    """Return per-repository-file metric counts from llvm-cov JSON."""

    data = report.get("data")
    if not isinstance(data, list):
        raise CoverageError("llvm-cov JSON is missing its data array")

    files: dict[str, dict[str, tuple[int, int]]] = {}
    for entry in data:
        if not isinstance(entry, dict):
            raise CoverageError("llvm-cov JSON data entries must be objects")
        raw_files = entry.get("files")
        if not isinstance(raw_files, list):
            raise CoverageError("llvm-cov JSON data entry is missing files")
        for raw_file in raw_files:
            if not isinstance(raw_file, dict) or not isinstance(raw_file.get("filename"), str):
                raise CoverageError("llvm-cov JSON contains an invalid file entry")
            relative = repo_relative_path(raw_file["filename"], repo_root)
            if relative is None:
                continue
            summary = raw_file.get("summary")
            if not isinstance(summary, dict):
                raise CoverageError(f"llvm-cov file {relative} is missing a summary")
            metrics = files.setdefault(relative, {})
            for metric in ("lines", "regions", "functions"):
                counts = _metric_counts(summary, metric)
                metrics[metric] = _merge_counts(metrics.get(metric, (0, 0)), counts)

    if not files:
        raise CoverageError("llvm-cov JSON did not contain any repository source files")
    return files


def aggregate_metrics(
    files: dict[str, dict[str, tuple[int, int]]], paths: list[str] | None = None
) -> dict[str, dict[str, int | float]]:
    selected = files if paths is None else {path: files[path] for path in paths if path in files}
    if not selected:
        return {}
    totals: dict[str, tuple[int, int]] = {metric: (0, 0) for metric in ("lines", "regions", "functions")}
    for metrics in selected.values():
        for metric in totals:
            totals[metric] = _merge_counts(totals[metric], metrics[metric])
    return {metric: metric_dict(counts) for metric, counts in totals.items()}


def _file_metrics(metrics: dict[str, tuple[int, int]]) -> dict[str, dict[str, int | float]]:
    return {metric: metric_dict(metrics[metric]) for metric in ("lines", "regions", "functions")}


def _display_method_command(summary: dict[str, Any]) -> str:
    method = summary["method"]
    command = ["cargo", "llvm-cov"]
    if method.get("package"):
        command.extend(["--package", method["package"]])
    if method.get("locked"):
        command.append("--locked")
    if method.get("all_targets"):
        command.append("--all-targets")
    if method.get("default_features", True) is False:
        command.append("--no-default-features")
    return shlex.join(command)


def summarize_report(
    report: dict[str, Any], repo_root: Path, policy: dict[str, Any], commit: str, dirty_paths: list[str],
    baseline_label: str, tool_versions: dict[str, str], commands: list[list[str]], output_dir: Path
) -> dict[str, Any]:
    files = aggregate_file_summaries(report, repo_root)
    targets: dict[str, dict[str, Any]] = {}
    errors: list[str] = []
    for target in policy["targets"]:
        name = target["name"]
        paths = list(target["paths"])
        if name == "overall":
            targets[name] = {
                "status": "measured",
                "paths": sorted(files),
                "metrics": aggregate_metrics(files),
                "eventual_line_percent": target["eventual_line_percent"],
            }
            continue

        present_paths = [path for path in paths if path in files]
        absent_paths = [path for path in paths if not (repo_root / path).is_file()]
        missing_paths = [path for path in paths if path not in files and path not in absent_paths]
        if missing_paths:
            errors.append(f"coverage JSON omitted configured source path(s) for {name}: {missing_paths}")
        status = "measured" if present_paths else "not_applicable"
        targets[name] = {
            "status": status,
            "paths": paths,
            "present_paths": present_paths,
            "absent_paths": absent_paths,
            "metrics": aggregate_metrics(files, present_paths),
            "eventual_line_percent": target["eventual_line_percent"],
        }

    if errors:
        raise CoverageError("; ".join(errors))

    measurement = policy["measurement"]
    if baseline_label == "historical-phase1" and commit != HISTORICAL_PHASE1_REF:
        raise CoverageError(
            "historical Phase 1 reports require checked-out commit "
            f"{HISTORICAL_PHASE1_REF}, observed {commit}"
        )

    output_marker = "<output-dir>"
    output_prefix = str(output_dir) + os.sep
    recorded_commands: list[str] = []
    for command in commands:
        recorded_arguments = []
        for argument in command:
            if argument == str(output_dir):
                recorded_arguments.append(output_marker)
            elif argument.startswith(output_prefix):
                recorded_arguments.append(output_marker + os.sep + argument[len(output_prefix) :])
            else:
                recorded_arguments.append(argument)
        recorded_commands.append(shlex.join(recorded_arguments))

    return {
        "schema_version": 1,
        "baseline_label": baseline_label,
        "source_commit": commit,
        "working_tree_dirty": bool(dirty_paths),
        "dirty_paths": sorted(dirty_paths),
        "tool": {
            "name": EXPECTED_TOOL_NAME,
            "version": policy["tool"]["version"],
            "rust_toolchain": policy["tool"]["rust_toolchain"],
            **tool_versions,
        },
        "method": {
            "package": measurement["package"],
            "locked": measurement["locked"],
            "all_targets": measurement["all_targets"],
            "default_features": measurement.get("default_features", True),
            "gate_metric": measurement["gate_metric"],
            "reported_metrics": measurement["reported_metrics"],
            "branch_coverage_enabled": measurement["branch_coverage"]["enabled"],
            "branch_coverage_note": measurement["branch_coverage"]["reason"],
            "historical_phase1_ref": HISTORICAL_PHASE1_REF,
            "commands": recorded_commands,
        },
        "report": {
            "raw_json": "coverage.json",
            "lcov": "lcov.info",
            "html_directory": "html",
            "text": "coverage.txt",
            "summary_json": "summary.json",
            "summary_markdown": "summary.md",
            "llvm_cov_json_version": report.get("version"),
        },
        "metrics": aggregate_metrics(files),
        "targets": targets,
        "files": {path: _file_metrics(files[path]) for path in sorted(files)},
        "eventual_targets_are_advisory": True,
        "output_directory": output_dir.name,
    }


def summary_markdown(summary: dict[str, Any]) -> str:
    """Render a concise, human-readable report from the machine summary."""

    lines = [
        "# Rust coverage report",
        "",
        f"- Baseline label: `{summary['baseline_label']}`",
        f"- Source commit: `{summary['source_commit']}`",
        f"- Tool: `{summary['tool']['name']} {summary['tool']['version']}`",
        f"- Rust toolchain: `{summary['tool']['rust_toolchain']}`",
        f"- Method: `{_display_method_command(summary)}`",
        "- Canonical metric: **line coverage**; regions and functions are reported for diagnosis.",
        "- Branch coverage: disabled/not gated because the LLVM branch mode is currently unstable.",
        "",
        "| Scope | Lines | Regions | Functions | Status | Eventual line target |",
        "| --- | ---: | ---: | ---: | --- | ---: |",
    ]
    target_names = [name for name in TARGET_DISPLAY_ORDER if name in summary["targets"]]
    target_names.extend(name for name in summary["targets"] if name not in target_names)
    for name in target_names:
        target = summary["targets"][name]
        metrics = target.get("metrics", {})
        line = metrics.get("lines", {}).get("percent", "N/A")
        region = metrics.get("regions", {}).get("percent", "N/A")
        function = metrics.get("functions", {}).get("percent", "N/A")
        eventual = f"{target['eventual_line_percent']:.2f}%"
        lines.append(f"| `{name}` | {line if line == 'N/A' else f'{line:.2f}%'} | "
                     f"{region if region == 'N/A' else f'{region:.2f}%'} | "
                     f"{function if function == 'N/A' else f'{function:.2f}%'} | "
                     f"{target['status']} | {eventual} |")
    lines.extend(
        [
            "",
            "Target shortfalls are advisory in this report-first phase; measurement and parser failures are not.",
            "",
        ]
    )
    return "\n".join(lines)


def _write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def _load_current_baseline(repo_root: Path, policy: dict[str, Any]) -> dict[str, Any]:
    relative = policy["baseline"]["current_artifact"]
    path = repo_root / relative
    if not path.is_file():
        raise CoverageError(f"current baseline artifact is required for enforcement: {path}")
    return _read_json(path)


def validate_mode(summary: dict[str, Any], repo_root: Path, policy: dict[str, Any], mode: str) -> None:
    if mode == "report-only":
        return

    baseline = _load_current_baseline(repo_root, policy)
    errors: list[str] = []
    for name, target in summary["targets"].items():
        current_metrics = target.get("metrics", {})
        baseline_target = baseline.get("targets", {}).get(name, {})
        baseline_metrics = baseline_target.get("metrics", {})
        current_line = line_percent(current_metrics)
        baseline_line = line_percent(baseline_metrics)
        if target.get("status") != "measured" or not isinstance(current_line, (int, float)):
            errors.append(f"{name} is not measurable in {mode} mode")
        elif not isinstance(baseline_line, (int, float)):
            errors.append(f"{name} has no measured current baseline floor")
        elif current_line + 1e-9 < baseline_line:
            errors.append(f"{name} regressed from {baseline_line:.2f}% to {current_line:.2f}% line coverage")

        if mode == "enforce" and target.get("status") == "measured":
            eventual = target["eventual_line_percent"]
            if not isinstance(current_line, (int, float)) or current_line + 1e-9 < eventual:
                observed = "N/A" if not isinstance(current_line, (int, float)) else f"{current_line:.2f}%"
                errors.append(f"{name} is below eventual {eventual:.2f}% line target (observed {observed})")

    if errors:
        raise CoverageError("; ".join(errors))


def build_commands(policy: dict[str, Any], output_dir: Path) -> tuple[list[str], list[list[str]]]:
    package = policy["measurement"]["package"]
    test_command = [
        "cargo",
        "llvm-cov",
        "--no-report",
        "--package",
        package,
        "--locked",
        "--all-targets",
    ]
    if policy["measurement"].get("default_features") is False:
        test_command.append("--no-default-features")

    report_base = ["cargo", "llvm-cov", "report", "--package", package, "--locked"]
    commands = [
        test_command,
        [*report_base, "--json", "--output-path", str(output_dir / "coverage.json")],
        [*report_base, "--lcov", "--output-path", str(output_dir / "lcov.info")],
        [*report_base, "--html", "--output-dir", str(output_dir)],
        [*report_base, "--text", "--output-path", str(output_dir / "coverage.txt")],
    ]
    return test_command, commands


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=DEFAULT_POLICY.parents[2])
    parser.add_argument("--policy", type=Path, default=DEFAULT_POLICY)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument(
        "--mode",
        choices=("report-only", "no-regression", "enforce"),
        default="report-only",
        help="report-only is the current CI mode; enforcement modes are future rollout stages",
    )
    parser.add_argument(
        "--baseline-label",
        choices=("ci", "current-implementation", "historical-phase1"),
        default="ci",
    )
    parser.add_argument("--baseline-output", type=Path)
    parser.add_argument("--baseline-markdown-output", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    repo_root = args.repo_root.resolve()
    policy_path = args.policy.resolve()
    output_dir = args.output_dir.resolve()
    try:
        if not (repo_root / "Cargo.toml").is_file():
            raise CoverageError(f"repo root does not contain Cargo.toml: {repo_root}")
        policy = load_policy(policy_path)
        tool_versions = verify_tool_versions(repo_root, policy)
        commit, dirty_paths = git_metadata(repo_root)
        if args.baseline_label == "historical-phase1" and commit != HISTORICAL_PHASE1_REF:
            raise CoverageError(
                "historical Phase 1 reports require checked-out commit "
                f"{HISTORICAL_PHASE1_REF}, observed {commit}"
            )

        if output_dir.exists():
            shutil.rmtree(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)
        test_command, commands = build_commands(policy, output_dir)
        _run(test_command, repo_root)
        for command in commands[1:]:
            _run(command, repo_root)

        report = _read_json(output_dir / "coverage.json")
        summary = summarize_report(
            report,
            repo_root,
            policy,
            commit,
            dirty_paths,
            args.baseline_label,
            tool_versions,
            commands,
            output_dir,
        )
        summary_json = json.dumps(summary, indent=2, sort_keys=True) + "\n"
        summary_md = summary_markdown(summary)
        _write_text(output_dir / "summary.json", summary_json)
        _write_text(output_dir / "summary.md", summary_md)
        if args.baseline_output:
            _write_text(args.baseline_output.resolve(), summary_json)
        if args.baseline_markdown_output:
            _write_text(args.baseline_markdown_output.resolve(), summary_md)
        validate_mode(summary, repo_root, policy, args.mode)
        print(summary_md)
        return 0
    except (CoverageError, OSError, subprocess.CalledProcessError) as error:
        print(f"coverage error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
