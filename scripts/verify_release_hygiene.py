"""Static regression checks for the crates.io release workflow."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


DEFAULT_WORKFLOW = (
    Path(__file__).resolve().parents[1] / ".github" / "workflows" / "crates-publish.yml"
)
STEP_HEADER = re.compile(r"^(?P<indent>[ \t]*)-\s+name:\s*(?P<name>.*?)\s*$")
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


def _without_comment(value: str) -> str:
    return value.split("#", 1)[0].strip().lower()


def _is_permissive_continue_on_error(value: str) -> bool:
    """Treat unknown or dynamic values as permissive so the guard fails closed."""

    normalized = _without_comment(value)
    return normalized not in {"false", "no", "off", "0"}


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
                continuation_indent = _indent_width(
                    continuation_line[: len(continuation_line) - len(continuation_line.lstrip())]
                )
                if continuation_indent <= key_indent:
                    break
                continuation.append(continuation_line.strip())
        return " ".join(continuation)

    return None


def verify(workflow_path: Path) -> list[str]:
    """Return all release-hygiene violations found in the workflow."""

    try:
        workflow = workflow_path.read_text(encoding="utf-8")
    except OSError as error:
        return [f"Unable to read workflow {workflow_path}: {error}"]

    violations: list[str] = []

    if "--tokenless" in workflow:
        violations.append("unsupported '--tokenless' cargo publish flag is present")

    steps = _steps(workflow)
    publish_step = _step_by_name(steps, "Publish to crates.io")
    if publish_step is None:
        violations.append("could not find the 'Publish to crates.io' step")
    else:
        publish_text = "\n".join(publish_step.lines)
        if not re.search(
            r"CARGO_REGISTRY_TOKEN\s*:\s*\$\{\{\s*secrets\.CARGO_REGISTRY_TOKEN\s*\}\}",
            publish_text,
        ):
            violations.append(
                "the 'Publish to crates.io' step must use the CARGO_REGISTRY_TOKEN secret"
            )
        if not re.search(r"\[\[\s*-z\s+[\"']?\$\{CARGO_REGISTRY_TOKEN:-\}", publish_text):
            violations.append(
                "the 'Publish to crates.io' step must fail when CARGO_REGISTRY_TOKEN is empty"
            )
        elif not re.search(r"\bexit\s+1\b", publish_text):
            violations.append(
                "the 'Publish to crates.io' step must exit non-zero when authentication is unavailable"
            )

    for line_number, value in _key_values(workflow.splitlines(), "continue-on-error"):
        if _is_permissive_continue_on_error(value):
            violation = (
                "the release workflow must not configure permissive continue-on-error "
                f"(line {line_number})"
            )
            if violation not in violations:
                violations.append(violation)

    release_step = _step_using(steps, "softprops/action-gh-release")
    if release_step is None:
        violations.append("could not find the GitHub Release creation step")
    else:
        release_condition = _if_expression(release_step)
        if not release_condition or "success()" not in release_condition:
            violations.append(
                "GitHub Release creation must be explicitly gated with success() "
                f"(step starts on line {release_step.start_line})"
            )
        if not release_condition or "github.event_name" not in release_condition or "push" not in release_condition:
            violations.append(
                "GitHub Release creation must remain limited to push/tag executions "
                f"(step starts on line {release_step.start_line})"
            )

    return violations


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--workflow",
        type=Path,
        default=DEFAULT_WORKFLOW,
        help="workflow file to inspect (default: .github/workflows/crates-publish.yml)",
    )
    args = parser.parse_args()

    print(f"Verifying release workflow hygiene: {args.workflow}")
    violations = verify(args.workflow)
    if violations:
        print("Release hygiene check failed:")
        for violation in violations:
            print(f"- {violation}")
        return 1

    print("Release hygiene check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
