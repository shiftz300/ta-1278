//! Build-time lossless resource packing, shared by the two consumers.
use std::{fmt::Write, path::Path};

pub fn pack(root: &Path, out: &Path, assets: &[(&str, &str)]) {
    println!(
        "cargo:rerun-if-changed={}",
        root.join("build/pack.rs").display()
    );
    let mut declarations = String::new();
    for (name, path) in assets {
        let source = root.join(path);
        println!("cargo:rerun-if-changed={}", source.display());
        let raw = std::fs::read(&source).expect("Read original resource");
        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&raw, 9);
        let restored =
            miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(&compressed, raw.len())
                .expect("Verify packed resource");
        assert_eq!(restored, raw, "Packing changed {}", source.display());
        std::fs::write(out.join(format!("{name}.z")), compressed).unwrap();
        writeln!(declarations,
            "pub const {name}: (&[u8], usize) = (include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{name}.z\")), {});",
            raw.len()).unwrap();
    }
    std::fs::write(out.join("packed.rs"), declarations).unwrap();
}
