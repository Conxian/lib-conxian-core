from __future__ import annotations

import io
import json
import os
import tempfile
import unittest
from pathlib import Path
from typing import Any
from urllib.error import HTTPError
from unittest.mock import patch

from scripts import verify_release_version as guard


COMMIT = "a" * 40
PACKAGE = "lib-conxian-core"
VERSION = "0.2.12"
TAG = f"v{VERSION}"
ANNOTATED_TAG_OBJECT = "b" * 40
SECOND_TAG_OBJECT = "c" * 40


class JsonResponse(io.BytesIO):
    def __init__(self, payload: dict[str, Any]):
        super().__init__(json.dumps(payload).encode("utf-8"))

    def __enter__(self) -> "JsonResponse":
        return self

    def __exit__(self, *args: Any) -> None:
        self.close()


class FakeOpener:
    def __init__(self, responses: list[dict[str, Any]]):
        self.responses = list(responses)
        self.urls: list[str] = []

    def __call__(self, request: Any, timeout: float) -> JsonResponse:
        self.urls.append(request.full_url)
        return JsonResponse(self.responses.pop(0))


class ErrorOpener:
    def __init__(self, error: Exception):
        self.error = error
        self.urls: list[str] = []

    def __call__(self, request: Any, timeout: float) -> JsonResponse:
        self.urls.append(request.full_url)
        raise self.error


class FakeRegistry:
    def __init__(self, published: str | None = None, sequence: list[str | None] | None = None):
        self.published = published
        self.sequence = list(sequence or [])
        self.calls: list[tuple[str, str]] = []

    def get_version(self, package_name: str, version: str) -> guard.RegistryVersion | None:
        self.calls.append((package_name, version))
        if self.sequence:
            published = self.sequence.pop(0)
        else:
            published = self.published
        return guard.RegistryVersion(published) if published is not None else None


class FakeGitHub:
    def __init__(self, target: str = COMMIT, release: dict[str, Any] | None = None):
        self.target = target
        self.release = release
        self.tag_calls: list[str] = []
        self.release_calls: list[str] = []

    def get_tag_target(self, tag: str) -> str | None:
        self.tag_calls.append(tag)
        return self.target

    def get_release(self, tag: str) -> dict[str, Any] | None:
        self.release_calls.append(tag)
        return self.release


class FailingGitHub(FakeGitHub):
    def get_tag_target(self, tag: str) -> str | None:
        self.tag_calls.append(tag)
        raise guard.RemoteCheckError("GitHub returned HTTP 503 for tag lookup")


def write_fixture(
    root: Path,
    *,
    version: str = VERSION,
    lock_version: str = VERSION,
    readme_version: str = VERSION,
    include_release: bool = True,
    changelog_text: str | None = None,
) -> None:
    (root / "Cargo.toml").write_text(
        f'''[package]\nname = "{PACKAGE}"\nversion = "{version}"\n\n[dependencies]\n''',
        encoding="utf-8",
    )
    (root / "Cargo.lock").write_text(
        f'''version = 3\n\n[[package]]\nname = "{PACKAGE}"\nversion = "{lock_version}"\ndependencies = []\n\n[[package]]\nname = "{PACKAGE}-fuzz"\nversion = "0.0.0"\nsource = "path+file:///workspace/fuzz"\n''',
        encoding="utf-8",
    )
    (root / "README.md").write_text(
        f'''[![Version](https://img.shields.io/badge/version-{readme_version}-blue.svg)](CHANGELOG.md)\n\n**v{readme_version} {'Pre-release' if '-' in readme_version else 'Stable'}.**\n\n```toml\nlib-conxian-core = "{readme_version}"\nlib-conxian-core = {{ version = "{readme_version}", features = ["enclave"] }}\n```\n''',
        encoding="utf-8",
    )
    if changelog_text is None:
        release_heading = f"## [v{version}] - 2026-07-15\n" if include_release else ""
        changelog_text = f"## [Unreleased]\n\n- Historical note for v{version}.\n\n{release_heading}"
    (root / "CHANGELOG.md").write_text(changelog_text, encoding="utf-8")


