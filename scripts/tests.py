#!/usr/bin/env python3
"""Release integrity checks, using isolated workspaces and a local bare Git remote."""
import os
from copy import deepcopy
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import version
import release as publishing


class Versions(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="ta1278-release-")
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.root = self.base / "app"
        self.root.mkdir()
        (self.root / "Cargo.toml").write_text('[workspace]\nmembers = ["core", "desktop"]\nresolver = "2"\n[workspace.package]\nversion = "0.2.2"\nedition = "2024"\n')
        for directory, name in (("core", "phone-core"), ("desktop", "phone-desktop")):
            path = self.root / directory
            (path / "src").mkdir(parents=True)
            (path / "Cargo.toml").write_text(f'[package]\nname = "{name}"\nversion.workspace = true\nedition.workspace = true\n')
            (path / "src/lib.rs").write_text('')
        self.command("cargo", "generate-lockfile", "--offline")

    def command(self, *args):
        return subprocess.check_output(args, cwd=self.root, text=True, stderr=subprocess.STDOUT).strip()

    def git(self):
        self.command("git", "init", "-b", "main")
        self.command("git", "config", "user.name", "Release test")
        self.command("git", "config", "user.email", "release@example.invalid")
        self.command("git", "init", "--bare", str(self.base / "remote.git"))
        self.command("git", "remote", "add", "origin", str(self.base / "remote.git"))
        self.command("git", "add", ".")
        self.command("git", "commit", "-m", "Initial version")
        self.command("git", "push", "-u", "origin", "main")

    def test_bump_updates_both_locked_packages_and_rejects_backwards_versions(self):
        self.assertEqual(version.bump("patch", self.root), "0.2.3")
        self.assertEqual(version.check(self.root), "0.2.3")
        self.assertEqual(version.bump("minor", self.root), "0.3.0")
        self.assertEqual(version.bump("major", self.root), "1.0.0")
        for value in ("0.9.0", "1.0.0", "1.2.3;echo bad", "1.2.3-beta"):
            with self.assertRaises(ValueError):
                version.bump(value, self.root)
        self.assertEqual(version.check(self.root), "1.0.0")

    def test_stale_lock_is_rejected(self):
        path = self.root / "Cargo.lock"
        path.write_text(path.read_text().replace('version = "0.2.2"', 'version = "0.2.1"', 1))
        with self.assertRaises(ValueError):
            version.check(self.root)

    def test_bump_preserves_crlf_without_doubling_carriage_returns(self):
        path = self.root / "Cargo.toml"
        path.write_bytes(path.read_text().replace('\n', '\r\n').encode())
        self.assertEqual(version.bump("patch", self.root), "0.2.3")
        self.assertNotIn(b'\r\r\n', path.read_bytes())
        self.assertIn(b'\r\n', path.read_bytes())

    def test_annotated_tag_is_idempotent_and_never_moves_for_unchanged_version(self):
        self.git()
        with patch.dict(os.environ, {"GITHUB_REF": "refs/heads/main"}):
            self.assertEqual(version.tag(self.root), ("v0.2.2", True))
            self.assertEqual(self.command("git", "cat-file", "-t", "refs/tags/v0.2.2"), "tag")
            original = self.command("git", "rev-parse", "refs/tags/v0.2.2")
            self.assertEqual(version.tag(self.root), ("v0.2.2", True))
            self.command("git", "commit", "--allow-empty", "-m", "Same-version change")
            self.command("git", "push", "origin", "main")
            self.assertEqual(version.tag(self.root), ("v0.2.2", False))
            self.assertEqual(self.command("git", "rev-parse", "refs/tags/v0.2.2"), original)
            with self.assertRaises(ValueError):
                version.release(self.root)

    def test_old_ci_commit_cannot_create_tag_and_other_branches_cannot_tag(self):
        self.git()
        self.command("git", "commit", "--allow-empty", "-m", "Newer main")
        self.command("git", "push", "origin", "main")
        self.command("git", "checkout", "HEAD^")
        with patch.dict(os.environ, {"GITHUB_REF": "refs/heads/main"}):
            self.assertEqual(version.tag(self.root), ("v0.2.2", False))
        with patch.dict(os.environ, {"GITHUB_REF": "refs/heads/topic"}):
            with self.assertRaises(ValueError):
                version.tag(self.root)
        self.assertEqual(self.command("git", "tag", "-l"), "")

    def ci_source(self):
        self.git()
        self.command("git", "tag", "-a", "v0.2.2", "-m", "Test version")
        source = {
            "head_sha": self.command("git", "rev-parse", "HEAD"),
            "head_branch": "main", "path": ".github/workflows/ci.yml", "event": "push",
            "repository": {"full_name": "owner/app"}, "head_repository": {"full_name": "owner/app"},
            "status": "in_progress", "conclusion": None,
        }
        jobs = [{"name": "build / " + name, "status": "completed", "conclusion": "success"}
                for name in ["checks", *publishing.TARGETS.values()]]
        # CI dispatches Release while its final tagging job is still finishing.
        jobs.append({"name": "tag", "status": "in_progress", "conclusion": None})
        return source, {"jobs": jobs}

    def verify_source(self, source, jobs):
        with patch.dict(os.environ, {"GITHUB_REPOSITORY": "owner/app", "GITHUB_REF": "refs/tags/v0.2.2"}), patch.object(publishing, "github", side_effect=[source, jobs]):
            publishing.verify_source("123", self.root)

    def test_release_reuses_only_matching_successful_ci_builds(self):
        source, jobs = self.ci_source()
        self.verify_source(source, jobs)
        source.update(status="completed", conclusion="success", event="workflow_dispatch")
        self.verify_source(source, jobs)
        for changes in (
            {"head_sha": "0" * 40}, {"head_branch": "other"}, {"event": "pull_request"},
            {"path": ".github/workflows/other.yml"}, {"repository": {"full_name": "other/app"}},
            {"head_repository": {"full_name": "fork/app"}}, {"conclusion": "failure"},
            {"status": "queued"},
        ):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                self.verify_source(dict(source, **changes), jobs)
        for status, conclusion in (("completed", "failure"), ("in_progress", None), ("completed", "skipped")):
            incomplete = deepcopy(jobs)
            incomplete["jobs"][1].update(status=status, conclusion=conclusion)
            with self.subTest(status=status, conclusion=conclusion), self.assertRaises(ValueError):
                self.verify_source(source, incomplete)
        with self.assertRaises(ValueError):
            self.verify_source(source, {"jobs": jobs["jobs"][1:]})
        with patch.object(publishing, "github") as api:
            for value in ("", "0", "-1", "1/jobs", "１２３"):
                with self.subTest(value=value), self.assertRaises(ValueError):
                    publishing.verify_source(value, self.root)
            api.assert_not_called()
        self.command("git", "commit", "--allow-empty", "-m", "Different commit")
        with self.assertRaises(ValueError):
            self.verify_source(source, jobs)


if __name__ == "__main__":
    unittest.main()
