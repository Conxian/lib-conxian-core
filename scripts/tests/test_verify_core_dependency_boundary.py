#!/usr/bin/env python3

import unittest

from scripts.verify_core_dependency_boundary import validate_metadata


CORE_ID = "path+file:///repo#lib-conxian-core@0.2.5"
GATEWAY_ID = "path+file:///repo/gateway#conxian-gateway@0.2.5"
BDK_ID = "registry+https://github.com/rust-lang/crates.io-index#bdk@0.30.2"


def package(package_id, name, version):
    return {"id": package_id, "name": name, "version": version}


def metadata(
    core_dependencies=None,
    bdk_dependencies=None,
    bdk_features=None,
    extra_packages=None,
    extra_nodes=None,
):
    modern_rustls_id = "registry+index#rustls@0.23.40"
    modern_webpki_id = "registry+index#rustls-webpki@0.103.13"
    return {
        "workspace_members": [CORE_ID, GATEWAY_ID],
        "packages": [
            package(CORE_ID, "lib-conxian-core", "0.2.5"),
            package(GATEWAY_ID, "conxian-gateway", "0.2.5"),
            package(BDK_ID, "bdk", "0.30.2"),
            package(modern_rustls_id, "rustls", "0.23.40"),
            package(modern_webpki_id, "rustls-webpki", "0.103.13"),
            *(extra_packages or []),
        ],
        "resolve": {
            "nodes": [
                {"id": CORE_ID, "dependencies": [BDK_ID, *(core_dependencies or [])]},
                {
                    "id": GATEWAY_ID,
                    "dependencies": [CORE_ID, modern_rustls_id],
                },
                {
                    "id": BDK_ID,
                    "dependencies": bdk_dependencies or [],
                    "features": bdk_features if bdk_features is not None else ["std"],
                },
                {"id": modern_rustls_id, "dependencies": [modern_webpki_id]},
                {"id": modern_webpki_id, "dependencies": []},
                *(extra_nodes or []),
            ]
        },
    }


class CoreDependencyBoundaryTests(unittest.TestCase):
    def test_accepts_core_closure_while_ignoring_gateway_modern_tls(self):
        self.assertEqual(validate_metadata(metadata()), [])

    def test_rejects_prohibited_packages_in_core_closure(self):
        cases = [
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
                        bdk_dependencies=[package_id],
                        extra_packages=[package(package_id, name, version)],
                        extra_nodes=[{"id": package_id, "dependencies": []}],
                    )
                )
                self.assertTrue(any(expected in item for item in violations), violations)

    def test_rejects_bdk_transport_or_persistence_features(self):
        for feature in ("electrum", "key-value-db"):
            with self.subTest(feature=feature):
                violations = validate_metadata(metadata(bdk_features=["std", feature]))
                self.assertIn(
                    f"bdk 0.30.x enables prohibited feature '{feature}'", violations
                )

    def test_rejects_bdk_without_std(self):
        self.assertIn(
            "bdk 0.30.x must enable feature 'std'",
            validate_metadata(metadata(bdk_features=[])),
        )


if __name__ == "__main__":
    unittest.main()
