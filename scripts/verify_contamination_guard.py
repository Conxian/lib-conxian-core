import os
import re
import sys

# Architectural Boundary: Core library must not contain environment-specific side effects or network IO.
# See AGENTS.md (CON-700)
FORBIDDEN_PATTERNS = [
    r"std::net",
    r"std::fs",
    r"std::process",
    r"std::env",
]

def is_test_file(filepath):
    # Skip explicit test files
    return filepath.endswith("tests.rs") or "test" in os.path.basename(filepath).lower()

def scan_file(filepath):
    if is_test_file(filepath):
        return []

    violations = []
    try:
        with open(filepath, 'r') as f:
            content = f.read()
    except Exception as e:
        print(f"Error reading {filepath}: {e}")
        return []

    # Strip multi-line comments
    content = re.sub(r'/\*.*?\*/', '', content, flags=re.DOTALL)

    lines = content.splitlines()
    skip_next_item = False

    for i, line in enumerate(lines):
        clean_line = line.strip()

        # Skip single-line comments
        if clean_line.startswith("//"):
            continue

        # Simple attribute check: #[cfg(test)] or #[test]
        # If the line contains only an attribute, it applies to the next item.
        # We use a heuristic: if a line is just an attribute, skip the next line.
        if re.match(r'^#\[(cfg\(test\)|test)\]\s*$', clean_line):
            skip_next_item = True
            continue

        if skip_next_item:
            skip_next_item = False
            continue

        # In-line attribute: #[cfg(test)] use ...
        if re.search(r'#\[(cfg\(test\)|test)\]', clean_line):
            continue

        # Check for forbidden patterns
        for pattern in FORBIDDEN_PATTERNS:
            if re.search(pattern, clean_line):
                violations.append((i + 1, clean_line, pattern))

    return violations

def main():
    print("Verifying architectural boundaries via contamination guard...")
    src_dir = "src"
    all_violations = {}

    for root, _, files in os.walk(src_dir):
        for file in files:
            if file.endswith(".rs"):
                filepath = os.path.join(root, file)
                violations = scan_file(filepath)
                if violations:
                    all_violations[filepath] = violations

    if all_violations:
        print("\nError: Found architectural boundary violations (forbidden I/O in core):")
        for filepath, violations in all_violations.items():
            for line_num, content, pattern in violations:
                print(f"  {filepath}:{line_num} -> Found '{pattern}': {content}")
        print("\nRemediation: Move environment-specific side effects or network IO to 'conxian-gateway'.")
        sys.exit(1)
    else:
        print("Success: No architectural contamination detected in core library.")
        sys.exit(0)

if __name__ == "__main__":
    main()
