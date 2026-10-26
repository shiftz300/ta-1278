#[path = "../../build/pack.rs"]
mod pack;

fn main() {
    let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    pack::pack(
        root.parent().unwrap().parent().unwrap(),
        &out,
        &[
            ("PINYIN", "assets/input/pinyin_simp.dict.yaml"),
            ("STROKE", "assets/input/stroke.dict.yaml"),
            ("ENGLISH", "assets/input/en_50k.txt"),
        ],
    );
}
