//! Developer acceptance for the derived fallback fonts; no original binaries shipped by this test.
use super::*;
use std::path::Path;

#[test]
#[ignore = "Exhaustive fallback glyph raster comparison; run after font subset changes"]
fn fallback_subsets_preserve_every_visible_glyph() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let font = |path: &str| {
        Font::from_bytes(
            std::fs::read(root.join(path)).unwrap(),
            FontSettings::default(),
        )
        .unwrap()
    };
    let original = [
        font("assets/fonts/device/font-0-latn.otf"),
        font("assets/fonts/device/font-1-hani.otf"),
        font("assets/fonts/device/font-2-hani.otf"),
    ];
    let before = [
        font("assets/fonts/NotoSansCJKsc-Regular.otf"),
        font("assets/fonts/NotoSansSymbols2-Regular.ttf"),
    ];
    let after = [
        font("assets/fonts/subsets/FallbackCJK.otf"),
        font("assets/fonts/subsets/FallbackSymbols.ttf"),
    ];
    for (before, after) in before.iter().zip(&after) {
        for size in [18., 20., 24.] {
            assert_eq!(
                before.horizontal_line_metrics(size),
                after.horizontal_line_metrics(size)
            );
        }
    }
    let mut compared = 0;
    for ch in (0..=0x10ffff).filter_map(char::from_u32) {
        if original.iter().any(|f| f.lookup_glyph_index(ch) != 0) {
            continue;
        }
        let b = before.iter().position(|f| f.lookup_glyph_index(ch) != 0);
        let a = after.iter().position(|f| f.lookup_glyph_index(ch) != 0);
        assert_eq!(b, a, "Fallback selection changed for U+{:04X}", ch as u32);
        if let Some(i) = b {
            for size in [18., 20., 24.] {
                assert_eq!(
                    before[i].rasterize(ch, size),
                    after[i].rasterize(ch, size),
                    "Glyph changed for U+{:04X} at size {size}",
                    ch as u32
                );
            }
            compared += 1;
        }
    }
    println!(
        "Compared {compared} visible fallback characters at 18, 20 and 24 px; all metrics/bitmaps identical."
    );
    assert!(compared > 20000);
}
