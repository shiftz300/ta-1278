#!/usr/bin/env python3
"""Build and verify a locally signed ARM64 APK with the official SDK/NDK; no Gradle."""
import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
SDK_VERSION = "35"
NDK_VERSION = "28.2.13676358"
BUILD_VERSION = "35.0.0"


def run(*args, **kwargs):
    subprocess.run([str(a) for a in args], cwd=ROOT, check=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument("--check", action="store_true", help="Run Android Clippy before building")
    args = parser.parse_args()
    sdk = Path(os.environ.get("ANDROID_HOME") or os.environ.get("ANDROID_SDK_ROOT") or Path.home() / "Library/Android/sdk")
    ndk = Path(os.environ.get("ANDROID_NDK_HOME") or sdk / "ndk" / NDK_VERSION)
    host = {"darwin": "darwin-x86_64", "linux": "linux-x86_64"}.get(sys.platform)
    if host is None:
        raise SystemExit("Run this packaging script on macOS or Linux.")
    toolchain = ndk / "toolchains/llvm/prebuilt" / host
    tools = sdk / "build-tools" / BUILD_VERSION
    android = sdk / "platforms" / ("android-" + SDK_VERSION) / "android.jar"
    clang = toolchain / "bin/aarch64-linux-android26-clang"
    for p in (clang, tools / "aapt2", android):
        if not p.is_file():
            raise SystemExit(f"Missing {p}. Install SDK {SDK_VERSION}, build-tools {BUILD_VERSION}, NDK {NDK_VERSION}; see docs/android.md.")
    env = os.environ.copy()
    env.update({"ANDROID_NDK_HOME": str(ndk),
                "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER": str(clang),
                "CC_aarch64_linux_android": str(clang),
                "CXX_aarch64_linux_android": str(clang) + "++",
                "AR_aarch64_linux_android": str(toolchain / "bin/llvm-ar")})
    env["BINDGEN_EXTRA_CLANG_ARGS_aarch64_linux_android"] = f"--sysroot={toolchain / 'sysroot'} --target=aarch64-linux-android26"
    if sys.platform == "darwin":
        env.setdefault("LIBCLANG_PATH", "/Library/Developer/CommandLineTools/usr/lib")
    if args.check:
        run("cargo", "clippy", "--locked", "-p", "phone-desktop", "--lib", "--release",
            "--no-default-features", "--features", "android", "--target", "aarch64-linux-android",
            "--", "-D", "warnings", env=env)
    if not args.no_build:
        run("cargo", "rustc", "--locked", "-p", "phone-desktop", "--lib", "--crate-type", "cdylib",
            "--no-default-features", "--features", "android", "--target", "aarch64-linux-android", "--release",
            "--", "-C", "link-arg=-Wl,-z,max-page-size=16384", "-C", "link-arg=-Wl,-z,common-page-size=16384", env=env)
    library = ROOT / "target/aarch64-linux-android/release/libphone_runtime.so"
    out = ROOT / "target/android"
    work = out / "package"
    # Generated intermediates only; keep the local signing key across builds.
    if work.exists():
        shutil.rmtree(work)
    classes, dex = work / "classes", work / "dex"
    classes.mkdir(parents=True)
    dex.mkdir()
    java = Path(os.environ["JAVA_HOME"]) / "bin" if os.environ.get("JAVA_HOME") else None
    def jtool(name):
        return java / name if java else name
    run(jtool("javac"), "-source", "8", "-target", "8", "-Xlint:-options", "-bootclasspath", str(android) + os.pathsep + str(tools / "core-lambda-stubs.jar"),
        "-d", classes, ROOT / "android/src/dev/shiftz/ta1278/App.java")
    run(tools / "d8", "--lib", android, "--min-api", "26", "--output", dex, *classes.rglob("*.class"))
    version = subprocess.check_output([sys.executable, "scripts/version.py", "check"], cwd=ROOT, text=True).strip()
    major, minor, patch = map(int, version.split("."))
    raw = work / "raw.apk"
    run(tools / "aapt2", "compile", "--dir", ROOT / "android/res", "-o", work / "res.zip")
    run(tools / "aapt2", "link", "-I", android, "--manifest", ROOT / "android/AndroidManifest.xml",
        "--version-name", version, "--version-code", str(major * 1000000 + minor * 1000 + patch),
        "-o", raw, work / "res.zip")
    needed = subprocess.check_output([toolchain / "bin/llvm-readelf", "-d", library], text=True)
    with zipfile.ZipFile(raw, "a") as apk:
        apk.write(dex / "classes.dex", "classes.dex", compress_type=zipfile.ZIP_DEFLATED)
        apk.write(library, "lib/arm64-v8a/libphone_runtime.so", compress_type=zipfile.ZIP_STORED)
        if "libc++_shared.so" in needed:
            apk.write(toolchain / "sysroot/usr/lib/aarch64-linux-android/libc++_shared.so",
                      "lib/arm64-v8a/libc++_shared.so", compress_type=zipfile.ZIP_STORED)
        # Redistributed resources and libraries retain their notices.
        for base in ("third-party-licenses", "assets/fonts"):
            for p in sorted((ROOT / base).rglob("*")):
                if not p.is_file() or (base == "assets/fonts" and p.suffix != ".txt"):
                    continue
                apk.write(p, "assets/notices/" + str(p.relative_to(ROOT)), compress_type=zipfile.ZIP_DEFLATED)
        for name in ("THIRD_PARTY_NOTICES.md", "assets/README.md", "assets/sources.json", "assets/ui/owner-wallpaper.json", "assets/input/t9/manifest.json"):
            apk.write(ROOT / name, "assets/notices/" + name, compress_type=zipfile.ZIP_DEFLATED)
        for p in sorted((ROOT / "assets/input").iterdir()):
            if p.is_file() and ("LICENSE" in p.name or "AUTHORS" in p.name or p.suffix in (".md", ".txt")):
                apk.write(p, "assets/notices/" + str(p.relative_to(ROOT)), compress_type=zipfile.ZIP_DEFLATED)
    aligned = work / "aligned.apk"
    run(tools / "zipalign", "-P", "16", "-f", "4", raw, aligned)
    key = out / "debug.keystore"
    if not key.exists():
        run(jtool("keytool"), "-genkeypair", "-keystore", key, "-storepass", "android", "-keypass", "android",
            "-alias", "androiddebugkey", "-keyalg", "RSA", "-keysize", "2048", "-validity", "10000",
            "-dname", "CN=Android Debug,O=Android,C=US")
    result = out / f"TA1278-v{version}-android-arm64.apk"
    run(tools / "apksigner", "sign", "--ks", key, "--ks-pass", "pass:android", "--key-pass", "pass:android",
        "--out", result, aligned)
    run(tools / "apksigner", "verify", "--verbose", result)
    run(tools / "zipalign", "-c", "-P", "16", "4", result)
    digest = hashlib.sha256(result.read_bytes()).hexdigest()
    result.with_suffix(".apk.sha256").write_text(f"{digest}  {result.name}\n")
    print(f"Ready: {result} ({result.stat().st_size} bytes). Local development signature; not a store release.")


if __name__ == "__main__":
    main()
