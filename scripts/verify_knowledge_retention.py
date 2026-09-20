#!/usr/bin/env python3
"""Verify knowledge retention and governance scorecards.

Validates the presence and structural integrity of required governance documents:
- docs/governance/CXIP_INDEX.md
- docs/governance/READINESS_SCORECARD.md
- docs/governance/EXECUTIVE_SCORECARD.md
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_GOVERNANCE_DOCS = [
    ROOT / "docs" / "governance" / "CXIP_INDEX.md",
    ROOT / "docs" / "governance" / "READINESS_SCORECARD.md",
    ROOT / "docs" / "governance" / "EXECUTIVE_SCORECARD.md",
]


def verify_doc(doc_path: Path) -> list[str]:
    violations: list[str] = []
    rel_path = doc_path.relative_to(ROOT)

    if not doc_path.is_file():
        violations.append(f"Missing required governance document: {rel_path}")
        return violations

    try:
        content = doc_path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        violations.append(f"Failed to read UTF-8 content from {rel_path}")
        return violations

    if len(content.strip()) < 100:
        violations.append(f"Document {rel_path} is suspiciously short or empty.")

    # Validate Markdown table headers exist in scorecards / indices
    if "| " not in content or "---" not in content:
        violations.append(f"Document {rel_path} lacks expected markdown table structure.")

    return violations


def main() -> int:
    print("Verifying knowledge retention and governance scorecards...")
    violations: list[str] = []

    for doc in REQUIRED_GOVERNANCE_DOCS:
        violations.extend(verify_doc(doc))

    if violations:
        print("Error: Knowledge retention verification failed:")
        for v in violations:
            print(f"  - {v}")
        return 1

    print("Knowledge retention and governance scorecard verification passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
