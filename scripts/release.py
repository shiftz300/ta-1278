#!/usr/bin/env python3
"""Validate desktop archives and the Android APK, then publish the version release."""
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


def verify_source(run_id, root=ROOT, *, android=False):
    """Accept only this tagged commit's successful native CI jobs in this repo."""
    if not run_id.isascii() or not run_id.isdigit() or int(run_id) < 1:
        raise ValueError("CI run must be a positive run ID")
    release(root)
    repository = os.environ["GITHUB_REPOSITORY"]
    path = f"repos/{repository}/actions/runs/{run_id}"
    source = github(path)
    if (source["head_sha"] != run("git", "rev-parse", "HEAD", root=root)
            or source["head_branch"] != "main"
            or source["path"] != (".github/workflows/android.yml" if android else ".github/workflows/ci.yml")
            or source["event"] not in ("push", "workflow_dispatch")
            or source["repository"]["full_name"] != repository
            or source["head_repository"]["full_name"] != repository
            or source["status"] not in ("in_progress", "completed")
            or source["conclusion"] not in (None, "success")):
        raise ValueError("CI source must match this repository, workflow and tagged commit")
    if android and (source["status"] != "completed" or source["conclusion"] != "success"):
        raise ValueError("Android CI must finish successfully before reuse")
    jobs = github(path + "/jobs?filter=latest&per_page=100")["jobs"]
    required = {"Android arm64"} if android else {"build / checks"} | {f"build / {p}" for p in TARGETS.values()}
    passed = {job["name"] for job in jobs if job["status"] == "completed" and job["conclusion"] == "success"}
    if not required.issubset(passed):
        raise ValueError("Every native build and shared check must succeed before reuse")


def find_android(root=ROOT):
    """Reuse only a successful same-commit Android run with an unexpired artifact."""
    release(root)
    repository = os.environ["GITHUB_REPOSITORY"]
    sha = run("git", "rev-parse", "HEAD", root=root)
    base = f"repos/{repository}/actions"
    runs = github(base + f"/workflows/android.yml/runs?head_sha={sha}&status=success&per_page=100")
    for source in runs["workflow_runs"]:
        if source["head_branch"] != "main":
            continue
        run_id = str(source["id"])
        verify_source(run_id, root, android=True)
        artifacts = github(base + f"/runs/{run_id}/artifacts?per_page=100")["artifacts"]
        if any(a["name"] == "ta1278-android-arm64" and not a["expired"] for a in artifacts):
            return run_id
    return ""


def packages(directory, version):
    """Require all desktop packages and the APK, each with its exact checksum."""
    names = [f"TA1278-v{version}-{platform}" + (".tar.gz" if "linux" in target else ".zip")
             for target, platform in TARGETS.items()]
    names.append(f"TA1278-v{version}-android-arm64.apk")
    files, lines = [], []
    for name in names:
        file = directory / name
        digest = hashlib.sha256(file.read_bytes()).hexdigest()
        checksum = file.with_name(name + ".sha256")
        if checksum.read_text().split() != [digest, name]:
            raise ValueError(f"Checksum mismatch: {name}")
        files.extend((str(file), str(checksum)))
        lines.append(f"{digest}  {name}\n")
    return files, ''.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path, nargs="?")
    parser.add_argument("--ci-run", help="Validate a CI run before reusing its artifacts")
    parser.add_argument("--find-android", action="store_true", help="Find reusable Android artifacts")
    args = parser.parse_args()
    if args.find_android:
        run_id = find_android()
        if os.environ.get("GITHUB_OUTPUT"):
            with open(os.environ["GITHUB_OUTPUT"], "a") as output:
                output.write(f"run={run_id}\n")
        print(run_id or "No reusable Android artifact; build from the tagged commit")
        return
    if args.ci_run is not None:
        verify_source(args.ci_run)
        return
    if args.directory is None:
        parser.error("An artifact directory or --ci-run is required")
    tag = release()
    version = check()
    archives, lines = packages(args.directory, version)
    sums = args.directory / "SHA256SUMS"
    sums.write_text(lines)
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
