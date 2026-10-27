//! Bounded original-engine query timing through the same editor-facing API.
use phone_core::predict::{Language, Prediction};
use std::time::Instant;
fn main() {
    if std::env::args().any(|a| a == "--baseline") {
        println!("{{}}");
        return;
    }
    let mut report = vec![];
    for (language, name, digits) in [
        (Language::Chinese, "chinese", &[6, 4, 2, 6][..]),
        (Language::English, "english", &[4, 6, 6, 3][..]),
        (Language::Stroke, "stroke", &[1, 2, 3, 4, 5][..]),
    ] {
        let mut p = Prediction::default();
        p.set_language(Some(language));
        let start = Instant::now();
        p.push(digits[0]);
        let cold_ms = start.elapsed().as_secs_f64() * 1000.;
        let mut warm = vec![];
        for _ in 0..100 {
            p.clear();
            for digit in digits {
                let start = Instant::now();
                p.push(*digit);
                std::hint::black_box(p.current());
                warm.push(start.elapsed().as_secs_f64() * 1000.);
            }
        }
        warm.sort_by(f64::total_cmp);
        report.push(serde_json::json!({"language":name,"first_query_ms":cold_ms,
            "warm_queries":warm.len(),"warm_p50_ms":warm[warm.len()/2],
            "warm_p95_ms":warm[warm.len()*95/100],"warm_max_ms":warm.last()}));
    }
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
