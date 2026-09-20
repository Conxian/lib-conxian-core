#!/usr/bin/env python3
"""Verify Docker Compose configurations and environment template files for secret exposure.

Scans compose files (*compose*.yml, *compose*.yaml) and environment template
files (.env*, *.env*) to ensure no live secrets or high-entropy raw private
keys are hardcoded or tracked.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# High-entropy private key patterns
RAW_HEX_KEY_RE = re.compile(r"\b[0-9a-fA-F]{64}\b")
PEM_PRIVATE_KEY_RE = re.compile(r"-----BEGIN (?:RSA|EC|OPENSSH|DSA|PRIVATE)? KEY-----")

# Whitelisted / safe template placeholders
ALLOWED_PLACEHOLDERS = {
    "",
    "0",
    "1",
    "true",
    "false",
    "change_me",
    "changeme",
    "your_api_key_here",
    "your_secret_here",
    "your_private_key_here",
    "placeholder",
    "localhost",
    "127.0.0.1",
}


def is_placeholder(value: str) -> bool:
    val = value.strip().strip("'\"").lower()
    if val in ALLOWED_PLACEHOLDERS:
        return True
    if val.startswith("your_") or val.endswith("_here") or "placeholder" in val or "example" in val:
        return True
    if val.startswith("${") and val.endswith("}"):
        return True
    return False


def scan_file(filepath: Path) -> list[str]:
    violations: list[str] = []
    rel_path = filepath.relative_to(ROOT)

    try:
        text = filepath.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return violations

    for line_num, line in enumerate(text.splitlines(), start=1):
        clean_line = line.strip()
        if not clean_line or clean_line.startswith("#"):
            continue

        if PEM_PRIVATE_KEY_RE.search(clean_line):
            violations.append(f"{rel_path}:{line_num} contains PEM private key header")
            continue

        if RAW_HEX_KEY_RE.search(clean_line) and not is_placeholder(clean_line):
            violations.append(f"{rel_path}:{line_num} contains raw 64-character hex key string")
            continue

        if "=" in clean_line:
            key, _, val = clean_line.partition("=")
            key = key.strip().upper()
            val = val.strip()

            if any(term in key for term in ("SECRET", "KEY", "TOKEN", "PASSWORD", "PRIVATE")):
                if val and not is_placeholder(val):
                    # Flag potential hardcoded secret values that are not placeholders
                    if len(val) >= 16 and not val.startswith("http"):
                        violations.append(f"{rel_path}:{line_num} hardcoded potential secret for key '{key}'")

    return violations


def find_target_files() -> list[Path]:
    target_files: list[Path] = []
    for p in ROOT.rglob("*"):
        if not p.is_file():
            continue
        # Skip git directory, target, and node_modules
        if any(part in p.parts for part in (".git", "target", "node_modules", ".pytest_cache")):
            continue

        name = p.name.lower()
        if "compose" in name and (name.endswith(".yml") or name.endswith(".yaml")):
            target_files.append(p)
        elif name.startswith(".env") or name.endswith(".env") or ".env." in name:
            target_files.append(p)

    return sorted(target_files)


def main() -> int:
    print("Verifying compose files and environment templates...")
    target_files = find_target_files()
    violations: list[str] = []

    for file_path in target_files:
        violations.extend(scan_file(file_path))

    if violations:
        print("Error: Hardcoded secret or key violations detected:")
        for v in violations:
            print(f"  - {v}")
        return 1

    print(f"Verified {len(target_files)} compose/environment template files. No hardcoded secrets found.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
