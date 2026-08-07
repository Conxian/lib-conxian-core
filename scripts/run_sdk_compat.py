#!/usr/bin/env python3
"""Run the opt-in Core/SDK v2.0.14 compatibility evidence matrix."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "tests" / "sdk-compat" / "Cargo.toml"

SUPPORTED_SDK_FEATURES = (
    "default",
    "mock-cloud-enclave",
)
UNSUPPORTED_SDK_FEATURES = (
    "development-simulators",
    "bip110_compliant",
)
SDK_MATRIX = (
    ("default", ()),
    ("mock-cloud-enclave", ("mock-cloud-enclave",)),
    ("all-supported", ("all-supported",)),
)
CORE_MATRIX = (
    ("default", None),
    ("enclave", "core-enclave"),
)


def run_matrix(toolchain: str, offline: bool) -> None:
    print("SDK v2.0.14 supported features: " + ", ".join(SUPPORTED_SDK_FEATURES))
    print(
        "SDK v2.0.14 unsupported candidates (not run): "
        + ", ".join(UNSUPPORTED_SDK_FEATURES)
    )

    for core_name, core_feature in CORE_MATRIX:
        for sdk_name, sdk_features in SDK_MATRIX:
            features = ["run"]
            if core_feature is not None:
                features.append(core_feature)
            features.extend(sdk_features)

            command = ["cargo", f"+{toolchain}", "test"]
            if offline:
                command.append("--offline")
            command.extend(
                [
                    "--manifest-path",
                    str(MANIFEST),
                    "--locked",
                    "--no-default-features",
                    "--features",
                    ",".join(features),
                ]
            )

            print(
                f"\n== Core {core_name} / SDK {sdk_name} ==\n$ "
                + " ".join(command)
            )
            subprocess.run(command, cwd=ROOT, check=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--toolchain",
        default="1.97.1",
        help="Rust toolchain passed to Cargo (default: 1.97.1)",
    )
    parser.add_argument(
        "--offline",
        action="store_true",
        help="Pass Cargo's --offline flag for cache-only verification",
    )
    args = parser.parse_args()
    run_matrix(args.toolchain, args.offline)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
