use phone_core::sms;

#[test]
fn sms_matches_traced_rules_for_every_bmp_scalar() {
    let expected = phone_core::embedded::decode((
        include_bytes!("fixtures/sms-native-bmp.z"),
        (65536 - 2048) * 2,
    ))
    .unwrap();
    for (i, character) in (0..65536).filter_map(char::from_u32).enumerate() {
        let count = sms::count(&character.to_string());
        assert_eq!(
            (count.remaining, count.ucs2, count.segments),
            (expected[i * 2] as usize, expected[i * 2 + 1] != 0, 1),
            "U+{:04X}",
            character as u32
        );
    }
}

#[derive(serde::Deserialize)]
struct Case {
    #[serde(default)]
    text: String,
    #[serde(default)]
    pattern: String,
    #[serde(default)]
    repeat: usize,
    remaining: usize,
    segments: usize,
    ucs2: bool,
}

#[test]
fn sms_matches_traced_rules_at_boundaries_and_language_combinations() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/sms-native-boundaries.json")).unwrap();
    for (i, case) in cases.iter().enumerate() {
        assert_eq!(
            sms::count(&format!(
                "{}{}",
                case.text,
                case.pattern.repeat(case.repeat)
            )),
            sms::Count {
                remaining: case.remaining,
                segments: case.segments,
                ucs2: case.ucs2,
            },
            "reference case {i}"
        );
    }
}