class ReleaseVersionGuardTests(unittest.TestCase):
    def repo(self, **kwargs: Any):
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        write_fixture(root, **kwargs)
        return temporary, root

    def test_source_parity_success(self) -> None:
        temporary, root = self.repo()
        self.addCleanup(temporary.cleanup)

        metadata, errors = guard.check_source_parity(root)

        self.assertEqual(errors, [])
        self.assertIsNotNone(metadata)
        self.assertEqual(metadata.package_name, PACKAGE)
        self.assertEqual(metadata.version, VERSION)

    def test_semver_prerelease_and_build_metadata_are_supported(self) -> None:
        for version in (
            "0.2.13-rc.1",
            "1.2.3-alpha.0",
            "1.2.3-0A",
            "1.2.3+build.7",
            "1.2.3-rc.1+build.7",
        ):
            with self.subTest(version=version):
                self.assertTrue(guard.is_valid_semver(version))
                self.assertTrue(guard.is_valid_release_tag(f"v{version}"))

    def test_malformed_semver_prerelease_is_rejected(self) -> None:
        for version in (
            "0.2.13-",
            "0.2.13-rc..1",
            "0.2.13-01",
            "0.2.13-rc.01",
            "0.2.13-rc_1",
            "01.2.13-rc.1",
            "0.2.13+",
        ):
            with self.subTest(version=version):
                self.assertFalse(guard.is_valid_semver(version))

    def test_prerelease_matches_source_tag_and_external_phases(self) -> None:
        version = "0.2.13-rc.1"
        tag = f"v{version}"
        temporary, root = self.repo(
            version=version,
            lock_version=version,
            readme_version=version,
        )
        self.addCleanup(temporary.cleanup)
        registry = FakeRegistry()
        github = FakeGitHub()

        errors = guard.verify_phase(
            root,
            "pre-publish",
            tag=tag,
            repository="Conxian/lib-conxian-core",
            source_revision=COMMIT,
            registry=registry,
            github=github,
        )
        self.assertEqual(errors, [])

        registry.published = version
        errors = guard.verify_phase(
            root,
            "post-publish",
            tag=tag,
            repository="Conxian/lib-conxian-core",
            source_revision=COMMIT,
            registry=registry,
            github=github,
            registry_attempts=1,
            registry_delay_seconds=0,
        )
        self.assertEqual(errors, [])

        github.release = {"tag_name": tag, "draft": False}
        errors = guard.verify_phase(
            root,
            "post-release",
            tag=tag,
            repository="Conxian/lib-conxian-core",
            source_revision=COMMIT,
            registry=registry,
            github=github,
            registry_attempts=1,
            registry_delay_seconds=0,
        )
        self.assertEqual(errors, [])

    def test_build_metadata_is_exactly_matched(self) -> None:
        version = "0.2.13-rc.1+build.7"
        temporary, root = self.repo(
            version=version,
            lock_version=version,
            readme_version=version,
        )
        self.addCleanup(temporary.cleanup)

        metadata, errors = guard.check_source_parity(root)

        self.assertEqual(errors, [])
        self.assertIsNotNone(metadata)
        self.assertEqual(metadata.version, version)

    def test_cargo_lock_mismatch_is_rejected(self) -> None:
        temporary, root = self.repo(lock_version="0.2.11")
        self.addCleanup(temporary.cleanup)

        _, errors = guard.check_source_parity(root)

        self.assertTrue(any("Cargo.lock root package version" in error for error in errors))

    def test_readme_mismatch_is_rejected(self) -> None:
        temporary, root = self.repo(readme_version="0.2.11")
        self.addCleanup(temporary.cleanup)

        _, errors = guard.check_source_parity(root)

        self.assertTrue(any("README.md" in error for error in errors))

    def test_missing_changelog_section_is_rejected(self) -> None:
        temporary, root = self.repo(include_release=False)
        self.addCleanup(temporary.cleanup)

        _, errors = guard.check_source_parity(root)

        self.assertTrue(any("CHANGELOG.md is missing" in error for error in errors))

    def test_unreleased_and_historical_mentions_do_not_satisfy_release_heading(self) -> None:
        changelog = """## [Unreleased]

- Preparing v0.2.13 while historical notes mention v0.2.13.

## [v0.2.12] - 2026-07-15

- Historical v0.2.13 reference only.
"""
        temporary, root = self.repo(
            version="0.2.13",
            lock_version="0.2.13",
            readme_version="0.2.13",
            changelog_text=changelog,
        )
        self.addCleanup(temporary.cleanup)

        _, errors = guard.check_source_parity(root)

        self.assertEqual(
            [error for error in errors if "CHANGELOG.md is missing" in error],
            ["CHANGELOG.md is missing the exact release section `## [v0.2.13]`"],
        )

    def test_malformed_or_mismatched_tag_is_rejected(self) -> None:
        temporary, root = self.repo()
        self.addCleanup(temporary.cleanup)
        registry = FakeRegistry()
        github = FakeGitHub()

        for tag in (
            "v0.2",
            "release-0.2.12",
            "v0.2.11",
            "v0.2.12-01",
            "v0.2.12-rc..1",
        ):
            with self.subTest(tag=tag):
                errors = guard.verify_phase(
                    root,
                    "pre-publish",
                    tag=tag,
                    repository="Conxian/lib-conxian-core",
                    source_revision=COMMIT,
                    registry=registry,
                    github=github,
                )
                self.assertTrue(any("release tag" in error for error in errors))
                self.assertEqual(github.tag_calls, [])

    def test_github_client_uses_runner_api_url(self) -> None:
        opener = FakeOpener(
            [
                {
                    "ref": f"refs/tags/{TAG}",
                    "object": {"type": "commit", "sha": COMMIT},
                }
            ]
        )

        with patch.dict(os.environ, {"GITHUB_API_URL": "https://github.example/api/v3"}):
            client = guard.GitHubClient("Conxian/lib-conxian-core", opener=opener)
            self.assertEqual(client.get_tag_target(TAG), COMMIT)

        self.assertEqual(
            opener.urls,
            [f"https://github.example/api/v3/repos/Conxian/lib-conxian-core/git/ref/tags/{TAG}"],
        )

    def test_github_client_falls_back_to_public_api_url(self) -> None:
        opener = FakeOpener(
            [
                {
                    "ref": f"refs/tags/{TAG}",
                    "object": {"type": "commit", "sha": COMMIT},
                }
            ]
        )

        with patch.dict(os.environ, {}, clear=True):
            client = guard.GitHubClient("Conxian/lib-conxian-core", opener=opener)
            self.assertEqual(client.get_tag_target(TAG), COMMIT)

        self.assertEqual(
            opener.urls,
            [f"https://api.github.com/repos/Conxian/lib-conxian-core/git/ref/tags/{TAG}"],
        )

    def test_github_client_resolves_annotated_tag_to_commit(self) -> None:
        opener = FakeOpener(
            [
                {
                    "ref": f"refs/tags/{TAG}",
                    "object": {"type": "tag", "sha": ANNOTATED_TAG_OBJECT},
                },
                {"object": {"type": "commit", "sha": COMMIT}},
            ]
        )

        client = guard.GitHubClient("Conxian/lib-conxian-core", opener=opener)

        self.assertEqual(client.get_tag_target(TAG), COMMIT)
        self.assertEqual(
            opener.urls,
            [
                f"https://api.github.com/repos/Conxian/lib-conxian-core/git/ref/tags/{TAG}",
                f"https://api.github.com/repos/Conxian/lib-conxian-core/git/tags/{ANNOTATED_TAG_OBJECT}",
            ],
        )

    def test_github_client_treats_tag_404_as_absence(self) -> None:
        opener = ErrorOpener(
            HTTPError(
                f"https://api.github.com/repos/Conxian/lib-conxian-core/git/ref/tags/{TAG}",
                404,
                "Not Found",
                None,
                io.BytesIO(),
            )
        )
        client = guard.GitHubClient("Conxian/lib-conxian-core", opener=opener)

        self.assertIsNone(client.get_tag_target(TAG))

    def test_remote_tag_lookup_failure_is_not_reported_as_missing_tag(self) -> None:
        temporary, root = self.repo()
        self.addCleanup(temporary.cleanup)
        github = FailingGitHub()

        errors = guard.verify_phase(
            root,
            "pre-publish",
            tag=TAG,
            repository="Conxian/lib-conxian-core",
            source_revision=COMMIT,
            registry=FakeRegistry(),
            github=github,
        )

        self.assertEqual(
            errors,
            [
                "GitHub tag lookup failed for 'v0.2.12': "
                "GitHub returned HTTP 503 for tag lookup"
            ],
        )
        self.assertFalse(any("does not exist" in error for error in errors))
        self.assertEqual(github.release_calls, [])

    def test_annotated_tag_source_mismatch_is_rejected(self) -> None:
        other_commit = "d" * 40
        opener = FakeOpener(
            [
                {
                    "ref": f"refs/tags/{TAG}",
                    "object": {"type": "tag", "sha": ANNOTATED_TAG_OBJECT},
                },
                {"object": {"type": "commit", "sha": other_commit}},
            ]
        )
        github = guard.GitHubClient("Conxian/lib-conxian-core", opener=opener)
        temporary, root = self.repo()
        self.addCleanup(temporary.cleanup)

        errors = guard.verify_phase(
            root,
            "pre-publish",
            tag=TAG,
            repository="Conxian/lib-conxian-core",
            source_revision=COMMIT,
            registry=FakeRegistry(),
            github=github,
        )

        self.assertTrue(any("points at" in error for error in errors))

    def test_annotated_tag_cycle_is_rejected(self) -> None:
        opener = FakeOpener(
            [
                {
                    "ref": f"refs/tags/{TAG}",
                    "object": {"type": "tag", "sha": ANNOTATED_TAG_OBJECT},
                },
                {"object": {"type": "tag", "sha": SECOND_TAG_OBJECT}},
                {"object": {"type": "tag", "sha": ANNOTATED_TAG_OBJECT}},
            ]
        )

        client = guard.GitHubClient("Conxian/lib-conxian-core", opener=opener)

        with self.assertRaisesRegex(guard.RemoteCheckError, "cyclic"):
            client.get_tag_target(TAG)

    def test_pre_publish_rejects_existing_crate_and_release(self) -> None:
        temporary, root = self.repo()
        self.addCleanup(temporary.cleanup)
        registry = FakeRegistry(published=VERSION)
        github = FakeGitHub(release={"tag_name": TAG, "draft": False})

        errors = guard.verify_phase(
            root,
            "pre-publish",
            tag=TAG,
            repository="Conxian/lib-conxian-core",
            source_revision=COMMIT,
            registry=registry,
            github=github,
        )

        self.assertTrue(any("already contains" in error for error in errors))
        self.assertTrue(any("already exists" in error for error in errors))

    def test_recovery_requires_published_candidate(self) -> None:
        temporary, root = self.repo()
        self.addCleanup(temporary.cleanup)
        registry = FakeRegistry()
        github = FakeGitHub()

        errors = guard.verify_phase(
            root,
            "post-publish",
            tag=TAG,
            repository="Conxian/lib-conxian-core",
            source_revision=COMMIT,
            registry=registry,
            github=github,
            registry_attempts=1,
            registry_delay_seconds=0,
        )

        self.assertTrue(any("did not expose" in error for error in errors))

    def test_recovery_accepts_published_candidate_without_release(self) -> None:
        temporary, root = self.repo()
        self.addCleanup(temporary.cleanup)
        registry = FakeRegistry(published=VERSION)
        github = FakeGitHub(release=None)

        errors = guard.verify_phase(
            root,
            "post-publish",
            tag=TAG,
            repository="Conxian/lib-conxian-core",
            source_revision=COMMIT,
            registry=registry,
            github=github,
            registry_attempts=1,
            registry_delay_seconds=0,
        )

        self.assertEqual(errors, [])

    def test_post_release_requires_matching_release(self) -> None:
        temporary, root = self.repo()
        self.addCleanup(temporary.cleanup)
        registry = FakeRegistry(published=VERSION)
        github = FakeGitHub(release=None)

        errors = guard.verify_phase(
            root,
            "post-release",
            tag=TAG,
            repository="Conxian/lib-conxian-core",
            source_revision=COMMIT,
            registry=registry,
            github=github,
            registry_attempts=1,
            registry_delay_seconds=0,
        )
        self.assertTrue(any("is missing" in error for error in errors))

        github.release = {"tag_name": TAG, "draft": False}
        errors = guard.verify_phase(
            root,
            "post-release",
            tag=TAG,
            repository="Conxian/lib-conxian-core",
            source_revision=COMMIT,
            registry=registry,
            github=github,
            registry_attempts=1,
            registry_delay_seconds=0,
        )
        self.assertEqual(errors, [])

    def test_registry_polling_is_bounded_and_retries_propagation(self) -> None:
        registry = FakeRegistry(sequence=[None, None, VERSION])
        delays: list[float] = []

        published = guard.wait_for_registry_version(
            registry,
            PACKAGE,
            VERSION,
            attempts=3,
            delay_seconds=2.5,
            sleep_fn=delays.append,
        )

        self.assertEqual(published.number, VERSION)
        self.assertEqual(len(registry.calls), 3)
        self.assertEqual(delays, [2.5, 2.5])


if __name__ == "__main__":
    unittest.main()
