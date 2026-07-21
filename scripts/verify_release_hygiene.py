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
DEFAULT_MAIN_WORKFLOW = DEFAULT_WORKFLOW.parent / "main.yml"
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


def _publish_command_has_flags(text: str, *flags: str) -> bool:
    """Return whether one cargo publish command contains every requested flag."""

    for line in text.splitlines():
        if not re.search(r"\bcargo\s+publish\b", line):
            continue
        if all(
            re.search(rf"(?<!\S){re.escape(flag)}(?=\s|$)", line)
            for flag in flags
        ):
            return True
    return False


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


def _workflow_has_release_inputs(workflow: str) -> list[str]:
    errors: list[str] = []
    if not re.search(
        r"(?ms)^[ \t]{2}workflow_dispatch:[ \t]*$.*?^[ \t]{4}inputs:[ \t]*$",
        workflow,
    ):
        errors.append("workflow_dispatch must define explicit release inputs")

    mode_block = re.search(
        r"(?ms)^[ \t]{6}mode:[ \t]*$.*?(?=^[ \t]{6}[A-Za-z0-9_-]+:[ \t]*$|\Z)",
        workflow,
    )
    mode_text = mode_block.group(0) if mode_block else ""
    if not mode_text:
        errors.append("workflow_dispatch must define a mode input")
    else:
        if not re.search(r"(?m)^[ \t]{8}required:[ \t]*true[ \t]*$", mode_text):
            errors.append("mode input must be required")
        if not re.search(r"(?m)^[ \t]{8}default:[ \t]*dry-run[ \t]*$", mode_text):
            errors.append("mode input must default to dry-run")
        if not re.search(r"(?m)^[ \t]{8}type:[ \t]*choice[ \t]*$", mode_text):
            errors.append("mode input must be a choice")
        for option in ("dry-run", "publish", "release-only"):
            if not re.search(rf"(?m)^[ \t]{{10}}-[ \t]*{re.escape(option)}[ \t]*$", mode_text):
                errors.append(f"mode input must offer the {option!r} choice")

    release_tag_block = re.search(
        r"(?ms)^[ \t]{6}release_tag:[ \t]*$.*?(?=^[ \t]{6}[A-Za-z0-9_-]+:[ \t]*$|\Z)",
        workflow,
    )
    release_tag_text = release_tag_block.group(0) if release_tag_block else ""
    if not release_tag_text:
        errors.append("workflow_dispatch must define a release_tag input")
    else:
        if not re.search(r"(?m)^[ \t]{8}required:[ \t]*false[ \t]*$", release_tag_text):
            errors.append("release_tag input must remain optional for dry-run mode")
        if not re.search(r"(?m)^[ \t]{8}type:[ \t]*string[ \t]*$", release_tag_text):
            errors.append("release_tag input must be a string")
        if not re.search(r"(?m)^[ \t]{8}default:[ \t]*(?:''|\"\")[ \t]*$", release_tag_text):
            errors.append("release_tag input must default to empty")

    if re.search(r"(?m)^[ \t]{6}dry_run:[ \t]*$", workflow):
        errors.append("legacy dry_run input must not be present")
    if not re.search(
        r"(?m)^\s*RELEASE_TAG:\s*\$\{\{\s*github\.event_name\s*==\s*['\"]push['\"]\s*&&\s*github\.ref_name\s*\|\|\s*inputs\.release_tag\s*\}\}",
        workflow,
    ):
        errors.append("job must derive RELEASE_TAG from the push ref or release_tag input")
    if "git fetch --force origin \"refs/tags/${RELEASE_TAG}:refs/tags/${RELEASE_TAG}\"" not in workflow:
        errors.append("manual real-release flows must fetch the explicit release_tag")
    if "git checkout --detach \"refs/tags/${RELEASE_TAG}\"" not in workflow:
        errors.append("manual real-release flows must check out the explicit release_tag")
    if not _publish_command_has_flags(workflow, "--dry-run", "--locked"):
        errors.append(
            "manual dry-run path must invoke `cargo publish --dry-run --locked`"
        )
    if "vX.Y.Z[-prerelease][+build]" not in workflow:
        errors.append(
            "release_tag documentation and validation must describe the supported SemVer form"
        )
    if "is_valid_release_tag" not in workflow:
        errors.append(
            "manual real-release validation must reuse the SemVer release-tag guard"
        )
    if "strict vX.Y.Z form" in workflow or "^v[0-9]+\\.[0-9]+\\.[0-9]+$" in workflow:
        errors.append("workflow must not retain numeric-only release-tag validation")
    return errors


