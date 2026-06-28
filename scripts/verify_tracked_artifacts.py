import subprocess
import sys

FORBIDDEN_DIRS = ["target/", "dist/", "node_modules/", "build/"]

def main():
    print("Verifying tracked artifacts via git ls-files...")
    found_forbidden = False
    for d in FORBIDDEN_DIRS:
        try:
            result = subprocess.run(["git", "ls-files", d], capture_output=True, text=True, check=True)
            if result.stdout.strip():
                print(f"Error: Found forbidden files in '{d}' tracked in git:")
                print(result.stdout)
                found_forbidden = True
        except subprocess.CalledProcessError:
            continue

    if found_forbidden:
        sys.exit(1)
    else:
        print("No forbidden tracked artifacts found.")
        sys.exit(0)

if __name__ == "__main__":
    main()
