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

## Checks

Run `cargo test --workspace --locked`. Browser regression tests use local synthetic HTML fixtures; no handset connection is required.
