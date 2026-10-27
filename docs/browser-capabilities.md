# Browser capabilities

2026-10-27. This is a small desktop text browser with phone controls and a bounded basic JavaScript adapter. It does not run the original Dorado or Opera Mini engine. No percentage of original-engine completion is claimed. See [JavaScript and security limits](javascript.md).

| Area | Current behavior | Boundary |
|---|---|---|
| Pages | HTTP/HTTPS HTML and plain text; relative links and redirects | Host network/TLS; no native proxy or APN stack |
| Display | Text wrapping, headings, lists, fieldsets, inline labels/fields and focused links; basic script-generated content | Simplified flow; no CSS layout, webpage images or WML |
| Navigation | URL editor, links, refresh, stop, Back/Forward, scrolling and focus | Stop ignores obsolete results; it does not immediately abort an active HTTP request. Spatial focus and history caching are host policies |
| Fields | Text/password fields, textarea, maxlength, readonly handling; disabled controls skipped | UTF-16 limit defaults/caps at 1024. Fields sync to basic JS before script activation. Native input masks and complete validation are not implemented |
| Choices | Checkbox, grouped radio, single/multiple select, form reset | Native select commit/cancel rules retained; complete DOM side effects are absent |
| Submission | GET forms, hidden fields, selected values and clicked submit button | No POST, upload, authentication UI or persistent login/cookie handling |
| Saved data | Bookmarks, history, offline pages and custom homepage | Offline saves parsed local documents, not an exact website archive; field values reset and passwords clear |
| Settings | Small/normal font, homepage, saved image-download toggle | Image toggle has no fetching backend; SIM APN entries are unavailable; Clear selects history/cache and commits locally |
| Page tools | Title/URL details | Share link prepares a new SMS draft, preserving unfinished text; no dispatch. Download manager has no backend. Save submenu follows the native order and supports setting home, bookmarks and offline pages; image saving is disabled |

Bounds: 10-second load/action network budget, 2 MiB decoded response, 4 MiB total, 16 requests, two waiting requests, 1200 parsed elements, parse depth 128, 32 Back entries, 100 persistent history entries, 20 offline pages and 1000 bookmarks. [Script limits](javascript.md) are stricter. These are host limits, not recovered handset capacities. In-memory navigation restores focus/scroll; reopening the process does not restore a live browser session.

Useful targets are static text pages, link directories, simple GET forms and basic script-built pages. Complex single-page applications and modern login sites remain outside the supported scope. A few public-page spot checks do not establish broad website compatibility.

## Checks

Run `cargo test --workspace --locked`. Browser regression tests use local synthetic HTML fixtures; no handset connection is required.
