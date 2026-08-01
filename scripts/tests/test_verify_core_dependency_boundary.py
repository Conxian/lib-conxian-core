#!/usr/bin/env python3

import unittest

from scripts.verify_core_dependency_boundary import validate_metadata


CORE_ID = "path+file:///repo#lib-conxian-core@0.3.1"
SDK_COMPAT_ID = "path+file:///repo/tests/sdk-compat#sdk-compat@0.1.0"


def package(package_id, name, version):
    return {"id": package_id, "name": name, "version": version}


def metadata(core_dependencies=None, extra_packages=None, extra_nodes=None):
    modern_rustls_id = "registry+index#rustls@0.23.40"
    modern_webpki_id = "registry+index#rustls-webpki@0.103.13"
    return {
        "workspace_members": [CORE_ID, SDK_COMPAT_ID],
        "packages": [
            package(CORE_ID, "lib-conxian-core", "0.3.1"),
            package(SDK_COMPAT_ID, "sdk-compat", "0.1.0"),
            package(modern_rustls_id, "rustls", "0.23.40"),
            package(modern_webpki_id, "rustls-webpki", "0.103.13"),
            *(extra_packages or []),
        ],
        "resolve": {
            "nodes": [
                {"id": CORE_ID, "dependencies": core_dependencies or []},
                {
                    "id": SDK_COMPAT_ID,
                    "dependencies": [CORE_ID, modern_rustls_id],
                },
                {"id": modern_rustls_id, "dependencies": [modern_webpki_id]},
                {"id": modern_webpki_id, "dependencies": []},
                *(extra_nodes or []),
            ]
        },
    }


class CoreDependencyBoundaryTests(unittest.TestCase):
    def test_accepts_core_closure_while_ignoring_independent_modern_tls(self):
        self.assertEqual(validate_metadata(metadata()), [])

    def test_rejects_prohibited_packages_in_core_closure(self):
        cases = [
            ("bdk", "0.30.2", "unused wallet implementation"),
            ("electrum-client", "0.18.0", "network transport implementation"),
            ("sled", "0.34.7", "persistence implementation"),
            ("rustls", "0.21.12", "legacy TLS implementation"),
            (
                "rustls-webpki",
                "0.101.7",
                "legacy TLS certificate implementation",
            ),
        ]
        for name, version, expected in cases:
            with self.subTest(package=name):
                package_id = f"registry+index#{name}@{version}"
                violations = validate_metadata(
                    metadata(
                        core_dependencies=[package_id],
                        extra_packages=[package(package_id, name, version)],
                        extra_nodes=[{"id": package_id, "dependencies": []}],
                    )
                )
                self.assertTrue(any(expected in item for item in violations), violations)


if __name__ == "__main__":
    unittest.main()
