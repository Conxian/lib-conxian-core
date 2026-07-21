#!/usr/bin/env python3
"""Verify source and remote release-version parity without third-party packages.

The source-only phase is intentionally local: an unreleased version on ``main``
is valid. Release phases are fail-closed and require a valid SemVer 2.0.0
``vX.Y.Z[-prerelease][+build]`` tag, an existing GitHub tag pointing at the
checked-out source, and the expected crates.io/GitHub Release state for the
selected lifecycle phase.

Cargo accepts SemVer prerelease and build metadata in package versions. This
guard therefore accepts both forms and compares the complete version string
exactly across Cargo, README, changelog, tags, and registry responses. Cargo
ignores build metadata when resolving dependency requirements, so build
metadata is informational here rather than a compatibility boundary.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Protocol
from urllib.error import HTTPError, URLError
from urllib.parse import quote
from urllib.request import Request, urlopen


PHASES = ("source-only", "pre-publish", "post-publish", "post-release")
SEMVER_NUMERIC_IDENTIFIER = r"(?:0|[1-9][0-9]*)"
SEMVER_PRERELEASE_IDENTIFIER = (
    r"(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)"
)
SEMVER_BUILD_IDENTIFIER = r"[0-9A-Za-z-]+"
SEMVER_CORE = (
    rf"{SEMVER_NUMERIC_IDENTIFIER}\.{SEMVER_NUMERIC_IDENTIFIER}\."
    rf"{SEMVER_NUMERIC_IDENTIFIER}"
)
SEMVER_PRERELEASE = (
    rf"(?:{SEMVER_PRERELEASE_IDENTIFIER})"
    rf"(?:\.(?:{SEMVER_PRERELEASE_IDENTIFIER}))*"
)
SEMVER_BUILD = rf"(?:{SEMVER_BUILD_IDENTIFIER})(?:\.(?:{SEMVER_BUILD_IDENTIFIER}))*"
SEMVER_TEXT = rf"{SEMVER_CORE}(?:-{SEMVER_PRERELEASE})?(?:\+{SEMVER_BUILD})?"
SEMVER_PATTERN = re.compile(rf"\A(?P<version>{SEMVER_TEXT})\Z")
TAG_PATTERN = re.compile(rf"\Av(?P<version>{SEMVER_TEXT})\Z")
PACKAGE_FIELD_PATTERN = re.compile(
    r"\A(?P<field>name|version)\s*=\s*[\"'](?P<value>[^\"']+)[\"']\s*(?:#.*)?\Z"
)
LOCK_FIELD_PATTERN = re.compile(
    r"\A(?P<field>name|version|source)\s*=\s*[\"'](?P<value>[^\"']+)[\"']\s*(?:#.*)?\Z"
)
README_BADGE_PATTERN = re.compile(
    rf"\[!\[Version\]\([^\n]*?version-(?P<version>{SEMVER_TEXT})-blue\.svg\)"
)
README_STATUS_PATTERN = re.compile(
    rf"(?m)^\*\*v(?P<version>{SEMVER_TEXT}) "
    r"(?:Stable|Pre-release)\.\*\*"
)
README_DEPENDENCY_PATTERN = re.compile(
    rf"(?m)^\s*lib-conxian-core\s*=\s*(?:\"(?P<simple>{SEMVER_TEXT})\"|"
    rf"\{{[^\n]*?version\s*=\s*\"(?P<table>{SEMVER_TEXT})\"[^\n]*\}})\s*$"
)
CHANGELOG_PATTERN_TEMPLATE = r"(?m)^## \[v{version}\](?:\s+-\s+[0-9]{{4}}-[0-9]{{2}}-[0-9]{{2}})?\s*$"
SHA_PATTERN = re.compile(r"\A[0-9a-fA-F]{7,64}\Z")
DEFAULT_GITHUB_API_URL = "https://api.github.com"
MAX_TAG_DEREFERENCES = 8


class RemoteCheckError(RuntimeError):
    """A remote release check failed and must stop the release flow."""


@dataclass(frozen=True)
class SourceMetadata:
    """Authoritative package metadata loaded from the repository."""

    package_name: str
    version: str


@dataclass(frozen=True)
class RegistryVersion:
    """The exact crates.io version returned by the registry API."""

    number: str


class RegistryLookup(Protocol):
    """Minimal registry client contract used by the phase verifier."""

    def get_version(self, package_name: str, version: str) -> RegistryVersion | None:
        """Return the exact published version, or ``None`` when absent."""


class GitHubLookup(Protocol):
    """Minimal GitHub client contract used by the phase verifier."""

    def get_tag_target(self, tag: str) -> str | None:
        """Return the commit targeted by a tag, or ``None`` when absent."""

    def get_release(self, tag: str) -> dict[str, Any] | None:
        """Return the release for a tag, or ``None`` when absent."""


def is_valid_semver(version: str) -> bool:
    """Return whether ``version`` is a complete SemVer 2.0.0 string."""

    return SEMVER_PATTERN.fullmatch(version) is not None


def is_valid_release_tag(tag: str) -> bool:
    """Return whether ``tag`` is ``v`` followed by a complete SemVer string."""

    return TAG_PATTERN.fullmatch(tag) is not None


def _read_text(path: Path, label: str, errors: list[str]) -> str | None:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as error:
        errors.append(f"Unable to read {label} ({path}): {error}")
        return None


def parse_package_manifest(text: str) -> SourceMetadata:
    """Parse the root ``[package]`` name and version from Cargo.toml."""

    in_package = False
    values: dict[str, str] = {}
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            in_package = line == "[package]"
            continue
        if not in_package:
            continue
        match = PACKAGE_FIELD_PATTERN.match(line)
        if match:
            values[match.group("field")] = match.group("value")

    missing = [field for field in ("name", "version") if field not in values]
    if missing:
        raise ValueError(f"Cargo.toml [package] is missing: {', '.join(missing)}")

    package_name = values["name"]
    version = values["version"]
    if not package_name:
        raise ValueError("Cargo.toml [package].name must not be empty")
    if not is_valid_semver(version):
        raise ValueError(
            "Cargo.toml [package].version must be valid SemVer "
            f"X.Y.Z[-prerelease][+build], got {version!r}"
        )
    return SourceMetadata(package_name=package_name, version=version)


def parse_root_lock_version(text: str, package_name: str) -> str:
    """Return the version of the root package entry in Cargo.lock."""

    blocks = re.split(r"(?m)^\[\[package\]\]\s*$", text)[1:]
    matches: list[tuple[str, bool]] = []
    for block in blocks:
        values: dict[str, str] = {}
        for raw_line in block.splitlines():
            match = LOCK_FIELD_PATTERN.match(raw_line.strip())
            if match:
                values[match.group("field")] = match.group("value")
        if values.get("name") == package_name and "version" in values:
            matches.append((values["version"], "source" in values))

    root_matches = [version for version, has_source in matches if not has_source]
    if len(root_matches) != 1:
        if not matches:
            raise ValueError(
                f"Cargo.lock has no root package entry for {package_name!r}"
            )
        raise ValueError(
            f"Cargo.lock must contain exactly one source-less root entry for {package_name!r}"
        )
    version = root_matches[0]
    if not is_valid_semver(version):
        raise ValueError(
            "Cargo.lock root package version must be valid SemVer "
            f"X.Y.Z[-prerelease][+build], got {version!r}"
        )
    return version


def parse_readme_markers(text: str) -> dict[str, list[str]]:
    """Extract only the structured current-version markers from README.md."""

    dependencies: list[str] = []
    for match in README_DEPENDENCY_PATTERN.finditer(text):
        dependencies.append(match.group("simple") or match.group("table"))
    return {
        "badge": [match.group("version") for match in README_BADGE_PATTERN.finditer(text)],
        "status": [match.group("version") for match in README_STATUS_PATTERN.finditer(text)],
        "dependencies": dependencies,
    }


def has_changelog_section(text: str, version: str) -> bool:
    """Return whether CHANGELOG.md has the exact release heading for ``version``."""

    pattern = re.compile(CHANGELOG_PATTERN_TEMPLATE.format(version=re.escape(version)))
    return pattern.search(text) is not None


def check_source_parity(root: Path) -> tuple[SourceMetadata | None, list[str]]:
    """Validate local source markers and return the authoritative package metadata."""

    errors: list[str] = []
    manifest_text = _read_text(root / "Cargo.toml", "Cargo.toml", errors)
    if manifest_text is None:
        return None, errors

    try:
        metadata = parse_package_manifest(manifest_text)
    except ValueError as error:
        errors.append(str(error))
        return None, errors

    lock_text = _read_text(root / "Cargo.lock", "Cargo.lock", errors)
    if lock_text is not None:
        try:
            lock_version = parse_root_lock_version(lock_text, metadata.package_name)
        except ValueError as error:
            errors.append(str(error))
        else:
            if lock_version != metadata.version:
                errors.append(
                    "Cargo.lock root package version does not match Cargo.toml: "
                    f"{lock_version!r} != {metadata.version!r}"
                )

    readme_text = _read_text(root / "README.md", "README.md", errors)
    if readme_text is not None:
        markers = parse_readme_markers(readme_text)
        for marker_name in ("badge", "status"):
            values = markers[marker_name]
            if len(values) != 1:
                errors.append(
                    f"README.md must contain exactly one structured {marker_name} version marker; "
                    f"found {len(values)}"
                )
            elif values[0] != metadata.version:
                errors.append(
                    f"README.md {marker_name} marker does not match Cargo.toml: "
                    f"{values[0]!r} != {metadata.version!r}"
                )

        dependencies = markers["dependencies"]
        if len(dependencies) != 2:
            errors.append(
                "README.md must contain exactly two structured lib-conxian-core dependency examples; "
                f"found {len(dependencies)}"
            )
        for dependency_version in dependencies:
            if dependency_version != metadata.version:
                errors.append(
                    "README.md dependency example does not match Cargo.toml: "
                    f"{dependency_version!r} != {metadata.version!r}"
                )

    changelog_text = _read_text(root / "CHANGELOG.md", "CHANGELOG.md", errors)
    if changelog_text is not None and not has_changelog_section(
        changelog_text, metadata.version
    ):
        errors.append(
            f"CHANGELOG.md is missing the exact release section `## [v{metadata.version}]`"
        )

    return metadata, errors


def _validate_repository(repository: str | None) -> str:
    if not repository or not re.fullmatch(r"[^/\s]+/[^/\s]+", repository):
        raise ValueError(
            "release phases require --repository OWNER/REPO or GITHUB_REPOSITORY"
        )
    return repository


def _validate_tag(tag: str | None, version: str) -> list[str]:
    errors: list[str] = []
    expected_tag = f"v{version}"
    if not tag:
        errors.append(f"release phases require an explicit tag matching {expected_tag!r}")
        return errors
    match = TAG_PATTERN.fullmatch(tag)
    if match is None:
        errors.append(
            "release tag must match valid SemVer "
            f"vX.Y.Z[-prerelease][+build] form; got {tag!r}"
        )
    elif match.group("version") != version:
        errors.append(
            f"release tag does not match Cargo.toml: {tag!r} != {expected_tag!r}"
        )
    return errors


def _validate_source_revision(source_revision: str | None) -> list[str]:
    if source_revision is None:
        return [
            "release phases require --source-revision so the GitHub tag can be checked against the checked-out source"
        ]
    if not SHA_PATTERN.fullmatch(source_revision):
        return [f"source revision must be a hexadecimal commit SHA; got {source_revision!r}"]
    return []


def _release_is_matching(release: dict[str, Any], tag: str) -> list[str]:
    errors: list[str] = []
    release_tag = release.get("tag_name")
    if release_tag != tag:
        errors.append(
            f"GitHub Release tag_name does not match requested tag: {release_tag!r} != {tag!r}"
        )
    return errors


def wait_for_registry_version(
    registry: RegistryLookup,
    package_name: str,
    version: str,
    *,
    attempts: int,
    delay_seconds: float,
    sleep_fn: Callable[[float], None] = time.sleep,
) -> RegistryVersion:
    """Poll crates.io with bounded retries for post-publication propagation."""

    if attempts < 1:
        raise ValueError("registry retry attempts must be at least 1")
    if delay_seconds < 0:
        raise ValueError("registry retry delay must not be negative")

    last_error: str | None = None
    for attempt in range(1, attempts + 1):
        try:
            published = registry.get_version(package_name, version)
        except RemoteCheckError as error:
            last_error = str(error)
        else:
            if published is not None and published.number == version:
                return published
            last_error = f"version {version} is not visible yet"

        if attempt < attempts:
            sleep_fn(delay_seconds)

    detail = f" Last error: {last_error}." if last_error else ""
    raise RemoteCheckError(
        f"crates.io did not expose {package_name} {version} after {attempts} attempt(s)."
        f"{detail} If cargo publish succeeded, do not republish; use workflow mode"
        " `release-only` after the registry is confirmed."
    )


class CratesIoClient:
    """Small stdlib-only client for the public crates.io version endpoint."""

    def __init__(
        self,
        *,
        opener: Callable[..., Any] = urlopen,
        base_url: str = "https://crates.io/api/v1",
        timeout: float = 15.0,
    ) -> None:
        self._opener = opener
        self._base_url = base_url.rstrip("/")
        self._timeout = timeout

    def get_version(self, package_name: str, version: str) -> RegistryVersion | None:
        url = f"{self._base_url}/crates/{quote(package_name, safe='')}/{quote(version, safe='')}"
        request = Request(
            url,
            headers={
                "Accept": "application/json",
                "User-Agent": "lib-conxian-core-release-guard/1.0",
            },
        )
        try:
            with self._opener(request, timeout=self._timeout) as response:
                payload = json.load(response)
        except HTTPError as error:
            if error.code == 404:
                return None
            raise RemoteCheckError(f"crates.io returned HTTP {error.code} for {url}") from error
        except (URLError, OSError, ValueError) as error:
            raise RemoteCheckError(f"unable to query crates.io at {url}: {error}") from error

        version_payload = payload.get("version") if isinstance(payload, dict) else None
        number = version_payload.get("num") if isinstance(version_payload, dict) else None
        if not isinstance(number, str):
            raise RemoteCheckError(f"crates.io response for {url} lacked version.num")
        return RegistryVersion(number=number)


class GitHubClient:
    """Small stdlib-only client for public GitHub tag and release endpoints."""

    def __init__(
        self,
        repository: str,
        *,
        token: str | None = None,
        opener: Callable[..., Any] = urlopen,
        base_url: str | None = None,
        timeout: float = 15.0,
        max_tag_dereferences: int = MAX_TAG_DEREFERENCES,
    ) -> None:
        if max_tag_dereferences < 1:
            raise ValueError("max_tag_dereferences must be at least 1")
        self._repository = _validate_repository(repository)
        self._token = token
        self._opener = opener
        self._base_url = (
            base_url or os.environ.get("GITHUB_API_URL") or DEFAULT_GITHUB_API_URL
        ).rstrip("/")
        self._timeout = timeout
        self._max_tag_dereferences = max_tag_dereferences

    def _get_json(self, path: str, *, allow_not_found: bool = False) -> dict[str, Any] | None:
        url = f"{self._base_url}{path}"
        headers = {
            "Accept": "application/vnd.github+json",
            "User-Agent": "lib-conxian-core-release-guard/1.0",
            "X-GitHub-Api-Version": "2022-11-28",
        }
        if self._token:
            headers["Authorization"] = f"Bearer {self._token}"
        request = Request(url, headers=headers)
        try:
            with self._opener(request, timeout=self._timeout) as response:
                payload = json.load(response)
        except HTTPError as error:
            if allow_not_found and error.code == 404:
                return None
            raise RemoteCheckError(f"GitHub returned HTTP {error.code} for {url}") from error
        except (URLError, OSError, ValueError) as error:
            raise RemoteCheckError(f"unable to query GitHub at {url}: {error}") from error
        if not isinstance(payload, dict):
            raise RemoteCheckError(f"GitHub response for {url} was not an object")
        return payload

    def _resolve_tag_object(
        self,
        tag: str,
        object_type: Any,
        object_sha: Any,
        *,
        seen: set[str],
        depth: int,
    ) -> str:
        if not isinstance(object_sha, str) or not SHA_PATTERN.fullmatch(object_sha):
            raise RemoteCheckError(
                f"GitHub tag {tag!r} response lacked a valid object SHA"
            )
        if object_type == "commit":
            return object_sha
        if object_type != "tag":
            raise RemoteCheckError(
                f"GitHub tag {tag!r} resolved to unsupported object type {object_type!r}"
            )
        if depth >= self._max_tag_dereferences:
            raise RemoteCheckError(
                f"GitHub annotated tag {tag!r} exceeded the "
                f"{self._max_tag_dereferences}-object dereference limit"
            )
        if object_sha in seen:
            raise RemoteCheckError(
                f"GitHub annotated tag {tag!r} contains a cyclic tag-object chain"
            )
        seen.add(object_sha)

        tag_payload = self._get_json(
            f"/repos/{self._repository}/git/tags/{quote(object_sha, safe='')}"
        )
        target = tag_payload.get("object") if tag_payload else None
        if not isinstance(target, dict):
            raise RemoteCheckError(
                f"GitHub annotated tag {tag!r} response lacked a target object"
            )
        return self._resolve_tag_object(
            tag,
            target.get("type"),
            target.get("sha"),
            seen=seen,
            depth=depth + 1,
        )

    def get_tag_target(self, tag: str) -> str | None:
        encoded_tag = quote(tag, safe="")
        ref_path = f"/repos/{self._repository}/git/ref/tags/{encoded_tag}"
        ref_payload = self._get_json(ref_path, allow_not_found=True)
        if ref_payload is None:
            return None
        ref_name = ref_payload.get("ref")
        if ref_name != f"refs/tags/{tag}":
            raise RemoteCheckError(
                f"GitHub returned unexpected tag ref {ref_name!r} for requested {tag!r}"
            )
        tag_object = ref_payload.get("object")
        if not isinstance(tag_object, dict):
            raise RemoteCheckError(f"GitHub tag {tag!r} response lacked an object")
        return self._resolve_tag_object(
            tag,
            tag_object.get("type"),
            tag_object.get("sha"),
            seen=set(),
            depth=0,
        )

    def get_release(self, tag: str) -> dict[str, Any] | None:
        encoded_tag = quote(tag, safe="")
        path = f"/repos/{self._repository}/releases/tags/{encoded_tag}"
        release = self._get_json(path, allow_not_found=True)
        if release is not None:
            errors = _release_is_matching(release, tag)
            if errors:
                raise RemoteCheckError("; ".join(errors))
        return release


def verify_phase(
    root: Path,
    phase: str,
    *,
    tag: str | None = None,
    repository: str | None = None,
    source_revision: str | None = None,
    registry: RegistryLookup | None = None,
    github: GitHubLookup | None = None,
    registry_attempts: int = 12,
    registry_delay_seconds: float = 10.0,
    sleep_fn: Callable[[float], None] = time.sleep,
) -> list[str]:
    """Return all parity violations for one release lifecycle phase."""

    if phase not in PHASES:
        return [f"unknown release verification phase {phase!r}; expected one of {PHASES}"]

    metadata, errors = check_source_parity(root)
    if metadata is None or errors:
        return errors
    if phase == "source-only":
        return []

    errors.extend(_validate_tag(tag, metadata.version))
    errors.extend(_validate_source_revision(source_revision))
    try:
        repository_name = _validate_repository(repository)
    except ValueError as error:
        errors.append(str(error))
        repository_name = ""
    if errors:
        return errors

    registry_client = registry or CratesIoClient()
    github_client = github or GitHubClient(
        repository_name,
        token=os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN"),
    )
    assert tag is not None

    try:
        tag_target = github_client.get_tag_target(tag)
    except RemoteCheckError as error:
        errors.append(f"GitHub tag lookup failed for {tag!r}: {error}")
        return errors
    if tag_target is None:
        errors.append(
            f"GitHub tag {tag!r} does not exist in {repository_name}; manual release flows must use an existing tag"
        )
    elif source_revision is not None and tag_target.lower() != source_revision.lower():
        errors.append(
            f"GitHub tag {tag!r} points at {tag_target}, but checked-out source is {source_revision}"
        )
    if errors:
        return errors

    if phase == "pre-publish":
        try:
            published = registry_client.get_version(metadata.package_name, metadata.version)
        except RemoteCheckError as error:
            errors.append(str(error))
        else:
            if published is not None:
                errors.append(
                    f"crates.io already contains {metadata.package_name} {metadata.version}; "
                    "use post-publish/recovery or release-only instead of republishing"
                )

        try:
            release = github_client.get_release(tag)
        except RemoteCheckError as error:
            errors.append(str(error))
        else:
            if release is not None:
                errors.append(
                    f"GitHub Release {tag!r} already exists; use release-only recovery instead of publishing"
                )
        return errors

    try:
        wait_for_registry_version(
            registry_client,
            metadata.package_name,
            metadata.version,
            attempts=registry_attempts,
            delay_seconds=registry_delay_seconds,
            sleep_fn=sleep_fn,
        )
    except (RemoteCheckError, ValueError) as error:
        errors.append(str(error))
        return errors

    try:
        release = github_client.get_release(tag)
    except RemoteCheckError as error:
        errors.append(str(error))
        return errors

    if release is not None:
        errors.extend(_release_is_matching(release, tag))

    if phase == "post-release":
        if release is None:
            errors.append(
                f"GitHub Release {tag!r} is missing; create it before post-release verification"
            )
        elif release.get("draft") is True:
            errors.append(f"GitHub Release {tag!r} exists but is still a draft")
    return errors


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--phase", choices=PHASES, required=True)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="repository root (default: the parent of scripts/)",
    )
    parser.add_argument("--tag", help="release tag, required for release phases")
    parser.add_argument(
        "--repository",
        default=os.environ.get("GITHUB_REPOSITORY"),
        help="GitHub OWNER/REPO (default: GITHUB_REPOSITORY)",
    )
    parser.add_argument(
        "--source-revision",
        help="checked-out commit SHA to compare with the GitHub tag",
    )
    parser.add_argument(
        "--registry-attempts",
        type=int,
        default=12,
        help="bounded crates.io polling attempts for post-publication phases",
    )
    parser.add_argument(
        "--registry-delay-seconds",
        type=float,
        default=10.0,
        help="delay between crates.io polling attempts",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    violations = verify_phase(
        args.root,
        args.phase,
        tag=args.tag,
        repository=args.repository,
        source_revision=args.source_revision,
        registry_attempts=args.registry_attempts,
        registry_delay_seconds=args.registry_delay_seconds,
    )
    if violations:
        print(
            f"Release version verification failed ({args.phase}) for {args.root}:",
            file=sys.stderr,
        )
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        return 1

    print(f"Release version verification passed ({args.phase}) for {args.root}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
