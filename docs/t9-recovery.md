# T9 algorithm recovery

## Runtime integration — 2026-10-27

The desktop now uses the original ARM engine through `predict/firmware/arm.rs`, with the ABI bridge in `session.rs` and private dictionary ownership in `mod.rs`. The earlier static pass below is preserved as historical evidence; its statement that the desktop still uses replacement dictionaries is superseded.

`python3 scripts/extract-t9-engine.py` verifies user.bin, kernel.bin, both LDB hashes and the two kernel helper instruction sequences. It extracts only the engine range `0x880cc000..0x880e9000` and read-only character tables `0x884ac300..0x884acc00`. [Asset hashes](../assets/input/t9/manifest.json). No firmware image is opened at runtime.

| Operation | Original entry | Observed contract |
| --- | --- | --- |
| English init/language | `880e4e44` / `880e54a8` | Language `0109`, public LDB callback |
| Chinese init/language | `880d7718` / `880d7ba4` | Language `0204`, reconstructed block-pointer callback, four characters per page |
| Pinyin / stroke | `880d6e94` / `880d73d8` | Mode selection |
| Input / next page | `880cf39c` | Digit key IDs; 17 advances candidates; rejected sequence status 7 |
| Select spelling | `880d7330` | One-based spelling index, not a string pointer |
| Select Chinese character | `880cf34c` | UTF-16 character in the currently restored candidate page |
| Load preceding text | `880cc5dc` | UTF-16 buffer and length; host keeps at most 16 preceding scalar characters |
| English / Chinese user data | `880e2220` / `880d7884` | Activate 4096 / 110 bytes with wrapper argument 2 |
| Unsigned / signed divmod | `881009cc` / `88100844` | Verified UDIV/SDIV + MLS kernel veneers translated into bounded host arithmetic |

Chinese learning checks in `880d6520..880d65bc` require the selected character to be in the current page and apply native thresholds. Replaying only a character on the first page incorrectly skipped learning for later candidates. Restoring the spelling/page before selecting “酿” changes the AUDB and survives reload. Common first-page choices may correctly leave it unchanged. The implementation does not add host frequency sorting.

`python3 scripts/t9/check.py` generates [20 synthetic fixtures](evidence/t9-runtime-probe.json) with the independent Python interpreter. Rust tests compare their candidate output, and separately verify paging, syllable rejection, context associations, native user-data round trips, checksum rejection, ARM arithmetic/shift boundaries, memory limits and instruction budget. These are original-routine and local-interpreter checks, not new physical handset captures.

The interpreter has no syscall, device, JIT or native-code-loading path. Only LDB reads, known timer/language notification requests and the two arithmetic helpers are bridged. Timer notifications do not schedule the native UI timer. Unsupported requests fail. The shell/GUI and modern host IO are never available to guest code.

The integration covers the currently supported simplified Chinese, stroke and English input flows. It does **not** recover original C source, fully explain the trie/ranking implementation, include every language, prove every ARM opcode, or establish exhaustive physical-phone parity. Host candidate limits and explicit association-confirm/cancel behavior are described in [the interface](input-engine.md).

## Earlier static recovery snapshot

2026-10-26. Static inspection of the published system image, firmware 17.00.17.02. No handset operation, native-code execution or private user-dictionary read.

Run `python3 scripts/recover-t9-core.py`. It checks the complete user.bin SHA-256, 19 exact ARM instructions, six wrapper call targets and their log strings. Outputs: [structured evidence](evidence/t9-core-recovery.json), [Thumb wrappers](evidence/t9-core-thumb.txt), [ARM routines](evidence/t9-core-arm.txt). Wrapper code is Thumb; BLX calls enter an ARM engine. Treating both as Thumb produces incorrect instructions. Thumb ADR string addresses are decoded from bytes because LLVM's relative annotations here are misleading.

## Recovered rules

| Rule | Evidence | Limit |
|---|---|---|
| AW initialization clears LDB completion and custom-word completion | Calls at `88a780de`, `88a780ec`; ARM targets `880e56c8`, `880e55b8`; masks `0x10000`, `0x800` in field + `0x18` | The preceding helper and all validation side effects are not emulated |
| Chinese context search depth is initialized to 4, accepted maximum is 8 | Wrapper `88a78d0e`; ARM `880d7020..2c` stores a halfword at field + `0x3b0` | Guard/rebuild calls and depth units are not fully established |
| Context fence argument is half of a wrapper halfword | `88a78cfe` calls `880d71d0`; input is `u16(wrapper+0x340)>>1`; setter checks field + `0x31c`, writes + `0x3ae` | No complete context-search reconstruction |
| Runtime record-span calculation | Entire routine `880d929c..880d9314` | Not established as the static LDB trie decoder |
| Extended record length is big-endian and can cross a ring boundary | `880e6950..880e69a4` | The host payload helper adds safe bounds checks |
| English key alphabet groups | Both 36,651-byte LDB variants: ten length bytes at `0x76`, UTF-16LE symbols at `0x80..0xdd` | Decoded layout, not candidate order or multi-tap order |

For a tag byte, let `n=(tag & 63)+1`. The high two bits select:

- `00`: `n+3`
- `01`: `2*n+ceil(n/2)+3`
- `10`: `ceil(n/2)+5`
- `11`: `n`, except `0xff`, which reads the following big-endian uint16 and adds one modulo 65536.

`phone_core::predict::native` implements these bounded rules. Exhaustive tag/extended-length tests and ring-boundary tests cover the translation. They do not establish full T9 equivalence. `record_span_at` takes the ring payload, not a raw firmware/UDB file; it rejects zero or oversized spans.

The wrappers clear `0x5ac` bytes for AW and `0x814` for Chinese. These are clear lengths, **not total engine RAM use**. Allocations, dictionaries, callbacks and runtime buffers are additional and not fully measured.

## Remaining algorithm work

Candidate traversal, scoring/tie order, Chinese block decoding, user-dictionary lifecycle and complete context completion are still unresolved. Native branch inputs and runtime invariants must be recovered before applying the small routines to a whole database. The record-span helper alone cannot extract a word list or produce verified candidates.

The shipped predictor still uses independent dictionaries. Persistent selected-word/context weighting is a host implementation, not the recovered T9 ranking. The [input engine interface](input-engine.md) separates this replacement from editor/UI and service contracts so further native recovery can be integrated without rewriting them. Original input output still needs representative handset sequence comparisons.
