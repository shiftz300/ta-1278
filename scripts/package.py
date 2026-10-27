#!/usr/bin/env python3
"""Build a portable desktop archive on its native OS; no installation or user data."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import zipfile
from version import ROOT, check, run

TARGETS = {
    "aarch64-apple-darwin": "macos-arm64",
    "x86_64-apple-darwin": "macos-x86_64",
    "x86_64-pc-windows-msvc": "windows-x86_64",
    "x86_64-unknown-linux-gnu": "linux-x86_64",
}


def notices(destination):
    shutil.copytree(ROOT / "third-party-licenses", destination / "licenses")
    for name in ("THIRD_PARTY_NOTICES.md", "assets/README.md", "assets/sources.json", "assets/runtime.json", "assets/ui/owner-wallpaper.json", "assets/input/t9/manifest.json"):
        source = ROOT / name
        shutil.copy2(source, destination / ("t9-manifest.json" if name.endswith("t9/manifest.json") else source.name))
    for folder in ("assets/fonts", "assets/input"):
        for p in (ROOT / folder).iterdir():
            if p.is_file() and (p.suffix == ".txt" or "LICENSE" in p.name or "AUTHORS" in p.name):
                shutil.copy2(p, destination / p.name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=TARGETS)
    parser.add_argument("--no-build", action="store_true")
    args = parser.parse_args()
    target = args.target or next(line[6:] for line in run("rustc", "-vV").splitlines() if line.startswith("host: "))
    if target not in TARGETS:
        parser.error("Unsupported desktop target")
    version = check()
    if not args.no_build:
        subprocess.run(["cargo", "build", "--release", "--locked", "-p", "phone-desktop", "--target", target], cwd=ROOT, check=True)
    binary = ROOT / "target" / target / "release" / ("phone-desktop.exe" if "windows" in target else "phone-desktop")
    if not binary.is_file():
        parser.error(f"Missing binary: {binary}")
    name = f"TA1278-v{version}-{TARGETS[target]}"
    dist = ROOT / "target/dist"
    dist.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="package-", dir=ROOT / "target") as temporary:
        stage = Path(temporary) / name
        stage.mkdir()
        if "apple-darwin" in target:
            subprocess.run(["sh", "scripts/package-macos.sh", target, "--no-build"], cwd=ROOT, check=True)
            shutil.copytree(ROOT / "target/TA1278.app", stage / "TA1278.app")
        else:
            shutil.copy2(binary, stage / ("TA1278.exe" if "windows" in target else "ta1278"))
            (stage / "Notices").mkdir()
            notices(stage / "Notices")
        shutil.copy2(ROOT / "config/default.json", stage / "config.example.json")
        (stage / "README.txt").write_text(
            f"TA1278 {version} ({TARGETS[target]})\n\n"
            "Open TA1278.app / TA1278.exe, or run ./ta1278 on Linux.\n"
            "Choose a folder for phone storage on first launch. --storage-dir PATH also works.\n"
            "Arrows: navigate. Enter: confirm. F1/F2: softkeys. Home: home. # switches input.\n"
            "Messages and telephone are offline. MP3/WAV playback uses local files.\n"
            "Image/video decoding currently requires macOS.\n"
            "Linux needs ALSA, X11 or Wayland, a Vulkan/OpenGL driver, and a file-dialog portal/Zenity.\n"
            "See https://github.com/shiftz300/ta-1278 for controls, limits and source.\n",
            encoding="utf-8")
        archive = dist / (name + (".tar.gz" if "linux" in target else ".zip"))
        if "linux" in target:
            with tarfile.open(archive, "w:gz") as output:
                output.add(stage, arcname=name)
        elif "apple-darwin" in target:
            subprocess.run(["ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", str(stage), str(archive)], check=True)
        else:
            with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as output:
                for file in sorted(stage.rglob("*")):
                    if file.is_file():
                        output.write(file, str(file.relative_to(stage.parent)))
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n")
        print(json.dumps({"archive": str(archive), "sha256": digest}))


if __name__ == "__main__":
    main()
