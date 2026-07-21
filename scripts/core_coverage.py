#!/usr/bin/env python3
"""Parse cargo-llvm-cov JSON and evaluate the Core coverage policy.

The script deliberately consumes the merged LLVM JSON report rather than
reimplementing coverage collection.  It keeps path selection and threshold
policy in the repository, emits deterministic artifacts, and supports a
report-only rollout before enforcement is enabled.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Mapping


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_POLICY = ROOT / "config" / "core_coverage.json"
DIMENSIONS = ("lines", "regions", "functions", "branches", "mcdc")


class CoverageReportError(ValueError):
    """Raised when an LLVM JSON report cannot be interpreted safely."""


@dataclass(frozen=True)
class Metric:
    """A normalized LLVM coverage metric."""

    covered: int
    total: int
    percent: float | None
    supported: bool
    reason: str | None = None

    def as_dict(self) -> dict[str, Any]:
        return {
            "covered": self.covered,
            "total": self.total,
            "percent": self.percent,
            "supported": self.supported,
            **({"reason": self.reason} if self.reason else {}),
        }


@dataclass(frozen=True)
class CoverageFile:
    """A report file with a repository-relative path and metrics."""

    path: str
    metrics: Mapping[str, Metric]


def _require_mapping(value: Any, context: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise CoverageReportError(f"{context} must be a JSON object")
    return value


def _require_sequence(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise CoverageReportError(f"{context} must be a JSON array")
    return value


def _as_nonnegative_int(value: Any, context: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise CoverageReportError(f"{context} must be a non-negative integer")
    return value


def _percent(covered: int, total: int) -> float | None:
    if total == 0:
        return None
    return round(covered * 100.0 / total, 2)


def _parse_metric(value: Any, context: str) -> Metric:
    """Parse one LLVM metric without trusting undocumented fields."""

    if value is None:
        return Metric(0, 0, None, False, "metric is absent from the LLVM report")

    raw = _require_mapping(value, context)
    total_value = raw.get("count")
    covered_value = raw.get("covered")
    if total_value is None or covered_value is None:
        return Metric(0, 0, None, False, "metric lacks count or covered fields")

    total = _as_nonnegative_int(total_value, f"{context}.count")
    covered = _as_nonnegative_int(covered_value, f"{context}.covered")
    if covered > total:
        raise CoverageReportError(f"{context}.covered cannot exceed count")

    if total == 0:
        return Metric(covered, total, None, False, "LLVM reported no measurable items")
    return Metric(covered, total, _percent(covered, total), True)


def _merge_metrics(metrics: Iterable[Metric]) -> Metric:
    values = list(metrics)
    covered = sum(metric.covered for metric in values)
    total = sum(metric.total for metric in values)
    if total == 0:
        reasons = sorted({metric.reason for metric in values if metric.reason})
        reason = reasons[0] if reasons else "no measurable items"
        return Metric(covered, total, None, False, reason)
    return Metric(covered, total, _percent(covered, total), True)


def _normalize_path(filename: str, repo_root: Path) -> str | None:
    """Normalize absolute or relative LLVM paths to repo-relative POSIX paths."""

    raw = filename.replace("\\", "/")
    root = repo_root.resolve()
    if os.path.isabs(raw):
        absolute = Path(os.path.normpath(raw))
        try:
            relative = absolute.relative_to(root)
        except ValueError:
            return None
        normalized = relative.as_posix()
    else:
        normalized = PurePosixPath(os.path.normpath(raw)).as_posix()

    normalized = normalized.removeprefix("./")
    if normalized == ".." or normalized.startswith("../"):
        return None
    return normalized


def _summary_for_file(entry: Mapping[str, Any]) -> Mapping[str, Any]:
    summary = entry.get("summary")
    if summary is None:
        summary = entry
    return _require_mapping(summary, "file summary")


def parse_report(report: Mapping[str, Any], repo_root: Path) -> tuple[list[CoverageFile], dict[str, Any]]:
    """Parse a merged cargo-llvm-cov JSON report.

    The current cargo-llvm-cov shape is ``data[].files[].summary``.  The
    parser also accepts direct per-file metric fields so a harmless report
    shape change does not silently produce zero coverage.
    """

    root = _require_mapping(report, "coverage report")
    data_value = root.get("data")
    if isinstance(data_value, Mapping):
        data = [data_value]
    else:
        data = _require_sequence(data_value, "coverage report.data")
    if not data:
        raise CoverageReportError("coverage report.data is empty")

    files_by_path: dict[str, CoverageFile] = {}
    for data_index, raw_data in enumerate(data):
        data_item = _require_mapping(raw_data, f"coverage report.data[{data_index}]")
        raw_files = data_item.get("files")
        if raw_files is None:
            continue
        for file_index, raw_file in enumerate(
            _require_sequence(raw_files, f"coverage report.data[{data_index}].files")
        ):
            entry = _require_mapping(
                raw_file,
                f"coverage report.data[{data_index}].files[{file_index}]",
            )
            filename = entry.get("filename", entry.get("file"))
            if not isinstance(filename, str) or not filename:
                raise CoverageReportError(
                    f"coverage report.data[{data_index}].files[{file_index}] lacks filename"
                )
            path = _normalize_path(filename, repo_root)
            if path is None:
                # LLVM may include files outside the repository (for example
                # dependency or toolchain sources). They are never part of
                # this policy denominator and are intentionally ignored.
                continue

            summary = _summary_for_file(entry)
            metrics = {
                dimension: _parse_metric(
                    summary.get(dimension),
                    f"{path}.{dimension}",
                )
                for dimension in DIMENSIONS
            }
            parsed = CoverageFile(path, metrics)
            previous = files_by_path.get(path)
            if previous is None:
                files_by_path[path] = parsed
            elif previous.metrics != parsed.metrics:
                raise CoverageReportError(
                    f"duplicate report entries for {path} have different metrics; "
                    "use a merged cargo-llvm-cov JSON report"
                )

    if not files_by_path:
        raise CoverageReportError("coverage report contains no repository-local files")

    raw_tool = root.get("cargo_llvm_cov", {})
    tool = _require_mapping(raw_tool, "coverage report.cargo_llvm_cov") if raw_tool else {}
    stable_tool = {
        "name": "cargo-llvm-cov",
        **({"version": tool["version"]} if isinstance(tool.get("version"), str) else {}),
    }
    metadata = {
        "report_type": root.get("type"),
        "report_version": root.get("version"),
        "tool": stable_tool,
    }
    return [files_by_path[path] for path in sorted(files_by_path)], metadata


def _matches(path: str, pattern: str) -> bool:
    """Match repository paths with both fnmatch and pathlib glob semantics."""

    patterns = {pattern}
    # Python's glob implementations disagree about whether ``**/`` can
    # match zero directories. Include the zero-directory form explicitly so
    # ``src/**/*.rs`` covers both ``src/lib.rs`` and ``src/module/lib.rs``.
    if "/**/" in pattern:
        patterns.add(pattern.replace("/**/", "/"))
    if pattern.startswith("**/"):
        patterns.add(pattern.removeprefix("**/"))
    return any(
        fnmatch.fnmatchcase(path, candidate) or PurePosixPath(path).match(candidate)
        for candidate in patterns
    )


def _matches_any(path: str, patterns: Iterable[str]) -> bool:
    return any(_matches(path, pattern) for pattern in patterns)


def _load_policy(policy: Mapping[str, Any]) -> Mapping[str, Any]:
    root = _require_mapping(policy, "coverage policy")
    version = root.get("schema_version")
    if version != 1:
        raise CoverageReportError(f"unsupported coverage policy schema_version: {version!r}")
    denominator = _require_mapping(root.get("denominator"), "coverage policy.denominator")
    for key in ("include", "exclude"):
        values = denominator.get(key)
        if not isinstance(values, list) or not all(isinstance(value, str) for value in values):
            raise CoverageReportError(f"coverage policy.denominator.{key} must be a string array")
    targets = _require_mapping(root.get("targets"), "coverage policy.targets")
    if "overall" not in targets:
        raise CoverageReportError("coverage policy.targets.overall is required")
    return root


def _select_denominator(files: list[CoverageFile], policy: Mapping[str, Any]) -> list[CoverageFile]:
    denominator = _require_mapping(policy["denominator"], "coverage policy.denominator")
    includes = denominator["include"]
    excludes = denominator["exclude"]
    selected = [
        file
        for file in files
        if _matches_any(file.path, includes) and not _matches_any(file.path, excludes)
    ]
    if not selected:
        raise CoverageReportError("coverage policy selected no denominator files")
    return selected


def _complete_source_inventory(
    files: list[CoverageFile],
    policy: Mapping[str, Any],
    repo_root: Path,
) -> list[CoverageFile]:
    """Add zero-measurement entries for source files absent from LLVM JSON.

    LLVM omits files that contain no instrumentable executable item (for
    example a module containing only declarations). Keeping those paths in the
    inventory makes the denominator auditable and prevents a missing source
    file from disappearing silently. Their metric totals remain zero because
    there is no measurable LLVM item to count.
    """

    denominator = _require_mapping(policy["denominator"], "coverage policy.denominator")
    includes = denominator["include"]
    excludes = denominator["exclude"]
    by_path = {file.path: file for file in files}
    source_paths: set[str] = set()
    for pattern in includes:
        for candidate in repo_root.glob(pattern):
            if not candidate.is_file():
                continue
            relative = _normalize_path(str(candidate), repo_root)
            if relative and not _matches_any(relative, excludes):
                source_paths.add(relative)

    for path in sorted(source_paths):
        if path in by_path:
            continue
        by_path[path] = CoverageFile(
            path,
            {
                dimension: Metric(
                    0,
                    0,
                    None,
                    False,
                    "source path is absent from LLVM JSON (no measurable executable items)",
                )
                for dimension in DIMENSIONS
            },
        )
    return [by_path[path] for path in sorted(by_path)]


def _aggregate_file_metrics(files: Iterable[CoverageFile]) -> dict[str, Metric]:
    values = list(files)
    return {
        dimension: _merge_metrics(file.metrics[dimension] for file in values)
        for dimension in DIMENSIONS
    }


def _metric_table(metrics: Mapping[str, Metric]) -> dict[str, Any]:
    return {dimension: metrics[dimension].as_dict() for dimension in DIMENSIONS}


def _format_percent(percent: float | None) -> str:
    return "n/a" if percent is None else f"{percent:.2f}%"


def _format_count(metric: Metric) -> str:
    return f"{metric.covered}/{metric.total}" if metric.supported else "n/a"


def _target_result(
    name: str,
    config: Mapping[str, Any],
    denominator_files: list[CoverageFile],
    all_files: list[CoverageFile],
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    metric_name = config.get("metric", "lines")
    if metric_name not in DIMENSIONS:
        raise CoverageReportError(f"target {name!r} uses unknown metric {metric_name!r}")

    if config.get("scope") == "denominator":
        scoped_files = denominator_files
        missing_patterns: list[str] = []
    else:
        patterns = config.get("files")
        if not isinstance(patterns, list) or not all(isinstance(value, str) for value in patterns):
            raise CoverageReportError(f"target {name!r}.files must be a string array")
        scoped_files = [file for file in all_files if _matches_any(file.path, patterns)]
        missing_patterns = [
            pattern for pattern in patterns if not any(_matches(file.path, pattern) for file in all_files)
        ]

    metrics = _aggregate_file_metrics(scoped_files) if scoped_files else {
        dimension: Metric(0, 0, None, False, "target scope has no matching files")
        for dimension in DIMENSIONS
    }
    metric = metrics[metric_name]
    minimum = config.get("minimum_percent")
    if isinstance(minimum, bool) or not isinstance(minimum, (int, float)):
        raise CoverageReportError(f"target {name!r}.minimum_percent must be numeric")

    issues: list[dict[str, Any]] = []
    status = "pass"
    if missing_patterns:
        status = "partial" if scoped_files else "missing"
        issues.append(
            {
                "target": name,
                "kind": "missing_scope",
                "message": (
                    f"target {name} is missing scope path(s): "
                    + ", ".join(missing_patterns)
                ),
                "patterns": missing_patterns,
                "action": "restore the scope or explicitly review the historical/module boundary before enforcement",
            }
        )
    if not metric.supported:
        status = "unsupported" if not missing_patterns else status
        if metric_name in ("branches", "mcdc"):
            issues.append(
                {
                    "target": name,
                    "kind": "unsupported_metric",
                    "message": (
                        f"target {name} requested {metric_name} coverage, but the report contains no measurable "
                        f"{metric_name} items"
                    ),
                    "action": "keep this dimension report-only and enable it only after cargo-llvm-cov/LLVM support is stable",
                }
            )
    elif metric.percent is not None and metric.percent < float(minimum):
        status = "fail" if not missing_patterns else status
        issues.append(
            {
                "target": name,
                "kind": "below_threshold",
                "message": (
                    f"target {name} is at {_format_percent(metric.percent)} for {metric_name}; "
                    f"required >= {float(minimum):.2f}%"
                ),
                "actual_percent": metric.percent,
                "required_percent": round(float(minimum), 2),
                "action": (
                    "add focused tests for the uncovered production paths, then rerun the report and review "
                    "the generated missing-line details"
                ),
            }
        )

    target = {
        "description": config.get("description"),
        "metric": metric_name,
        "minimum_percent": round(float(minimum), 2),
        "status": status,
        "files": sorted(file.path for file in scoped_files),
        "missing_patterns": missing_patterns,
        "metrics": _metric_table(metrics),
    }
    for optional_key in ("branch_intent", "boundary_matrix"):
        if optional_key in config:
            target[optional_key] = config[optional_key]
    return target, issues


def evaluate(
    report: Mapping[str, Any],
    policy: Mapping[str, Any],
    repo_root: Path,
    mode: str = "report-only",
    *,
    commit_sha: str | None = None,
    label: str | None = None,
    report_name: str | None = None,
) -> dict[str, Any]:
    """Evaluate a report and return the deterministic machine-readable result."""

    policy = _load_policy(policy)
    files, report_metadata = parse_report(report, repo_root)
    files = _complete_source_inventory(files, policy, repo_root)
    denominator_files = _select_denominator(files, policy)
    overall_metrics = _aggregate_file_metrics(denominator_files)

    targets_config = _require_mapping(policy["targets"], "coverage policy.targets")
    targets: dict[str, Any] = {}
    issues: list[dict[str, Any]] = []
    for name in sorted(targets_config):
        target, target_issues = _target_result(
            name,
            _require_mapping(targets_config[name], f"coverage policy.targets.{name}"),
            denominator_files,
            files,
        )
        targets[name] = target
        issues.extend(target_issues)

    result = {
        "schema_version": 1,
        "mode": mode,
        "metadata": {
            "commit_sha": commit_sha,
            "label": label,
            "report_name": report_name,
            "report_type": report_metadata.get("report_type"),
            "report_version": report_metadata.get("report_version"),
            "tool": report_metadata.get("tool"),
            "policy_tool": policy.get("tool"),
        },
        "denominator": {
            "include": policy["denominator"]["include"],
            "exclude": policy["denominator"]["exclude"],
            "file_count": len(denominator_files),
            "files": sorted(file.path for file in denominator_files),
            "unmeasured_files": sorted(
                file.path
                for file in denominator_files
                if all(not file.metrics[dimension].supported for dimension in DIMENSIONS)
            ),
            "metrics": _metric_table(overall_metrics),
        },
        "targets": targets,
        "issues": sorted(
            issues,
            key=lambda issue: (issue.get("target", ""), issue.get("kind", ""), issue.get("message", "")),
        ),
    }

    blocking_issues = [
        issue
        for issue in result["issues"]
        if issue["kind"] in ("missing_scope", "below_threshold")
    ]
    result["status"] = "pass" if not blocking_issues else "thresholds_below_target"
    result["enforce_exit_code"] = 1 if mode == "enforce" and blocking_issues else 0
    return result


def render_markdown(result: Mapping[str, Any]) -> str:
    """Render the result as a stable GitHub-summary-friendly Markdown document."""

    metadata = result["metadata"]
    lines = [
        "# Core coverage report",
        "",
        f"- Mode: `{result['mode']}`",
        f"- Status: `{result['status']}`",
        f"- Commit: `{metadata.get('commit_sha') or 'not supplied'}`",
        f"- Tool: `{(metadata.get('tool') or {}).get('version') or 'unknown'}`",
        "",
        "## Denominator",
        "",
        f"Production files measured: **{result['denominator']['file_count']}**",
        "",
        "| Dimension | Covered | Total | Percent |",
        "| --- | ---: | ---: | ---: |",
    ]
    for dimension, metric in result["denominator"]["metrics"].items():
        lines.append(
            f"| {dimension} | {metric['covered']} | {metric['total']} | "
            f"{_format_percent(metric['percent'])} |"
        )

    lines.extend(
        [
            "",
            "## Named targets",
            "",
            "| Target | Metric | Covered/total | Actual | Required | Status |",
            "| --- | --- | ---: | ---: | ---: | --- |",
        ]
    )
    for name, target in result["targets"].items():
        metric = target["metrics"][target["metric"]]
        lines.append(
            f"| {name} | {target['metric']} | {_format_count(Metric(metric['covered'], metric['total'], metric['percent'], metric['supported']))} | "
            f"{_format_percent(metric['percent'])} | {target['minimum_percent']:.2f}% | {target['status']} |"
        )

    issues = result["issues"]
    lines.extend(["", "## Actionable review items", ""])
    if not issues:
        lines.append("No threshold or scope issues were reported.")
    else:
        for issue in issues:
            lines.append(f"- **{issue['target']} / {issue['kind']}:** {issue['message']}. {issue['action']}.")

    if result["mode"] == "report-only":
        lines.extend(
            [
                "",
                "> This is the initial report-only stage. Threshold and scope issues are visible but do not fail CI.",
            ]
        )
    return "\n".join(lines) + "\n"


def _write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _write_json(path: Path, value: Mapping[str, Any]) -> None:
    _write_text(path, json.dumps(value, indent=2, sort_keys=True) + "\n")


def build_argument_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path, help="cargo-llvm-cov JSON report")
    parser.add_argument("--policy", type=Path, default=DEFAULT_POLICY)
    parser.add_argument("--repo-root", type=Path, default=ROOT)
    parser.add_argument("--mode", choices=("report-only", "enforce"), default="report-only")
    parser.add_argument("--output-json", type=Path, help="write deterministic JSON summary here")
    parser.add_argument("--output-markdown", type=Path, help="write deterministic Markdown summary here")
    parser.add_argument("--commit-sha", help="commit represented by the report")
    parser.add_argument("--label", help="human-readable baseline label")
    parser.add_argument("--print-markdown", action="store_true", help="print Markdown summary to stdout")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_argument_parser().parse_args(argv)
    try:
        report = json.loads(args.input.read_text(encoding="utf-8"))
        policy = json.loads(args.policy.read_text(encoding="utf-8"))
        result = evaluate(
            report,
            policy,
            args.repo_root,
            args.mode,
            commit_sha=args.commit_sha,
            label=args.label,
            report_name=args.input.name,
        )
        markdown = render_markdown(result)
        if args.output_json:
            _write_json(args.output_json, result)
        else:
            print(json.dumps(result, indent=2, sort_keys=True))
        if args.output_markdown:
            _write_text(args.output_markdown, markdown)
        if args.print_markdown:
            print(markdown, end="")
        print(
            f"core coverage: {result['status']} ({len(result['issues'])} review item(s)); "
            f"mode={args.mode}",
            file=sys.stderr,
        )
        return int(result["enforce_exit_code"])
    except (OSError, json.JSONDecodeError, CoverageReportError) as error:
        print(f"core coverage error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
