#!/usr/bin/env python3
"""Validate the crates.io publishing workflow's release safety invariants."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


DEFAULT_WORKFLOW = (
    Path(__file__).resolve().parents[1] / ".github" / "workflows" / "crates-publish.yml"
)
STEP_START = re.compile(r"^(?P<indent>[ \t]*)-[ \t]+name:[ \t]*(?P<name>.*)$")


def _step_blocks(text: str) -> list[tuple[str, str]]:
    """Return name and source for each step in the workflow's step list."""

    lines = text.splitlines()
    starts: list[tuple[int, int, str]] = []
    for line_number, line in enumerate(lines):
        match = STEP_START.match(line)
        if match:
            starts.append(
                (line_number, len(match.group("indent")), match.group("name").strip())
            )

    blocks: list[tuple[str, str]] = []
    for index, (start, indent, name) in enumerate(starts):
        end = len(lines)
        for next_start, next_indent, _ in starts[index + 1 :]:
            if next_indent == indent:
                end = next_start
                break
        blocks.append((name, "\n".join(lines[start:end])))
    return blocks


def _has_real_publish_command(block: str) -> bool:
    return any(
        re.search(r"\bcargo\s+publish\b", line) and "--dry-run" not in line
        for line in block.splitlines()
    )


def _step_id(block: str) -> str | None:
    match = re.search(r"(?m)^\s*id:\s*([A-Za-z0-9_-]+)\s*$", block)
    return match.group(1) if match else None


def _if_expression(block: str) -> str | None:
    match = re.search(r"(?m)^\s*if:\s*(.+?)\s*$", block)
    return match.group(1) if match else None


def verify(workflow: Path) -> list[str]:
    errors: list[str] = []
    try:
        text = workflow.read_text(encoding="utf-8")
    except OSError as exc:
        return [f"unable to read {workflow}: {exc}"]

    if "cargo publish --tokenless" in text:
        errors.append("unsupported `cargo publish --tokenless` is present")

    if not re.search(
        r"(?ms)^[ \t]{2}workflow_dispatch:[ \t]*$.*?^[ \t]{4}inputs:[ \t]*$.*?^[ \t]{6}dry_run:[ \t]*$",
        text,
    ):
        errors.append("workflow_dispatch must define a dry_run input")
    if not re.search(
        r"(?ms)^[ \t]{6}dry_run:[ \t]*$.*?^[ \t]{8}type:[ \t]*boolean[ \t]*$",
        text,
    ):
        errors.append("dry_run input must be boolean")
    if not re.search(
        r"(?ms)^[ \t]{6}dry_run:[ \t]*$.*?^[ \t]{8}default:[ \t]*true[ \t]*$",
        text,
    ):
        errors.append("dry_run input must default to true")
    if "cargo publish --dry-run" not in text:
        errors.append("manual dry-run path must invoke `cargo publish --dry-run`")

    steps = _step_blocks(text)
    real_publish_steps = [
        (name, block) for name, block in steps if _has_real_publish_command(block)
    ]
    if not real_publish_steps:
        errors.append("no real `cargo publish` step found")
    else:
        for name, block in real_publish_steps:
            if re.search(r"(?m)^[ \t]*continue-on-error[ \t]*:", block):
                errors.append(f"real publish step {name!r} uses continue-on-error")

            has_token_env = bool(
                re.search(
                    r"CARGO_REGISTRY_TOKEN\s*:\s*\$\{\{\s*secrets\.CARGO_REGISTRY_TOKEN\s*\}\}",
                    block,
                )
            )
            has_token_check = bool(
                re.search(r"-z[ \t]+[^\n]*CARGO_REGISTRY_TOKEN", block)
                and re.search(r"\bexit\s+1\b", block)
            )
            if not has_token_env or not has_token_check:
                errors.append(
                    f"real publish step {name!r} must require CARGO_REGISTRY_TOKEN and fail when absent"
                )

    release_steps = [
        (name, block)
        for name, block in steps
        if re.search(r"github\s+release", name, flags=re.IGNORECASE)
    ]
    if not release_steps:
        errors.append("no GitHub Release step found")
    else:
        for name, block in release_steps:
            expression = _if_expression(block)
            if expression is None:
                errors.append(f"GitHub Release step {name!r} has no explicit if gate")
                continue

            if not re.search(r"github\.event_name\s*==\s*['\"]push['\"]", expression):
                errors.append(
                    f"GitHub Release step {name!r} must be limited to tag push events"
                )

            publish_step_id = _step_id(real_publish_steps[0][1])
            has_success_function = bool(re.search(r"\bsuccess\s*\(\s*\)", expression))
            has_publish_outcome = bool(
                publish_step_id
                and re.search(
                    rf"steps\.{re.escape(publish_step_id)}\.outcome\s*==\s*['\"]success['\"]",
                    expression,
                )
            )
            if not (has_success_function or has_publish_outcome):
                errors.append(
                    f"GitHub Release step {name!r} must require successful publication"
                )

    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "workflow",
        nargs="?",
        type=Path,
        default=DEFAULT_WORKFLOW,
        help="workflow to validate (default: .github/workflows/crates-publish.yml)",
    )
    args = parser.parse_args(argv)

    errors = verify(args.workflow)
    if errors:
        print(f"Release hygiene verification failed for {args.workflow}:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(f"Release hygiene verification passed for {args.workflow}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
