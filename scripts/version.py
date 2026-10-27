#!/usr/bin/env python3
"""Workspace version checks, explicit bumps and CI-owned immutable release tags."""
import argparse
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SEMVER = re.compile(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)")


def run(*args, root=ROOT):
    return subprocess.check_output(args, cwd=root, text=True).strip()


def version(root=ROOT):
    text = (root / "Cargo.toml").read_text()
    section = re.search(r"(?ms)^\[workspace\.package\]\s*\n(.*?)(?=^\[|\Z)", text)
    match = re.search(r'^version\s*=\s*"([^"]+)"', section[1], re.M) if section else None
    if not match or not SEMVER.fullmatch(match[1]):
        raise ValueError("Expected a stable X.Y.Z workspace version")
    return match[1]


def check(root=ROOT):
    current = version(root)
    blocks = (root / "Cargo.lock").read_text().split("[[package]]")[1:]
    for name in ("phone-core", "phone-desktop"):
        found = [b for b in blocks if re.search(r'^name = "' + re.escape(name) + r'"$', b, re.M)]
        if len(found) != 1 or not re.search(r'^version = "' + re.escape(current) + r'"$', found[0], re.M):
            raise ValueError(f"Cargo.lock version mismatch for {name}; run the bump command")
    return current


def bump(value, root=ROOT):
    old = check(root)
    parts = list(map(int, old.split('.')))
    if value in ("patch", "minor", "major"):
        index = {"major": 0, "minor": 1, "patch": 2}[value]
        parts[index] += 1
        parts[index + 1:] = [0] * (2 - index)
        value = '.'.join(map(str, parts))
    if not SEMVER.fullmatch(value) or tuple(map(int, value.split('.'))) <= tuple(map(int, old.split('.'))):
        raise ValueError("New version must be a larger stable X.Y.Z version")
    manifest, lock = root / "Cargo.toml", root / "Cargo.lock"
    original, original_lock = manifest.read_bytes(), lock.read_bytes()
    text = original.decode()
    section = re.search(r"(?ms)^\[workspace\.package\]\s*\n(.*?)(?=^\[|\Z)", text)
    updated = re.sub(r'^version\s*=\s*"[^"]+"', f'version = "{value}"', section[1], count=1, flags=re.M)
    manifest.write_bytes((text[:section.start(1)] + updated + text[section.end(1):]).encode())
    try:
        subprocess.run(["cargo", "update", "--offline", "--workspace"], cwd=root, check=True)
        check(root)
    except BaseException:
        manifest.write_bytes(original)
        lock.write_bytes(original_lock)
        raise
    return value


def release(root=ROOT):
    tag = "v" + check(root)
    if run("git", "rev-parse", f"refs/tags/{tag}^{{commit}}", root=root) != run("git", "rev-parse", "HEAD", root=root):
        raise ValueError("Release must build the commit named by the matching version tag")
    if os.environ.get("GITHUB_REF", "").startswith("refs/tags/") and os.environ["GITHUB_REF"] != f"refs/tags/{tag}":
        raise ValueError("Pushed tag does not match Cargo.toml")
    return tag


def tag(root=ROOT):
    if os.environ.get("GITHUB_REF") != "refs/heads/main":
        raise ValueError("Automatic tagging is restricted to CI on main")
    name = "v" + check(root)
    run("git", "fetch", "origin", "main", "--tags", root=root)
    head = run("git", "rev-parse", "HEAD", root=root)
    if head != run("git", "rev-parse", "origin/main", root=root):
        return name, False
    exists = subprocess.run(["git", "show-ref", "--verify", "--quiet", f"refs/tags/{name}"], cwd=root).returncode == 0
    if exists:
        return name, run("git", "rev-parse", f"refs/tags/{name}^{{commit}}", root=root) == head
    run("git", "-c", "user.name=github-actions[bot]", "-c", "user.email=41898282+github-actions[bot]@users.noreply.github.com", "tag", "-a", name, "-m", f"TA1278 {name[1:]}", root=root)
    run("git", "push", "origin", f"refs/tags/{name}", root=root)
    return name, True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("show", "check", "bump", "tag", "release"))
    parser.add_argument("value", nargs="?")
    args = parser.parse_args()
    outputs = {"version": check(), "tag": "v" + version()}
    if args.command == "bump":
        print(bump(args.value or "patch"))
        return
    if args.command == "tag":
        outputs["tag"], publish = tag()
        outputs["publish"] = str(publish).lower()
    elif args.command == "release":
        outputs["tag"] = release()
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a") as output:
            output.writelines(f"{key}={value}\n" for key, value in outputs.items())
    print(outputs["tag"] if args.command in ("tag", "release") else outputs["version"])


if __name__ == "__main__":
    main()
