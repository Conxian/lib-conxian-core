#!/usr/bin/env python3
"""Validate the crates.io publishing workflow's release safety invariants."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


DEFAULT_WORKFLOW = (
    Path(__file__).resolve().parents[1] / ".github" / "workflows" / "crates-publish.yml"
)
STEP_HEADER = re.compile(r"^(?P<indent>[ \t]*)-[ \t]+name:[ \t]*(?P<name>.*)$")
KEY_VALUE = re.compile(
    r"^(?P<indent>[ \t]*)(?P<key>[A-Za-z0-9_-]+):(?:\s*(?P<value>.*))?$"
)


@dataclass(frozen=True)
class Step:
    """A named GitHub Actions step and its source lines."""

    name: str
    start_line: int
    lines: tuple[str, ...]


def _indent_width(value: str) -> int:
    """Return a stable indentation width for spaces or tabs."""

    return len(value.expandtabs(2))


def _steps(workflow: str) -> list[Step]:
    """Extract named step blocks without requiring a third-party YAML parser."""

    lines = workflow.splitlines()
    steps: list[Step] = []
    current_name: str | None = None
    current_start = 0
    current_indent = 0
    current_lines: list[str] = []

    def finish_current() -> None:
        nonlocal current_name, current_start, current_lines
        if current_name is not None:
            steps.append(Step(current_name, current_start, tuple(current_lines)))
        current_name = None
        current_start = 0
        current_lines = []

    for line_number, line in enumerate(lines, start=1):
        match = STEP_HEADER.match(line)
        if match:
            indent = _indent_width(match.group("indent"))
            if current_name is not None and indent <= current_indent:
                finish_current()

            if current_name is None:
                current_name = match.group("name").strip()
                current_start = line_number
                current_indent = indent
            else:
                current_lines.append(line)
            continue

        if current_name is not None:
            current_lines.append(line)

    finish_current()
    return steps


def _step_by_name(steps: list[Step], name: str) -> Step | None:
    for step in steps:
        if step.name == name:
            return step
    return None


def _step_using(steps: list[Step], action_prefix: str) -> Step | None:
    for step in steps:
        if any(
            re.match(rf"^\s*uses:\s*{re.escape(action_prefix)}@", line)
            for line in step.lines
        ):
            return step
    return None


def _key_values(lines: tuple[str, ...] | list[str], key: str):
    for line_number, line in enumerate(lines, start=1):
        match = KEY_VALUE.match(line)
        if match and match.group("key") == key:
            yield line_number, (match.group("value") or "").strip()


def _is_real_publish_step(step: Step) -> bool:
    return any(
        re.search(r"\bcargo\s+publish\b", line) and "--dry-run" not in line
        for line in step.lines
    )


def _step_id(step: Step) -> str | None:
    for _, value in _key_values(step.lines, "id"):
        if re.fullmatch(r"[A-Za-z0-9_-]+", value):
            return value
    return None


def _if_expression(step: Step) -> str | None:
    """Read a one-line or folded/multiline GitHub Actions if expression."""

    lines = list(step.lines)
    for index, line in enumerate(lines):
        match = KEY_VALUE.match(line)
        if not match or match.group("key") != "if":
            continue

        value = (match.group("value") or "").strip()
        if value not in {"|", ">", "|-", ">-", "|+", ">+"}:
            return value

        key_indent = _indent_width(match.group("indent"))
        continuation: list[str] = []
        for continuation_line in lines[index + 1 :]:
            if continuation_line.strip():
                indentation = continuation_line[: len(continuation_line) - len(continuation_line.lstrip())]
                if _indent_width(indentation) <= key_indent:
                    break
                continuation.append(continuation_line.strip())
        return " ".join(continuation)

    return None


def _workflow_has_dry_run_input(workflow: str) -> list[str]:
    errors: list[str] = []
    if not re.search(
        r"(?ms)^[ \t]{2}workflow_dispatch:[ \t]*$.*?^[ \t]{4}inputs:[ \t]*$.*?^[ \t]{6}dry_run:[ \t]*$",
        workflow,
    ):
        errors.append("workflow_dispatch must define a dry_run input")
    if not re.search(
        r"(?ms)^[ \t]{6}dry_run:[ \t]*$.*?^[ \t]{8}type:[ \t]*boolean[ \t]*$",
        workflow,
    ):
        errors.append("dry_run input must be boolean")
    if not re.search(
        r"(?ms)^[ \t]{6}dry_run:[ \t]*$.*?^[ \t]{8}default:[ \t]*true[ \t]*$",
        workflow,
    ):
        errors.append("dry_run input must default to true")
    if "cargo publish --dry-run" not in workflow:
        errors.append("manual dry-run path must invoke `cargo publish --dry-run`")
    return errors


def verify(workflow_path: Path) -> list[str]:
    """Return all release-hygiene violations found in the workflow."""

    try:
        workflow = workflow_path.read_text(encoding="utf-8")
    except OSError as error:
        return [f"Unable to read workflow {workflow_path}: {error}"]

    violations: list[str] = []

    if re.search(r"\bcargo\s+publish\b[^\n]*--tokenless\b", workflow):
        violations.append("unsupported `cargo publish --tokenless` is present")

    violations.extend(_workflow_has_dry_run_input(workflow))

    steps = _steps(workflow)
    publish_step = _step_by_name(steps, "Publish to crates.io")
    real_publish_steps = [step for step in steps if _is_real_publish_step(step)]
    if not real_publish_steps:
        violations.append("no real `cargo publish` step found")
    else:
        for step in real_publish_steps:
            publish_text = "\n".join(step.lines)
            if any(True for _ in _key_values(step.lines, "continue-on-error")):
                violations.append(f"real publish step {step.name!r} uses continue-on-error")

            has_token_env = bool(
                re.search(
                    r"CARGO_REGISTRY_TOKEN\s*:\s*\$\{\{\s*secrets\.CARGO_REGISTRY_TOKEN\s*\}\}",
                    publish_text,
                )
            )
            has_token_check = bool(
                re.search(r"-z[ \t]+[^\n]*CARGO_REGISTRY_TOKEN", publish_text)
                and re.search(r"\bexit\s+1\b", publish_text)
            )
            if not has_token_env or not has_token_check:
                violations.append(
                    f"real publish step {step.name!r} must require CARGO_REGISTRY_TOKEN and fail when absent"
                )

    if publish_step is None:
        violations.append("could not find the 'Publish to crates.io' step")

    release_step = _step_using(steps, "softprops/action-gh-release")
    if release_step is None:
        violations.append("could not find the GitHub Release creation step")
    else:
        release_condition = _if_expression(release_step)
        if release_condition is None:
            violations.append(
                "GitHub Release creation must have an explicit if gate "
                f"(step starts on line {release_step.start_line})"
            )
        else:
            if not re.search(r"github\.event_name\s*==\s*['\"]push['\"]", release_condition):
                violations.append(
                    "GitHub Release creation must remain limited to push/tag executions "
                    f"(step starts on line {release_step.start_line})"
                )

            publish_step_id = _step_id(real_publish_steps[0]) if real_publish_steps else None
            has_success_function = bool(re.search(r"\bsuccess\s*\(\s*\)", release_condition))
            has_publish_outcome = bool(
                publish_step_id
                and re.search(
                    rf"steps\.{re.escape(publish_step_id)}\.outcome\s*==\s*['\"]success['\"]",
                    release_condition,
                )
            )
            if not (has_success_function or has_publish_outcome):
                violations.append(
                    "GitHub Release creation must require successful publication "
                    f"(step starts on line {release_step.start_line})"
                )

    return violations


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "workflow",
        nargs="?",
        type=Path,
        default=None,
        help="workflow to validate (default: .github/workflows/crates-publish.yml)",
    )
    parser.add_argument(
        "--workflow",
        dest="workflow_option",
        type=Path,
        help="workflow to validate (legacy option form)",
    )
    args = parser.parse_args(argv)
    workflow_path = args.workflow_option or args.workflow or DEFAULT_WORKFLOW

    violations = verify(workflow_path)
    if violations:
        print(f"Release hygiene verification failed for {workflow_path}:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        return 1

    print(f"Release hygiene verification passed for {workflow_path}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