def _verify_main_workflow(workflow_path: Path) -> list[str]:
    try:
        workflow = workflow_path.read_text(encoding="utf-8")
    except OSError as error:
        return [f"Unable to read main workflow {workflow_path}: {error}"]

    violations: list[str] = []
    if "python scripts/verify_release_version.py --phase source-only" not in workflow:
        violations.append("main CI must run the release version guard in source-only mode")
    if "python -m unittest discover -s scripts/tests -p 'test_*.py'" not in workflow:
        violations.append("main CI must run the focused standard-library Python tests")
    return violations


def verify(workflow_path: Path, main_workflow_path: Path = DEFAULT_MAIN_WORKFLOW) -> list[str]:
    """Return all release-hygiene violations found in the workflow."""

    try:
        workflow = workflow_path.read_text(encoding="utf-8")
    except OSError as error:
        return [f"Unable to read workflow {workflow_path}: {error}"]

    violations: list[str] = []

    if not re.search(
        r"(?ms)^permissions:[ \t]*$.*?^[ \t]{2}contents:[ \t]*write[ \t]*$",
        workflow,
    ):
        violations.append("release workflow must grant top-level contents: write permission")

    if re.search(r"\bcargo\s+publish\b[^\n]*--tokenless\b", workflow):
        violations.append("unsupported `cargo publish --tokenless` is present")

    violations.extend(_workflow_has_release_inputs(workflow))

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
            if not _publish_command_has_flags(publish_text, "--locked"):
                violations.append(
                    f"real publish step {step.name!r} must invoke `cargo publish --locked`"
                )

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
    else:
        publish_condition = _if_expression(publish_step) or ""
        if "github.event_name == 'push'" not in publish_condition:
            violations.append("real publication must remain available for tag pushes")
        if not re.search(r"inputs\.mode\s*==\s*['\"]publish['\"]", publish_condition):
            violations.append("manual real publication must require mode == publish")
        if "release-only" in publish_condition:
            violations.append("release-only recovery must never execute cargo publish")

    dry_run_step = _step_by_name(steps, "Publish to crates.io (dry run)")
    if dry_run_step is None:
        violations.append("could not find the manual dry-run publication step")
    else:
        dry_run_text = "\n".join(dry_run_step.lines)
        if not _publish_command_has_flags(dry_run_text, "--dry-run", "--locked"):
            violations.append(
                "manual dry-run path must invoke `cargo publish --dry-run --locked`"
            )
        dry_run_condition = _if_expression(dry_run_step) or ""
        if not re.search(r"inputs\.mode\s*==\s*['\"]dry-run['\"]", dry_run_condition):
            violations.append("dry-run publication must require mode == dry-run")

    source_step = _step_by_name(steps, "Validate source parity (dry run)")
    if source_step is None or "--phase source-only" not in "\n".join(source_step.lines):
        violations.append("dry-run mode must run the source-only version guard")

    pre_publish_step = _step_by_name(steps, "Validate pre-publish parity")
    if pre_publish_step is None:
        violations.append("real publication must run a pre-publish version guard")
    else:
        pre_publish_text = "\n".join(pre_publish_step.lines)
        pre_publish_condition = _if_expression(pre_publish_step) or ""
        if "--phase pre-publish" not in pre_publish_text:
            violations.append("pre-publish step must invoke the pre-publish version guard")
        if "--tag \"$RELEASE_TAG\"" not in pre_publish_text:
            violations.append("pre-publish guard must validate RELEASE_TAG")
        if "--source-revision" not in pre_publish_text:
            violations.append("pre-publish guard must validate tag/source identity")
        if "github.event_name == 'push'" not in pre_publish_condition:
            violations.append("pre-publish guard must run for tag pushes")
        if not re.search(r"inputs\.mode\s*==\s*['\"]publish['\"]", pre_publish_condition):
            violations.append("pre-publish guard must run for manual publish mode")

    recovery_step = _step_by_name(steps, "Validate published candidate for recovery")
    if recovery_step is None:
        violations.append("release-only recovery must validate the published candidate")
    else:
        recovery_text = "\n".join(recovery_step.lines)
        recovery_condition = _if_expression(recovery_step) or ""
        if "--phase post-publish" not in recovery_text:
            violations.append("recovery validation must invoke post-publish mode")
        if not re.search(r"inputs\.mode\s*==\s*['\"]release-only['\"]", recovery_condition):
            violations.append("recovery validation must be limited to release-only mode")

    registry_step = _step_by_name(steps, "Verify crates.io publication")
    if registry_step is None:
        violations.append("real publication must verify crates.io before release creation")
    else:
        registry_text = "\n".join(registry_step.lines)
        registry_condition = _if_expression(registry_step) or ""
        if "--phase post-publish" not in registry_text:
            violations.append("registry verification must invoke post-publish mode")
        if "steps.publish.outcome == 'success'" not in registry_condition:
            violations.append("registry verification must require successful cargo publish")
        if not re.search(r"--registry-attempts\s+1[2-9]", registry_text):
            violations.append("registry verification must use bounded propagation retries")

    release_step = _step_by_name(steps, "Create GitHub Release (idempotent)")
    if release_step is None:
        violations.append("could not find the idempotent GitHub Release creation step")

    release_command_steps = [
        step
        for step in steps
        if any(re.search(r"\bgh\s+release\s+(?:view|create)\b", line) for line in step.lines)
    ]
    if not release_command_steps:
        violations.append("release commands must use gh release view/create")
    else:
        for step in release_command_steps:
            release_command_text = "\n".join(step.lines)
            if not re.search(
                r"(?m)^\s*GH_TOKEN:\s*\$\{\{\s*secrets\.GITHUB_TOKEN\s*\}\}\s*$",
                release_command_text,
            ):
                violations.append(
                    f"release command step {step.name!r} must wire GH_TOKEN from secrets.GITHUB_TOKEN"
                )

    if release_step is not None:
        release_text = "\n".join(release_step.lines)
        release_condition = _if_expression(release_step)
        if release_condition is None:
            violations.append(
                "GitHub Release creation must have an explicit if gate "
                f"(step starts on line {release_step.start_line})"
            )
        else:
            if "success()" not in release_condition:
                violations.append(
                    "GitHub Release creation must require successful preceding steps "
                    f"(step starts on line {release_step.start_line})"
                )
            if "steps.publish.outcome == 'success'" not in release_condition:
                violations.append(
                    "GitHub Release creation must require successful publication "
                    f"(step starts on line {release_step.start_line})"
                )
            if "steps.registry.outcome == 'success'" not in release_condition:
                violations.append(
                    "GitHub Release creation must require successful crates.io verification "
                    f"(step starts on line {release_step.start_line})"
                )
            if "steps.recovery.outcome == 'success'" not in release_condition:
                violations.append(
                    "GitHub Release creation must support successful release-only recovery "
                    f"(step starts on line {release_step.start_line})"
                )
        if "gh release view \"$RELEASE_TAG\"" not in release_text:
            violations.append("GitHub Release creation must be idempotent when the release exists")
        if "gh release create \"$RELEASE_TAG\"" not in release_text:
            violations.append("GitHub Release creation must use the explicit RELEASE_TAG")
        if "--verify-tag" not in release_text:
            violations.append("GitHub Release creation must verify the existing Git tag")
        if "Unable to determine whether GitHub Release" not in release_text:
            violations.append(
                "GitHub Release lookup failures must emit diagnostics and fail closed"
            )
        if "post_create_view_output" not in release_text:
            violations.append(
                "GitHub Release creation failures must re-check for an already-existing release"
            )
        if not re.search(r"--phase\s+post-release(?:\s|\\|$)", release_text):
            violations.append(
                "race recovery must run post-release verification before accepting a failed create"
            )

    post_release_step = _step_by_name(steps, "Verify post-release parity")
    if post_release_step is None:
        violations.append("release flow must verify post-release parity")
    else:
        post_release_text = "\n".join(post_release_step.lines)
        post_release_condition = _if_expression(post_release_step) or ""
        if "--phase post-release" not in post_release_text:
            violations.append("post-release step must invoke post-release mode")
        if "steps.release.outcome == 'success'" not in post_release_condition:
            violations.append("post-release verification must require release creation success")

    step_positions = {step.name: index for index, step in enumerate(steps)}
    required_order = (
        "Validate pre-publish parity",
        "Publish to crates.io",
        "Verify crates.io publication",
        "Create GitHub Release (idempotent)",
        "Verify post-release parity",
    )
    for before, after in zip(required_order, required_order[1:]):
        if before in step_positions and after in step_positions:
            if step_positions[before] >= step_positions[after]:
                violations.append(f"workflow steps must keep {before!r} before {after!r}")

    violations.extend(_verify_main_workflow(main_workflow_path))

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
    parser.add_argument(
        "--main-workflow",
        type=Path,
        default=DEFAULT_MAIN_WORKFLOW,
        help="main CI workflow to validate (default: .github/workflows/main.yml)",
    )
    args = parser.parse_args(argv)
    workflow_path = args.workflow_option or args.workflow or DEFAULT_WORKFLOW

    violations = verify(workflow_path, args.main_workflow)
    if violations:
        print(f"Release hygiene verification failed for {workflow_path}:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        return 1

    print(f"Release hygiene verification passed for {workflow_path}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
