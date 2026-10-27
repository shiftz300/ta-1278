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
            ("T9_CODE", "assets/input/t9/engine.arm"),
            ("T9_TABLES", "assets/input/t9/character-tables.bin"),
            ("T9_ENGLISH", "assets/input/t9/english.ldb"),
            ("T9_CHINESE", "assets/input/t9/chinese.ldb"),
        ],
    );
}
