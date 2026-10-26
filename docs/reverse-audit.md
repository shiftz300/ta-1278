# Reverse-engineering audit

2026-10-26. Scope: all existing published resource/application research and its use by the desktop replica. This does not claim recovery of every firmware function. No phone, private storage or external network was accessed.

## Rechecked evidence

The isolated resource pipeline completed all 14 stages. It rechecked five system components, 22,500 strings, 147 multi-tap tables, 22,349 glyph outlines, 2,824 image previews, 612 additional frames, 751 window fragments, 1,357 window names, 73 audio hashes, 479 DATA records, 55 colors, three rectangles and the 105-byte key table. Some previews and metadata remain interpretations, as stated in the original reports.

The application pipeline rebuilt 79 outputs byte-for-byte and passed 75 native-anchor/reference-model tests. It reused 63 archived public framework reference files only after checking their recorded hashes. Missing references are now supplied with `verify_app_reproduction.py --reference-root PATH`; their previous archival no longer requires copying them into the working repository.

Eleven focused checks covered current assets, Latin multi-tap, shared controls, contacts/torch, theme/style, alarms, Snake/maps/randomness, SMS lifecycle, original sound resources and browser byte anchors. [Exact results](evidence/reverse-audit.json).

## Evidence to implementation

| Area | Used in the replica | Explicit remaining boundary |
|---|---|---|
| Fonts | All three decoded original fonts take priority; deduplicated supplemental fonts fill missing characters without changing coverage | Native hinting/rasterizer is not recovered. Host fontdue renders outlines |
| Images/animations | Relevant original menu, softkey, control, torch and Snake art | Unused/region-specific resources are not all loaded. SABM channel interpretation and native animation timing remain uncertain |
| Colors/text | Recovered six theme palettes and application labels; black home background follows observation | Native dynamic resource selection and every regional string branch are not known |
| Shared windows/controls | Shared title, list, popup, form/editor and softkey primitives; measured native geometry | 751 descriptor byte checks do not equal an implementation of every original window or the MMK runtime |
| Keys/text input | UTF-16 positions, original Latin multi-tap data, short/held/released key separation | Host repeat timing, scalar-safe edits and replacement pinyin/stroke ranking remain deliberate policies; original T9 is excluded |
| SMS | Character classification/counting, menus, draft/queue save gating, retry causes, native operation labels and sampled conversations | No modem or native NV database. Real send/receive confirmation, segment scheduling and all interrupt branches are not emulated |
| Browser | Native menu order, page Back priority, field limits, select commit/cancel, basic forms and sampled settings | Engine selection, full DOM/JS, masks, auth/download backend and subresource lifecycle remain outside the text adapter; see [capabilities](browser-capabilities.md) |
| Music | Short release versus held seek, repeat modes, local real playback and bounded queue end behavior | Native review/media helper side effects, callback reentrancy and random permutation generation remain unresolved |
| Alarms | Seven weekday masks, five snooze choices, field order and sampled save/cancel behavior | Host persistence/scheduling and audio interruption policy; no OS wake service |
| Contacts/torch | Original control descriptors, torch table/art, sampled contact form/details/search layout | Search matching and local group/photo/ringtone management are host implementations; no LED backend |
| Calendar/utilities | Sampled navigation, fields, controls and local arithmetic/state | Complete native recurrence/formulas/recorder/FM implementations were not recovered; no claim of code parity |
| Snake | Native maps, speeds, starting geometry, collision/bonus/score rules, art and 55-word RNG update with two draws | Initial RNG state, full-board/overflow branches and exact scheduling/death-animation timing remain uncertain |
| Audio | Verified original SMFs, preset resources, click and SF2 samples, independent local synthesis | Some preset-name/event bindings are inferred; proprietary DSP/effects parity is not established |
| Acquisition/storage | Public system images checked; VFS and JSON state provide the agreed local experience | NAND/OOB recovery is archival evidence, not a desktop storage emulator. Personal NV is neither loaded nor reproduced |

## Corrections and checks in this pass

- Browser field construction now limits the initial text as well as subsequent input. The native XHTML copy is bounded by `min(length, limit)`; cancelling retains the original value. A UTF-16 surrogate split is avoided as an explicit host safeguard.
- Browser Save now follows the recovered four-item order: `作为主页`, `书签`, `网页`, `图像`. Setting the current page as home is implemented and persists. Image saving is disabled because the text adapter does not load images. The three working actions are tested after serialization/reload.
- Music shuffle with repeat off no longer plays indefinitely. The recovered natural-end mode is 1, and `0x8861b452` rejects queue wrap for modes 1/4. The host now traverses a finite shuffled order and stops at its end; repeat-all wraps and repeat-one stays on the current track. Fisher-Yates ordering and its seed are host policies, not a claim to the original random sequence. Manual skip remains a separate host path.
- The actual Rust SMS counter is cross-checked against the separately traced reference for all 63,488 valid BMP scalars plus 169 boundary/mixed-language cases. This closes the gap between “reference tests pass” and “the desktop counter matches that reference.” Surrogate-pair cases are included in the boundary set. The explicit encoding configuration remains `alphabet_management=1, sms_encode=0`.

Known native code is applied where the required state and service exist. Merely adding an unused copy of an authentication, modem or driver function would not reproduce its behavior. Those boundaries stay recorded above instead of being marked completed.

## Reproduce

```sh
python3 nokia-ta1278-research/tools/verify_app_reproduction.py --reference-root /path/to/archived/mmi-framework --report /tmp/app-reproduction.json
# Run tools/rebuild_offline_ui.py inside the isolated export reported by that command.
python3 scripts/check-browser-evidence.py
python3 scripts/generate-native-parity.py
cargo test --workspace --locked
```

Raw table/resource recovery commands also exist under `scripts/recover-*.py`. Use an isolated export for broad regeneration so historical capture/validation annotations are preserved. The generated SMS fixtures are test-only and are not embedded in the shipping application.

Package reduction is separate from fidelity: [measured composition and checks](build-size.md). Neither passing tests nor a smaller executable constitutes full physical-phone acceptance.
