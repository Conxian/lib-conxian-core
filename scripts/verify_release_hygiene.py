#!/usr/bin/env python3
"""Validate the crates.io publishing workflow's release safety invariants."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterable, Mapping
from urllib.error import HTTPError, URLError
from urllib.parse import quote
from urllib.request import Request, urlopen


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_WORKFLOW = (
    REPOSITORY_ROOT / ".github" / "workflows" / "crates-publish.yml"
)
DEFAULT_MAIN_WORKFLOW = DEFAULT_WORKFLOW.parent / "main.yml"
DEFAULT_CI_WORKFLOW = DEFAULT_MAIN_WORKFLOW
CRATES_IO_API = "https://crates.io/api/v1/crates"
REMOTE_STATE_MISSING_EXIT = 10
SEMVER_BODY = r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?"
SEMVER_RE = re.compile(rf"^{SEMVER_BODY}$")
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


@dataclass(frozen=True)
class ReleaseMetadata:
    """Authoritative package identity loaded from the repository."""

    crate_name: str
    version: str


@dataclass(frozen=True)
class RemoteCheck:
    """A fail-closed remote lookup result."""

    state: str
    detail: str


REMOTE_PRESENT = "present"
REMOTE_MISSING = "missing"
REMOTE_ERROR = "error"
REMOTE_MISMATCH = "mismatch"
UrlOpen = Callable[..., object]


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
            re.search(rf"(?<!\S){re.escape(flag)}(?=\s|[;&|]|$)", line)
            for flag in flags
        ):
            return True
    return False


def _publish_command_has_package(text: str, package_name: str) -> bool:
    """Return whether a cargo publish command names the expected package."""

    package_pattern = rf"(?:-p|--package)\s+{re.escape(package_name)}(?:\s|[;&|]|$)"
    return any(
        re.search(r"\bcargo\s+publish\b", line)
        and re.search(package_pattern, line)
        for line in text.splitlines()
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


def _normalize_if_expression(expression: str) -> str:
    """Normalize YAML scalar formatting without weakening the expression check."""

    normalized = re.sub(r"\s+", " ", expression).strip()
    if normalized.startswith("${{") and normalized.endswith("}}"):
        normalized = normalized[3:-2].strip()
    return normalized


def _top_level_block(lines: list[str], key: str) -> tuple[str, ...] | None:
    """Extract a top-level YAML mapping block using indentation only."""

    start_index: int | None = None
    for index, line in enumerate(lines):
        match = KEY_VALUE.match(line)
        if match and _indent_width(match.group("indent")) == 0 and match.group("key") == key:
            start_index = index
            break
    if start_index is None:
        return None

    block: list[str] = []
    for line in lines[start_index + 1 :]:
        if line.strip() and not line.lstrip().startswith("#"):
            match = KEY_VALUE.match(line)
            if match and _indent_width(match.group("indent")) == 0:
                break
        block.append(line)
    return tuple(block)


def _workflow_concurrency_violations(workflow: str) -> list[str]:
    block = _top_level_block(workflow.splitlines(), "concurrency")
    if block is None:
        return ["release workflow must define top-level concurrency protection"]

    group_values = list(_key_values(block, "group"))
    if not group_values:
        errors = ["release workflow concurrency must define a group"]
    else:
        group = group_values[0][1]
        errors = []
        if not re.search(r"\bgithub\.workflow\b", group) or not re.search(r"\bgithub\.ref\b", group):
            errors.append("release workflow concurrency group must be scoped to github.workflow and github.ref")

    cancel_values = list(_key_values(block, "cancel-in-progress"))
    if not cancel_values or cancel_values[0][1].strip().strip("'\"") != "false":
        errors.append("release workflow concurrency must set cancel-in-progress: false")
    return errors


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

    violations.extend(_workflow_concurrency_violations(workflow))
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
        publish_text = "\n".join(publish_step.lines)
        if not _publish_command_has_package(publish_text, "lib-conxian-core"):
            violations.append(
                "Core publication must use `cargo publish ... -p lib-conxian-core`"
            )
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
        if not _publish_command_has_package(dry_run_text, "lib-conxian-core"):
            violations.append(
                "manual dry-run path must target `lib-conxian-core` explicitly"
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

    addon_dry_run_step = _step_by_name(steps, "Verify add-on package (after Core registry propagation)")
    if addon_dry_run_step is None:
        violations.append("real publication must dry-run the add-on after Core registry verification")
    else:
        addon_dry_run_text = "\n".join(addon_dry_run_step.lines)
        addon_dry_run_condition = _if_expression(addon_dry_run_step) or ""
        if not _publish_command_has_flags(addon_dry_run_text, "--dry-run", "--locked"):
            violations.append("add-on verification must invoke `cargo publish --dry-run --locked`")
        if not _publish_command_has_package(
            addon_dry_run_text,
            "lib-conxian-core-enclave",
        ):
            violations.append(
                "add-on verification must target `lib-conxian-core-enclave` explicitly"
            )
        if "steps.registry.outcome == 'success'" not in addon_dry_run_condition:
            violations.append(
                "add-on dry-run must require successful Core registry verification"
            )

    addon_publish_step = _step_by_name(steps, "Publish add-on to crates.io")
    if addon_publish_step is None:
        violations.append("real publication must publish the add-on after its dry-run")
    else:
        addon_publish_text = "\n".join(addon_publish_step.lines)
        addon_publish_condition = _if_expression(addon_publish_step) or ""
        if not _publish_command_has_package(
            addon_publish_text,
            "lib-conxian-core-enclave",
        ):
            violations.append(
                "add-on publication must use `cargo publish ... -p lib-conxian-core-enclave`"
            )
        if "steps.addon_dry_run.outcome == 'success'" not in addon_publish_condition:
            violations.append(
                "add-on publication must require successful add-on dry-run verification"
            )

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
            if "steps.addon_publish.outcome == 'success'" not in release_condition:
                violations.append(
                    "GitHub Release creation must require successful add-on publication "
                    f"(step starts on line {release_step.start_line})"
                )
            if "steps.addon_recovery.outcome == 'success'" not in release_condition:
                violations.append(
                    "GitHub Release creation must require successful add-on recovery verification "
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
        "Verify add-on package (after Core registry propagation)",
        "Publish add-on to crates.io",
        "Create GitHub Release (idempotent)",
        "Verify post-release parity",
    )
    for before, after in zip(required_order, required_order[1:]):
        if before in step_positions and after in step_positions:
            if step_positions[before] >= step_positions[after]:
                violations.append(f"workflow steps must keep {before!r} before {after!r}")

    recovery_position = step_positions.get("Validate published candidate for recovery")
    addon_recovery_position = step_positions.get("Validate add-on candidate for recovery")
    if (
        recovery_position is not None
        and addon_recovery_position is not None
        and recovery_position >= addon_recovery_position
    ):
        violations.append(
            "workflow steps must keep Core recovery validation before add-on recovery validation"
        )

    violations.extend(_verify_main_workflow(main_workflow_path))

    return violations


def _parse_package_field(cargo_toml: str, field: str) -> str:
    in_package = False
    for line in cargo_toml.splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_package = stripped == "[package]"
            continue
        if not in_package:
            continue
        match = re.match(rf"^{re.escape(field)}\s*=\s*['\"]([^'\"]+)['\"]", stripped)
        if match:
            return match.group(1)
    raise ValueError(f"[package].{field} is missing")


def _parse_lock_packages(cargo_lock: str) -> list[dict[str, str]]:
    packages: list[dict[str, str]] = []
    blocks = re.split(r"(?m)^\[\[package\]\]\s*$", cargo_lock)[1:]
    for block in blocks:
        package: dict[str, str] = {}
        for field in ("name", "version", "source"):
            match = re.search(rf"(?m)^{field}\s*=\s*['\"]([^'\"]+)['\"]", block)
            if match:
                package[field] = match.group(1)
        if "name" in package and "version" in package:
            packages.append(package)
    return packages


def _normalize_version(value: str) -> str | None:
    candidate = value.strip()
    if candidate.startswith("v"):
        candidate = candidate[1:]
    return candidate if SEMVER_RE.fullmatch(candidate) else None


def _read_file(root: Path, name: str) -> tuple[str | None, str | None]:
    path = root / name
    try:
        return path.read_text(encoding="utf-8"), None
    except OSError as error:
        return None, f"unable to read {path}: {error}"


def load_release_metadata(root: Path = REPOSITORY_ROOT) -> tuple[ReleaseMetadata | None, list[str]]:
    """Load the package name/version that define the release identity."""

    cargo_toml, error = _read_file(root, "Cargo.toml")
    if error:
        return None, [error]
    assert cargo_toml is not None

    violations: list[str] = []
    try:
        crate_name = _parse_package_field(cargo_toml, "name")
    except ValueError as parse_error:
        violations.append(str(parse_error))
        crate_name = ""
    try:
        version = _parse_package_field(cargo_toml, "version")
    except ValueError as parse_error:
        violations.append(str(parse_error))
        version = ""

    if version and not SEMVER_RE.fullmatch(version):
        violations.append(f"[package].version {version!r} is not a supported semantic version")
    if violations:
        return None, violations
    return ReleaseMetadata(crate_name=crate_name, version=version), []


def _markdown_heading(line: str) -> tuple[int, str] | None:
    match = re.match(r"^(?P<marks>#{1,6})\s+(?P<title>.+?)\s*$", line)
    if not match:
        return None
    title = match.group("title").rstrip("#").strip()
    return len(match.group("marks")), title


def _markdown_section(readme: str, title: str) -> str | None:
    """Return one heading section, excluding later same-level sections."""

    lines = readme.splitlines()
    section_start: int | None = None
    section_level = 0
    for index, line in enumerate(lines):
        heading = _markdown_heading(line)
        if heading is None:
            continue
        level, heading_title = heading
        if heading_title.casefold() == title.casefold():
            section_start = index + 1
            section_level = level
            break
    if section_start is None:
        return None

    section_end = len(lines)
    for index in range(section_start, len(lines)):
        heading = _markdown_heading(lines[index])
        if heading is not None and heading[0] <= section_level:
            section_end = index
            break
    return "\n".join(lines[section_start:section_end])


def _markdown_header(readme: str) -> str:
    """Return the README preamble before the first level-two section."""

    lines = readme.splitlines()
    for index, line in enumerate(lines):
        heading = _markdown_heading(line)
        if heading is not None and heading[0] == 2:
            return "\n".join(lines[:index])
    return readme


def _check_readme_versions(readme: str, crate_name: str, expected: str) -> list[str]:
    status_section = _markdown_section(readme, "Status")
    usage_section = _markdown_section(readme, "Usage")
    markers: tuple[tuple[str, re.Pattern[str]], ...] = (
        (
            "version badge",
            re.compile(rf"version-(?P<version>{SEMVER_BODY})-blue\.svg"),
        ),
        (
            "status",
            re.compile(rf"\*\*v(?P<version>{SEMVER_BODY}) (?:Stable|Pre-release)\.\*\*"),
        ),
        (
            "dependency examples",
            re.compile(
                rf"{re.escape(crate_name)}\s*=\s*(?:\{{\s*version\s*=\s*)?['\"](?P<version>{SEMVER_BODY})['\"]"
            ),
        ),
    )
    scopes = (_markdown_header(readme), status_section or "", usage_section or "")
    section_names = ("README header", "Status section", "Usage section")
    violations: list[str] = []
    for (label, pattern), scope, section_name in zip(markers, scopes, section_names):
        matches = [match.group("version") for match in pattern.finditer(scope)]
        if not matches:
            violations.append(f"README.md is missing the current {label} marker in the {section_name}")
            continue
        for marker_version in matches:
            if marker_version != expected:
                violations.append(
                    f"README.md {label} {marker_version!r} does not match authoritative version {expected!r}"
                )
    return violations


def _check_changelog_version(changelog: str, expected: str) -> list[str]:
    heading_pattern = re.compile(r"(?m)^##\s+\[([^\]]+)\]")
    for match in heading_pattern.finditer(changelog):
        label = match.group(1).strip()
        if label.casefold() == "unreleased":
            continue
        heading_version = _normalize_version(label)
        if heading_version is None:
            return [
                "CHANGELOG.md latest non-Unreleased heading "
                f"[{label}] is not a supported semantic version"
            ]
        if heading_version != expected:
            return [
                "CHANGELOG.md latest non-Unreleased heading "
                f"[{label}] does not match authoritative version {expected!r}"
            ]
        return []
    return ["CHANGELOG.md is missing a non-Unreleased version heading"]


def check_local_parity(root: Path = REPOSITORY_ROOT) -> list[str]:
    """Validate Cargo, README, and changelog release markers."""

    metadata, violations = load_release_metadata(root)
    if metadata is None:
        return violations

    cargo_lock, error = _read_file(root, "Cargo.lock")
    if error:
        violations.append(error)
    else:
        assert cargo_lock is not None
        root_packages = [
            package
            for package in _parse_lock_packages(cargo_lock)
            if package.get("name") == metadata.crate_name and "source" not in package
        ]
        if not root_packages:
            violations.append(
                f"Cargo.lock is missing the root package entry for {metadata.crate_name!r}"
            )
        elif len(root_packages) != 1:
            violations.append(
                f"Cargo.lock must contain exactly one root package entry for {metadata.crate_name!r}"
            )
        elif root_packages[0].get("version") != metadata.version:
            violations.append(
                "Cargo.lock root package version "
                f"{root_packages[0].get('version')!r} does not match authoritative version {metadata.version!r}"
            )

    readme, error = _read_file(root, "README.md")
    if error:
        violations.append(error)
    else:
        assert readme is not None
        violations.extend(_check_readme_versions(readme, metadata.crate_name, metadata.version))

    changelog, error = _read_file(root, "CHANGELOG.md")
    if error:
        violations.append(error)
    else:
        assert changelog is not None
        violations.extend(_check_changelog_version(changelog, metadata.version))

    return violations


def check_tag(tag: str, root: Path = REPOSITORY_ROOT) -> list[str]:
    """Require a release tag to be exactly ``v{Cargo.toml version}``."""

    metadata, violations = load_release_metadata(root)
    if metadata is None:
        return violations
    expected_tag = f"v{metadata.version}"
    if tag != expected_tag:
        violations.append(
            f"release tag {tag!r} does not match authoritative tag {expected_tag!r}"
        )
    return violations


def _response_status(response: object) -> int | None:
    """Read HTTP status metadata, returning ``None`` when it is unverifiable."""

    try:
        status = getattr(response, "status", None)
    except Exception:
        return None
    if status is not None:
        try:
            return int(status)
        except (TypeError, ValueError):
            return None

    try:
        getcode = getattr(response, "getcode", None)
    except Exception:
        return None
    if not callable(getcode):
        return None
    try:
        status = getcode()
        return int(status) if status is not None else None
    except Exception:
        return None


def _read_response(response: object) -> bytes:
    reader = getattr(response, "read", None)
    if not callable(reader):
        raise ValueError("remote response has no readable body")
    body = reader()
    return body if isinstance(body, bytes) else str(body).encode("utf-8")


def _close_response(response: object) -> None:
    closer = getattr(response, "close", None)
    if callable(closer):
        closer()


def _json_response(response: object) -> Mapping[str, object]:
    try:
        payload = json.loads(_read_response(response).decode("utf-8"))
    finally:
        _close_response(response)
    if not isinstance(payload, dict):
        raise ValueError("remote response JSON was not an object")
    return payload


def fetch_crates_io_state(
    crate_name: str,
    version: str,
    *,
    opener: UrlOpen = urlopen,
    timeout: float = 20.0,
) -> RemoteCheck:
    """Check one exact crates.io version without fail-open interpretation."""

    url = f"{CRATES_IO_API}/{quote(crate_name, safe='')}/{quote(version, safe='')}"
    request = Request(
        url,
        headers={"Accept": "application/json", "User-Agent": "lib-conxian-core-release-guard"},
    )
    try:
        response = opener(request, timeout=timeout)
        response_status = _response_status(response)
        if response_status is None:
            _close_response(response)
            return RemoteCheck(
                REMOTE_ERROR,
                "crates.io response omitted or had unverifiable HTTP status",
            )
        if response_status != 200:
            _close_response(response)
            if response_status == 404:
                return RemoteCheck(
                    REMOTE_MISSING,
                    f"crates.io returned confirmed HTTP 404 for {crate_name} {version}",
                )
            return RemoteCheck(
                REMOTE_ERROR,
                f"crates.io returned HTTP {response_status}; publication state is unknown",
            )
        payload = _json_response(response)
    except HTTPError as error:
        if error.code == 404:
            return RemoteCheck(
                REMOTE_MISSING,
                f"crates.io returned confirmed HTTP 404 for {crate_name} {version}",
            )
        return RemoteCheck(
            REMOTE_ERROR,
            f"crates.io returned HTTP {error.code}; publication state is unknown",
        )
    except (URLError, TimeoutError, OSError, ValueError, json.JSONDecodeError) as error:
        return RemoteCheck(
            REMOTE_ERROR,
            f"unable to confirm crates.io state: {error.__class__.__name__}",
        )

    crate = payload.get("crate")
    published_version = payload.get("version")
    if not isinstance(crate, dict) or not isinstance(published_version, dict):
        return RemoteCheck(REMOTE_ERROR, "crates.io response omitted crate/version identity")
    if crate.get("name") != crate_name or published_version.get("num") != version:
        return RemoteCheck(
            REMOTE_MISMATCH,
            "crates.io response identity did not match the expected crate/version",
        )
    return RemoteCheck(REMOTE_PRESENT, f"crates.io confirms {crate_name} {version}")


def wait_for_crates_io(
    crate_name: str,
    version: str,
    *,
    attempts: int = 12,
    delay_seconds: float = 10.0,
    opener: UrlOpen = urlopen,
    sleep: Callable[[float], None] = time.sleep,
) -> RemoteCheck:
    """Poll a bounded number of times for an exact crates.io version."""

    if attempts < 1:
        return RemoteCheck(REMOTE_ERROR, "crates.io polling attempts must be at least one")
    for attempt in range(1, attempts + 1):
        result = fetch_crates_io_state(crate_name, version, opener=opener)
        if result.state != REMOTE_MISSING:
            return result
        if attempt < attempts:
            sleep(delay_seconds)
    return RemoteCheck(
        REMOTE_ERROR,
        f"crates.io did not confirm {crate_name} {version} after {attempts} bounded checks",
    )


def fetch_github_release_state(
    repository: str,
    tag: str,
    token: str | None,
    *,
    opener: UrlOpen = urlopen,
    timeout: float = 20.0,
) -> RemoteCheck:
    """Check whether the exact GitHub release tag already has a release."""

    if not token:
        return RemoteCheck(REMOTE_ERROR, "GITHUB_TOKEN is required to validate GitHub Release state")
    if not repository or "/" not in repository:
        return RemoteCheck(REMOTE_ERROR, "GITHUB_REPOSITORY must be in owner/repository form")

    url = f"https://api.github.com/repos/{repository}/releases/tags/{quote(tag, safe='')}"
    request = Request(
        url,
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "User-Agent": "lib-conxian-core-release-guard",
            "X-GitHub-Api-Version": "2022-11-28",
        },
    )
    try:
        response = opener(request, timeout=timeout)
        response_status = _response_status(response)
        if response_status is None:
            _close_response(response)
            return RemoteCheck(
                REMOTE_ERROR,
                "GitHub response omitted or had unverifiable HTTP status",
            )
        if response_status != 200:
            _close_response(response)
            if response_status == 404:
                return RemoteCheck(REMOTE_MISSING, f"GitHub has no release for exact tag {tag}")
            return RemoteCheck(
                REMOTE_ERROR,
                f"GitHub returned HTTP {response_status}; release state is unknown",
            )
        payload = _json_response(response)
    except HTTPError as error:
        if error.code == 404:
            return RemoteCheck(REMOTE_MISSING, f"GitHub has no release for exact tag {tag}")
        return RemoteCheck(REMOTE_ERROR, f"GitHub returned HTTP {error.code}; release state is unknown")
    except (URLError, TimeoutError, OSError, ValueError, json.JSONDecodeError) as error:
        return RemoteCheck(
            REMOTE_ERROR,
            f"unable to confirm GitHub Release state: {error.__class__.__name__}",
        )

    if payload.get("tag_name") != tag:
        return RemoteCheck(REMOTE_MISMATCH, "GitHub Release tag identity did not match the requested tag")
    return RemoteCheck(REMOTE_PRESENT, f"GitHub confirms an existing release for exact tag {tag}")


def publication_decision(result: RemoteCheck) -> str:
    """Return the safe preflight action for a crates.io state result."""

    if result.state == REMOTE_PRESENT:
        return "skip-republish"
    if result.state == REMOTE_MISSING:
        return "publish"
    return "fail-closed"


def release_creation_allowed(
    *,
    local_parity_ok: bool,
    publication_confirmed: bool,
    github_release_state: RemoteCheck,
) -> bool:
    """Return whether creating a new GitHub Release is safe."""

    return (
        local_parity_ok
        and publication_confirmed
        and github_release_state.state == REMOTE_MISSING
    )


def _print_violations(violations: Iterable[str], workflow_path: Path | None = None) -> int:
    if violations:
        prefix = "Release hygiene verification failed"
        if workflow_path is not None:
            prefix += f" for {workflow_path}"
        print(f"{prefix}:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        return 1
    return 0


def _remote_exit(result: RemoteCheck, *, allow_missing: bool) -> int:
    if result.state == REMOTE_PRESENT:
        print(result.detail)
        return 0
    if result.state == REMOTE_MISSING and allow_missing:
        print(result.detail)
        return REMOTE_STATE_MISSING_EXIT
    print(result.detail, file=sys.stderr)
    return 1


def _metadata_for_remote(root: Path) -> tuple[ReleaseMetadata | None, int]:
    metadata, violations = load_release_metadata(root)
    if metadata is None:
        return None, _print_violations(violations)
    parity_violations = check_local_parity(root)
    if parity_violations:
        return None, _print_violations(parity_violations)
    return metadata, 0


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
    parser.add_argument(
        "--root",
        type=Path,
        default=REPOSITORY_ROOT,
        help="repository root containing Cargo.toml (default: script repository root)",
    )
    parser.add_argument("--tag", help="validate a release tag against Cargo.toml")
    remote_group = parser.add_mutually_exclusive_group()
    remote_group.add_argument(
        "--crates-io-state",
        action="store_true",
        help="check the exact authoritative crate/version; exit 10 only for confirmed HTTP 404",
    )
    remote_group.add_argument(
        "--wait-for-crates-io",
        action="store_true",
        help="poll crates.io for the exact authoritative crate/version",
    )
    remote_group.add_argument(
        "--github-release-state",
        action="store_true",
        help="check whether the exact GitHub tag already has a release",
    )
    parser.add_argument("--poll-attempts", type=int, default=12)
    parser.add_argument("--poll-delay-seconds", type=float, default=10.0)
    args = parser.parse_args(argv)
    workflow_path = args.workflow_option or args.workflow or DEFAULT_WORKFLOW

    if args.crates_io_state or args.wait_for_crates_io:
        metadata, status = _metadata_for_remote(args.root)
        if metadata is None:
            return status
        if args.wait_for_crates_io:
            result = wait_for_crates_io(
                metadata.crate_name,
                metadata.version,
                attempts=args.poll_attempts,
                delay_seconds=args.poll_delay_seconds,
            )
            return _remote_exit(result, allow_missing=False)
        result = fetch_crates_io_state(metadata.crate_name, metadata.version)
        return _remote_exit(result, allow_missing=True)

    if args.github_release_state:
        metadata, status = _metadata_for_remote(args.root)
        if metadata is None:
            return status
        if not args.tag:
            print("--github-release-state requires --tag", file=sys.stderr)
            return 1
        tag_violations = check_tag(args.tag, args.root)
        if tag_violations:
            return _print_violations(tag_violations)
        repository = os.environ.get("GITHUB_REPOSITORY", "")
        token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
        result = fetch_github_release_state(repository, args.tag, token)
        return _remote_exit(result, allow_missing=True)

    violations = verify(workflow_path, args.main_workflow)
    violations.extend(check_local_parity(args.root))
    if args.tag:
        violations.extend(check_tag(args.tag, args.root))
    if workflow_path.resolve() == DEFAULT_WORKFLOW.resolve():
        try:
            ci_workflow = DEFAULT_CI_WORKFLOW.read_text(encoding="utf-8")
        except OSError as error:
            violations.append(f"Unable to read normal CI workflow {DEFAULT_CI_WORKFLOW}: {error}")
        else:
            if not re.search(r"python(?:3)?\s+scripts/verify_release_hygiene\.py\b", ci_workflow):
                violations.append(
                    "normal CI workflow must run scripts/verify_release_hygiene.py for local parity"
                )

    status = _print_violations(violations, workflow_path)
    if status:
        return status

    print(f"Release hygiene verification passed for {workflow_path}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
