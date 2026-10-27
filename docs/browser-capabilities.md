# Browser capabilities

2026-10-26. This is a small desktop text browser with phone controls. It does not run the original Dorado or Opera Mini engine. No percentage of original-engine completion is claimed.

| Area | Current behavior | Boundary |
|---|---|---|
| Pages | HTTP/HTTPS HTML and plain text; relative links and redirects | Host network/TLS; no native proxy or APN stack |
| Display | Text wrapping, headings, lists, fieldsets, inline labels/fields and focused links | Simplified flow; no CSS or JavaScript engine, webpage images or WML |
| Navigation | URL editor, links, refresh, stop, Back/Forward, scrolling and focus | Stop ignores obsolete results; it does not immediately abort an active HTTP request. Spatial focus and history caching are host policies |
| Fields | Text/password fields, textarea, maxlength, readonly handling; disabled controls skipped | UTF-16 limit defaults/caps at 1024. Native input masks, JS callbacks and complete validation are not implemented |
| Choices | Checkbox, grouped radio, single/multiple select, form reset | Native select commit/cancel rules retained; complete DOM side effects are absent |
| Submission | GET forms, hidden fields, selected values and clicked submit button | No POST, upload, authentication UI or persistent login/cookie handling |
| Saved data | Bookmarks, history, offline pages and custom homepage | Offline saves parsed local documents, not an exact website archive; field values reset and passwords clear |
| Settings | Small/normal font, homepage, saved image-download toggle | Image toggle has no fetching backend; SIM APN entries are unavailable; Clear selects history/cache and commits locally |
| Page tools | Title/URL details | Share link prepares a new SMS draft, preserving unfinished text; no dispatch. Download manager has no backend. Save submenu follows the native order and supports setting home, bookmarks and offline pages; image saving is disabled |

Bounds: 12-second request timeout, 2 MiB decoded response, two waiting requests, 1200 parsed elements, parse depth 128, 32 Back entries, 100 persistent history entries, 20 offline pages and 1000 bookmarks. These are host limits, not recovered handset capacities. In-memory navigation restores focus/scroll; reopening the process does not restore a live browser session.

Useful targets are static text pages, link directories and simple GET forms. Script-rendered pages and modern login sites are outside the supported scope. No live public-site compatibility survey was performed in this audit.

## Firmware check

`python3 scripts/check-browser-evidence.py` checks the published `user.bin` hash, 13 reviewed browser code regions and 3037 listed instructions against the original bytes. It does not execute firmware. [Machine-readable results](evidence/browser-code-audit.json).

| Rule | Native evidence | Replica check |
|---|---|---|
| Initial field length | XHTML constructor `0x88b48e48`; clamp `0x88b48ef2`; initial copy `0x88b48fa0..0x88b48fb2` | Fixed initial values exceeding maxlength. Previously only later edits were limited. Cancel keeps the page's old value; confirm writes the bounded value |
| Single/multiple select | `0x88b487c4`; WML `0x88b48350` | Single confirm commits and Back cancels; multiple confirm toggles and Back commits |
| Page Back priority | `0x88b62486` | Stop loading first, otherwise Back history, otherwise ask to close |
| Checkbox/radio/reset | `0x880bed86`, `0x880bea84` | Local value/group/reset operations exist. Callback order, vetoes, unusual select-reset fallback and DOM lifetime are not fully reproduced |
| Subresource progress | `0x88b65986`, `0x88b62c68` | Original uses resource-count progress. Replica loads a single document and has no subresource pipeline; no equivalence claim |

These code anchors are the Dorado branch. The firmware also contains an Opera Mini launcher; this audit does not establish the handset's runtime engine-selection setting. Physical capture evidence remains separately recorded in [browser layout](browser-layout.md).

The initial-copy fix preserves valid Unicode characters if a UTF-16 limit would split a surrogate pair. Native code copies raw units; this small host difference is deliberate. Generic editor initialization is unchanged, so loading existing SMS drafts is unaffected.

Remaining fidelity gaps include native URL conversion/validation, input-mask behavior, authentication, download lifecycles, exact focus/scroll geometry, exact native clearing/reset branches and native callback semantics. The earlier research reference model implements some rules that the desktop adapter does not; its passing tests alone do not prove that the desktop has those features.

## Repeat checks

```sh
python3 scripts/check-browser-evidence.py
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The added initial-value test covers ASCII, Chinese, supplementary Unicode, edit limits, cancel and confirm using the actual phone route. Existing tests cover select behavior, Back/Forward, stale requests, settings, form values and offline persistence. This is code and local-model validation, not a new physical-phone or live-network test.
