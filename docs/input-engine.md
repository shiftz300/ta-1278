# Input engine

The desktop now uses `predict::firmware::Firmware` by default. It interprets the original T9 ARM routines and reads the extracted simplified Chinese and English databases. No firmware binary is loaded as host code. Recovery, addresses and limits (research archive).

The shared boundary remains independent of message and service backends:

```rust
pub trait Engine: std::fmt::Debug + Send + Sync {
    fn query(&self, language: Language, digits: &str, context: &str) -> Vec<Candidate>;
    fn learn(&self, language: Language, digits: &str, context: &str, candidate: &Candidate) {}
    fn strict(&self) -> bool { false }
    fn associations(&self, language: Language, context: &str) -> Vec<Candidate> { vec![] }
}
```

Attach an `Arc<dyn Engine>` with `Prediction::set_engine`. The desktop shares its provider across active editors. Providers do no network or disk IO. Unknown native key sequences are rejected with the previous composition retained; they do not silently fall back to another dictionary. Runtime faults are bounded and reported in the status line.

Chinese input uses a syllable at a time: `64`, confirm the spelling and character, then `426` and confirm gives “你好”. Candidate order, spelling groups, four-character pages and following-character suggestions come from the original routines. The host collects up to 256 candidates with a fair per-spelling page allowance. English enumeration stops at 64 candidates or the first repeated result. These host bounds can omit deeper native results.

Explicit candidate confirmation can open association suggestions. Sending, moving the cursor, switching modes, inserting host text and cancelling do not implicitly accept a suggestion. Associations are not preview text or part of the SMS character count. A new numeric sequence starts a fresh composition. Failed length-limit commits retain the current candidate and text. Password editors suppress learning, context and associations.

## Persistence

`input-native.json` stores schema version 1 and the synthetic English 4096-byte and Chinese 110-byte user dictionaries. These sizes follow the inspected wrapper calls. The engine itself updates them when its learning rules apply; selecting a common first-page character need not change anything. Restored dictionaries are checked by the original activation routines, and unexpected reset/checksum repair is rejected rather than silently replacing the file.

`Firmware::pending_save` returns data and a revision. `saved(revision)` acknowledges only that revision after an atomic write; newer changes remain dirty. The desktop uses the same backups, retry and Quit guards as other private records. No handset user dictionary is imported. These files and backups contain private learning data and stay outside the repository.

The previous `input-learning.json` is retained untouched. Its independent frequency/context weights cannot be converted into native AUDB/UDB records safely. `Builtin` remains an optional replacement provider for callers and regression tests; it is no longer constructed by the desktop. Its dictionaries are consequently removed from the release executable by dead-code elimination.

## Host keyboard

System IME is disabled for the entire window, and IME events are ignored. Physical key positions produce US-English letters and symbols, with Shift for uppercase/symbols; OS event text, dead-key composition and Option/Alt text input are not consumed. Digits still use the selected internal phone input mode. # (Shift+3 or a dedicated keypad hash key) uses the phone key path to cycle 拼 → 笔 → Abc → abc → ABC → 123. F4/F5 do not switch input modes. English nine-key prediction remains available through the engine API.
