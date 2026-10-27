# T9 cost and limits

The desktop now uses the original engine through a small bounded ARM interpreter. It no longer loads the replacement Rime/FrequencyWords dictionaries. This is reuse of the original routines, not a complete rewrite or semantic decompilation of every ranking branch.

| Bundled input data | Bytes |
| --- | ---: |
| ARM code and local constants | 118,784 |
| Read-only character tables | 2,304 |
| Simplified Chinese LDB | 135,848 |
| English LDB | 36,651 |
| Total before lossless compression | 293,587 (286.7 KiB) |

The four assets occupy **211,813 bytes after compression** in the current executable. The interpreter itself adds compiled Rust code. Each base session has 128 KiB guest RAM; three language/mode bases use 384 KiB. Query/page clones add temporary RAM, and UI candidates and decoded assets are additional allocations. This is not a measured whole-process RSS value or the native handset's RAM requirement.

One release arm64 timing run measured per-key P95 around **4.23 ms Chinese, 3.09 ms English and 4.59 ms stroke**. These common synthetic sequences are not a worst-case performance guarantee. See [raw timings](evidence/t9-runtime-cost.json). Reproduce with `cargo run --release --locked -p phone-core --example prediction_cost`.

Queries share a 3,000,000-instruction budget across candidate-page clones. Guest code cannot call arbitrary firmware/kernel routines, access host memory, files, sockets or devices, or generate host machine code. Invalid addresses, unsupported instructions and budget exhaustion stop the query. A fault is reported, not silently hidden by a different predictor.

Supported databases are simplified Chinese and one English variant from firmware 17.00.17.02. Traditional Chinese and other languages are not included. Native automatic selection learning is saved; a UI for arbitrary custom vocabulary is not included. Full native IME timer behavior, every language branch and exhaustive handset sequence parity remain unverified. [Detailed evidence](t9-recovery.md).

The older replacement-provider measurements remain in [prediction-cost.json](evidence/prediction-cost.json) as historical data, not current app memory or timing figures.
