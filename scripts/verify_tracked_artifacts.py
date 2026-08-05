import subprocess
import sys

# Core/Gateway shared forbidden tracking patterns
FORBIDDEN_PATTERNS = [
    # Generated/runtime directories
    "target/",
    "dist/",
    "node_modules/",
    "build/",
    "test-results/",
    "playwright-report/",
    # Sensitive configuration & secrets
    ".env*",
    "*.pem",
    "*.key",
    "*.pub",
]

def verify_patterns(patterns=None, git_dir=None):
    if patterns is None:
        patterns = FORBIDDEN_PATTERNS

    found_forbidden = False
    for pattern in patterns:
        try:
            cmd = ["git"]
            if git_dir:
                cmd.extend(["-C", git_dir])
            cmd.extend(["ls-files", pattern])
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            if result.stdout.strip():
                print(f"Error: Found forbidden tracked files matching '{pattern}':")
                print(result.stdout)
                found_forbidden = True
        except subprocess.CalledProcessError:
            # git ls-files might exit with non-zero under some conditions, ignore gracefully
            continue

    return not found_forbidden

def main():
    print("Verifying tracked artifacts via git ls-files...")
    if not verify_patterns():
        sys.exit(1)
    else:
        print("No forbidden tracked artifacts found.")
        sys.exit(0)

if __name__ == "__main__":
    main()
