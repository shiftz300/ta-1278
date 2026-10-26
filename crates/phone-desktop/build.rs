fn main() {
    println!("cargo:rerun-if-changed=native/MediaHelper.swift");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let helper = out.join("ta1278-media");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");
    let arch = match std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "aarch64" => "arm64",
        "x86_64" => "x86_64",
        other => panic!("Unsupported macOS architecture: {other}"),
    };
    let deployment = std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "12.0".into());
    let status = std::process::Command::new("xcrun")
        .args(["swiftc", "-O", "-swift-version", "5", "-module-cache-path"])
        .arg(out.join("swift-cache"))
        .arg("-target")
        .arg(format!("{arch}-apple-macos{deployment}"))
        .arg("native/MediaHelper.swift")
        .arg("-o")
        .arg(&helper)
        .status()
        .expect("Apple Swift compiler is required for macOS media support");
    assert!(status.success(), "Could not build native media helper");
    println!("cargo:rustc-env=TA1278_MEDIA_HELPER={}", helper.display());
}
