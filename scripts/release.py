#!/usr/bin/env python3
"""Validate all native archives, upload a draft, then publish the version release."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
from package import TARGETS
from version import ROOT, check, release, run


def github(path):
    return json.loads(subprocess.check_output(["gh", "api", path], text=True))


def verify_source(run_id, root=ROOT):
    """Accept only this tagged commit's successful native CI jobs in this repo."""
    if not run_id.isascii() or not run_id.isdigit() or int(run_id) < 1:
        raise ValueError("CI run must be a positive run ID")
    release(root)
    repository = os.environ["GITHUB_REPOSITORY"]
    path = f"repos/{repository}/actions/runs/{run_id}"
    source = github(path)
    if (source["head_sha"] != run("git", "rev-parse", "HEAD", root=root)
            or source["head_branch"] != "main"
            or source["path"] != ".github/workflows/ci.yml"
            or source["event"] not in ("push", "workflow_dispatch")
            or source["repository"]["full_name"] != repository
            or source["head_repository"]["full_name"] != repository
            or source["status"] not in ("in_progress", "completed")
            or source["conclusion"] not in (None, "success")):
        raise ValueError("CI source must match this repository, workflow and tagged commit")
    jobs = github(path + "/jobs?filter=latest&per_page=100")["jobs"]
    required = {"build / checks"} | {f"build / {p}" for p in TARGETS.values()}
    passed = {job["name"] for job in jobs if job["status"] == "completed" and job["conclusion"] == "success"}
    if not required.issubset(passed):
        raise ValueError("Every native build and shared check must succeed before reuse")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path, nargs="?")
    parser.add_argument("--ci-run", help="Validate a CI run before reusing its artifacts")
    args = parser.parse_args()
    if args.ci_run is not None:
        verify_source(args.ci_run)
        return
    if args.directory is None:
        parser.error("An artifact directory or --ci-run is required")
    tag = release()
    version = check()
    archives = []
    lines = []
    for target, platform in TARGETS.items():
        name = f"TA1278-v{version}-{platform}" + (".tar.gz" if "linux" in target else ".zip")
        file = args.directory / name
        digest = hashlib.sha256(file.read_bytes()).hexdigest()
        expected = file.with_name(file.name + ".sha256").read_text().split()
        if expected != [digest, name]:
            raise ValueError(f"Checksum mismatch: {name}")
        archives.append(str(file))
        lines.append(f"{digest}  {name}\n")
    sums = args.directory / "SHA256SUMS"
    sums.write_text(''.join(lines))
    existing = subprocess.run(["gh", "release", "view", tag, "--json", "isDraft"], text=True, capture_output=True)
    if existing.returncode == 0:
        if not json.loads(existing.stdout)["isDraft"]:
            print(f"{tag} is already published; keeping its existing assets")
            return
    else:
        subprocess.run(["gh", "release", "create", tag, "--draft", "--verify-tag", "--generate-notes", "--title", f"TA1278 {version}"], check=True)
    # Only a draft can be resumed/replaced. Published version assets stay immutable.
    subprocess.run(["gh", "release", "upload", tag, *archives, str(sums), "--clobber"], check=True)
    subprocess.run(["gh", "release", "edit", tag, "--draft=false"], check=True)


if __name__ == "__main__":
    main()
