#!/usr/bin/env python3
"""Validate all native archives, upload a draft, then publish the version release."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from package import TARGETS
from version import check, release


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
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
