#!/usr/bin/env python3
"""Verify Business Operation System (BOS) production boundary in Core library.

Enforces strict separation between core protocol primitives (lib-conxian-core)
and application-level Business Operation System (BOS) logic, database drivers,
or user management side-effects.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = ROOT / "src"

# Forbidden patterns in core production source code (excluding tests)
FORBIDDEN_BOS_PATTERNS = [
    (re.compile(r"\bpostgres\b", re.IGNORECASE), "Direct PostgreSQL database reference"),
    (re.compile(r"\bmysql\b", re.IGNORECASE), "Direct MySQL database reference"),
    (re.compile(r"\bsqlx\b", re.IGNORECASE), "Direct SQLx ORM reference"),
    (re.compile(r"\bdiesel::\b", re.IGNORECASE), "Direct Diesel ORM reference"),
    (re.compile(r"\bstripe\b", re.IGNORECASE), "Direct Stripe billing reference"),
    (re.compile(r"\bauth0\b", re.IGNORECASE), "Direct Auth0 identity reference"),
]


def is_test_file(path: Path) -> bool:
    name = path.name.lower()
    return name == "tests.rs" or name.endswith("_test.rs") or "test" in name


def scan_src_file(path: Path) -> list[str]:
    violations: list[str] = []
    if is_test_file(path):
        return violations

    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return violations

    in_test_module = False
    for line_num, line in enumerate(text.splitlines(), start=1):
        clean_line = line.strip()

        # Simple test module tracking
        if "#[cfg(test)]" in clean_line:
            in_test_module = True
            continue

        if in_test_module:
            continue

        # Skip comments
        if clean_line.startswith("//") or clean_line.startswith("/*") or clean_line.startswith("*"):
            continue

        for pattern, description in FORBIDDEN_BOS_PATTERNS:
            if pattern.search(clean_line):
                rel_path = path.relative_to(ROOT)
                violations.append(f"{rel_path}:{line_num} {description}: '{clean_line}'")

    return violations


def main() -> int:
    print("Verifying BOS production boundary in Core library (src/)...")
    if not SRC_DIR.is_dir():
        print("Error: src/ directory not found.")
        return 1

    violations: list[str] = []
    for rs_file in SRC_DIR.rglob("*.rs"):
        violations.extend(scan_src_file(rs_file))

    if violations:
        print("Error: BOS boundary violations detected in core library:")
        for v in violations:
            print(f"  - {v}")
        return 1

    print("BOS production boundary verification passed. No domain leaks found.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
