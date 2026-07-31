#!/usr/bin/env python3
"""Reject prohibited transport, persistence, and legacy TLS in Core's closure."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import deque
from pathlib import Path
from typing import Any


CORE_PACKAGE = "lib-conxian-core"


class BoundaryError(ValueError):
    """Raised when metadata violates or cannot prove the Core boundary."""


def version_series(version: str) -> tuple[int, int]:
    """Return the numeric major/minor pair from a Cargo package version."""
    try:
        major, minor, _ = version.split(".", 2)
        return int(major), int(minor)
    except (TypeError, ValueError) as error:
        raise BoundaryError(f"invalid package version: {version!r}") from error


def package_label(package: dict[str, Any]) -> str:
    return f"{package['name']} {package['version']}"


def find_core_root(metadata: dict[str, Any]) -> str:
    packages = {package["id"]: package for package in metadata.get("packages", [])}
    candidates = [
        package_id
        for package_id in metadata.get("workspace_members", [])
        if packages.get(package_id, {}).get("name") == CORE_PACKAGE
    ]
    if len(candidates) != 1:
        raise BoundaryError(
            f"expected exactly one {CORE_PACKAGE!r} workspace member, found {len(candidates)}"
        )
    return candidates[0]


def transitive_closure(metadata: dict[str, Any], root_id: str) -> set[str]:
    resolve = metadata.get("resolve")
    if not resolve:
        raise BoundaryError("cargo metadata did not include a resolved dependency graph")

    dependencies = {
        node["id"]: node.get("dependencies", []) for node in resolve.get("nodes", [])
    }
    if root_id not in dependencies:
        raise BoundaryError(f"resolved dependency graph has no node for {root_id}")

    closure: set[str] = set()
    queue = deque([root_id])
    while queue:
        package_id = queue.popleft()
        if package_id in closure:
            continue
        if package_id not in dependencies:
            raise BoundaryError(f"dependency graph references missing node {package_id}")
        closure.add(package_id)
        queue.extend(dependencies[package_id])
    return closure


def prohibited_reason(package: dict[str, Any]) -> str | None:
    name = package["name"]
    series = version_series(package["version"])
    if name == "bdk":
        return "unused wallet implementation"
    if name == "electrum-client":
        return "network transport implementation"
    if name == "sled":
        return "persistence implementation"
    if name == "rustls" and series == (0, 21):
        return "legacy TLS implementation"
    if name == "rustls-webpki" and series == (0, 101):
        return "legacy TLS certificate implementation"
    return None


def validate_metadata(metadata: dict[str, Any]) -> list[str]:
    """Return deterministic violations from the locked Core package closure."""
    packages = {package["id"]: package for package in metadata.get("packages", [])}
    root_id = find_core_root(metadata)
    closure = transitive_closure(metadata, root_id)

    violations = []
    for package_id in sorted(closure):
        package = packages.get(package_id)
        if package is None:
            raise BoundaryError(f"dependency graph references unknown package {package_id}")
        reason = prohibited_reason(package)
        if reason:
            violations.append(f"{package_label(package)}: prohibited {reason}")
    return violations


def load_metadata(path: Path | None) -> dict[str, Any]:
    if path is not None:
        with path.open(encoding="utf-8") as metadata_file:
            return json.load(metadata_file)

    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return json.loads(result.stdout)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--metadata",
        type=Path,
        help="validate saved Cargo metadata instead of invoking Cargo",
    )
    args = parser.parse_args()

    try:
        violations = validate_metadata(load_metadata(args.metadata))
    except (BoundaryError, json.JSONDecodeError, OSError, subprocess.CalledProcessError) as error:
        print(f"Core dependency boundary guard could not validate the graph: {error}", file=sys.stderr)
        return 2

    if violations:
        print("Core dependency boundary guard failed:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        return 1

    print(
        "Core dependency boundary guard passed: the locked Core closure has no BDK, "
        "Electrum, sled, rustls 0.21.x, or rustls-webpki 0.101.x packages."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
