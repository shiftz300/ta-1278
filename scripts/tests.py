#!/usr/bin/env python3
"""Release integrity checks, using isolated workspaces and a local bare Git remote."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import version


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


if __name__ == "__main__":
    unittest.main()
