#!/usr/bin/env python3
"""Verify that no tracked files or submodules contain secret or key filenames.

Enforces zero sensitive key/secret exposure across tracked repository files
and git submodules.
"""

from __future__ import annotations

import fnmatch
import subprocess
import sys
from pathlib import Path

# Secret patterns that must never be tracked in git
FORBIDDEN_SECRET_PATTERNS = [
    ".env",
    "*.env",
    "*.pem",
    "*.key",
    "id_rsa*",
    "id_ed25519*",
    "*.pfx",
    "*.p12",
    "*.jks",
    "*.keystore",
    "credentials.json",
    "*.pub",
]

# Safe exemption patterns (templates / examples)
EXEMPTION_PATTERNS = [
    "*.example",
    "*.template",
    "*.sample",
    "*.dist",
]


def is_exempt(filename: str) -> bool:
    name = Path(filename).name.lower()
    return any(fnmatch.fnmatch(name, pat) for pat in EXEMPTION_PATTERNS)


def check_git_files(git_dir: str | None = None) -> list[tuple[str, str]]:
    violations: list[tuple[str, str]] = []
    cmd = ["git"]
    if git_dir:
        cmd.extend(["-C", git_dir])
    cmd.extend(["ls-files"])

    try:
        res = subprocess.run(cmd, capture_output=True, text=True, check=True)
        tracked_files = [line.strip() for line in res.stdout.splitlines() if line.strip()]
    except (subprocess.CalledProcessError, FileNotFoundError):
        return violations

    for filepath in tracked_files:
        filename = Path(filepath).name
        for pattern in FORBIDDEN_SECRET_PATTERNS:
            if fnmatch.fnmatch(filename, pattern) or fnmatch.fnmatch(filepath, pattern):
                if not is_exempt(filename):
                    scope = "submodule" if git_dir else "repository"
                    violations.append((scope, pattern))
                break

    return violations


def main() -> int:
    print("Verifying submodule and repository secret filenames...")
    violations: list[tuple[str, str]] = []

    # Check main repository tracked files
    violations.extend(check_git_files())

    # Check submodules if any exist
    try:
        sub_res = subprocess.run(
            ["git", "submodule", "status"], capture_output=True, text=True, check=True
        )
        for line in sub_res.stdout.splitlines():
            line = line.strip()
            if not line:
                continue
            parts = line.split()
            if len(parts) >= 2:
                submodule_path = parts[1]
                violations.extend(check_git_files(git_dir=submodule_path))
    except (subprocess.CalledProcessError, FileNotFoundError):
        pass

    if violations:
        print("Error: Forbidden secret filenames found in tracked git repository/submodules.")
        print(f"Total violations: {len(violations)}")
        pattern_counts: dict[str, int] = {}
        for _, pattern in violations:
            pattern_counts[pattern] = pattern_counts.get(pattern, 0) + 1
        print("Matched forbidden patterns:")
        for pattern, count in sorted(pattern_counts.items()):
            print(f"  - {pattern}: {count}")
        return 1

    print("No forbidden secret filenames detected in tracked files or submodules.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
