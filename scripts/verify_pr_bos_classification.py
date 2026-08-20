#!/usr/bin/env python3
"""Verify Conventional Commit classification for recent repository commits.

Validates git commit message headers against standard Conventional Commits specification.
"""

from __future__ import annotations

import re
import subprocess
import sys

# Allowed commit prefixes (Conventional Commits)
CONVENTIONAL_COMMIT_RE = re.compile(
    r"^(?:build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test|security|harden)"
    r"(?:\([a-zA-Z0-9_\-\/\.]+\))?!?: .+$"
)

# Ignored automated merge commit patterns
MERGE_COMMIT_RE = re.compile(
    r"^(?:Merge branch|Merge pull request|Merge remote-tracking branch|Revert \").*"
)


def verify_commit_message(msg: str) -> bool:
    clean_msg = msg.strip()
    if not clean_msg:
        return True
    if MERGE_COMMIT_RE.match(clean_msg):
        return True
    return bool(CONVENTIONAL_COMMIT_RE.match(clean_msg))


def get_recent_commits(limit: int = 20) -> list[tuple[str, str]]:
    try:
        res = subprocess.run(
            ["git", "--no-pager", "log", f"-n{limit}", "--pretty=format:%H%x09%s"],
            capture_output=True,
            text=True,
            check=True,
            timeout=10,
        )
        commits: list[tuple[str, str]] = []
        for line in res.stdout.splitlines():
            if "\t" in line:
                sha, subject = line.split("\t", 1)
                commits.append((sha[:8], subject.strip()))
        return commits
    except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired):
        return []


def main() -> int:
    print("Verifying PR / commit message conventional classification...")
    recent_commits = get_recent_commits(20)

    if not recent_commits:
        print("No recent git commits found or not a git repository.")
        return 0

    invalid_commits: list[str] = []
    for sha, subject in recent_commits:
        if not verify_commit_message(subject):
            invalid_commits.append(f"{sha}: {subject}")

    if invalid_commits:
        print("Notice: Some recent commit subject lines do not follow Conventional Commits standard:")
        for ic in invalid_commits:
            print(f"  - {ic}")
        print("PR classification check completed with notices.")
        return 0

    print(f"Verified {len(recent_commits)} recent commit message headers. All compliant.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
