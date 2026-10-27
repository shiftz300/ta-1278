# JavaScript

The native browser adapter uses pinned `rquickjs`/`rquickjs-sys` 0.14.0 and its bundled QuickJS-NG source. It evaluates JavaScript source, never downloaded bytecode. The phone renderer and keyboard controls remain in the runtime; the optional native adapter owns script contexts and HTTP.

Enabled by default. Disable it with `--disable javascript` or:

```json
{"capabilities":{"javascript":false}}
```

## Supported

Classic inline and external scripts, basic DOM creation and edits, `textContent`/`innerHTML`/`document.write`, simple tag/id/class/attribute selectors, `DOMContentLoaded`/load, bubbling click handlers, Promise jobs and short timers. Inputs are synchronized before the next script action; input/change handlers then run before click/submit. Buttons use the existing focusable link control. Normal GET form behavior and `preventDefault()` work.

The adapter supplies limited `location`, URL/URLSearchParams, GET `fetch` with text/JSON responses and asynchronous GET XMLHttpRequest. Script navigation is bounded. Text and controls are rendered from the resulting DOM snapshot.

Load and action phases settle short timers for about 180 ms; there is no continuous background page event loop. Two recent live contexts are retained. Older actions ask for a refresh. Offline saves retain readable text and ordinary HTTP(S) link destinations, and discard live script actions. Form defaults survive snapshots; saved password fields are cleared as before.

This is a small web API adapter, not a complete DOM or browser engine. ES modules, React hydration, complete selectors, CSS layout, images, storage/cookies, service workers, WebSocket, authentication flows, POST and uploads are unavailable. Some script-generated pages now work; support for most modern websites is **not** established.

## Security boundary

- No std/os module, file access, process execution, native module loading, phone storage, profile, messaging or call bindings. Hidden host functions are captured by the adapter and removed from the page global object.
- All requests use HTTP(S); URL credentials and other schemes are rejected. Page `fetch`/XHR are GET-only and same-origin, including every redirect. Custom headers and request bodies are unavailable. No cookies or host credentials are supplied. Classic external scripts may load across origins; HTTPS pages reject HTTP script URLs.
- A page declaring an HTTP or meta Content Security Policy runs no scripts. This conservative policy avoids pretending to implement CSP allowlists/nonces. Inline event handlers are subject to the same gate.
- Each QuickJS context has a 32 MiB heap and 512 KiB stack limit. Execution slices are interrupted after 250 ms; network waits are budgeted separately. DOM creation is capped at 6000 nodes/depth 64, strings/output at 2 MiB, timers at 32 and Promise draining at 128 jobs per batch. Host HTML parsing rejects excessive tag counts before building the tree. Buffers and Rust allocations are additional to the JS heap limit.
- Each load/action has a 10-second network budget, at most 16 requests and 4 MiB total decoded input. A response is at most 2 MiB. The first request allows up to 8 seconds; later requests allow up to 3 seconds, each bounded by the remaining budget. There are at most five script navigations and six hops per HTTP chain.
- Atomics and SharedArrayBuffer are removed: a native blocking wait would evade interpreter interrupts. Loading, scripts and HTTP run outside the window thread. Stop/new navigation cancels obsolete script work; an already blocking HTTP call finishes within its request timeout. Errors retain loaded/static content and show a partial-script notice.

These are capability and resource limits inside the application process. They are **not** an OS sandbox or Chromium-style process isolation. A native QuickJS vulnerability could affect the host process. Keep the pinned dependency updated; disable JavaScript for untrusted browsing when that boundary is insufficient. No claim of a security certification or complete engine vulnerability audit is made.

## Repeat

```sh
cargo test --locked -p phone-desktop web:: -- --test-threads=1
# Explicit localhost HTTP check; no public requests or audio:
cargo test --locked -p phone-desktop web::client::tests::script_http_click_navigation_disable_and_cancel -- --ignored --exact
# Optional public-page readability spot check, not a compatibility survey:
cargo test --locked -p phone-desktop web::client::tests::public_pages -- --ignored --exact --nocapture
```

Tests exercise DOM load, Promise/timers, field synchronization, delegated/inline clicks, forms, cancellation, disabled JS, CSP, cross-origin redirects, forbidden schemes/credentials, absent host bindings, infinite loops, catastrophic regex, recursive overflow, DOM exhaustion and heap exhaustion. `scripts/fixtures/browser-script.html` supplies a small manual page; serve `/data.json` as `{"text":"Fetched JSON content"}` and `/next.html` as ordinary HTML.

Sources: [rquickjs API](https://docs.rs/rquickjs/0.14.0/rquickjs/), [QuickJS-NG](https://github.com/quickjs-ng/quickjs), [upstream security boundary](https://github.com/quickjs-ng/quickjs/blob/master/SECURITY.md). Exact dependency checksums and licenses are retained in Cargo.lock and the packaged notices.

## Validation, 2026-10-27

Workspace: 224 passed, 7 opt-in checks skipped. The local HTTP script test and public readability test passed separately. The headless browser group passed 12 tests. Clippy with warnings denied, formatting, resource hashes and package signing passed.

The native window loaded the fixture, fetched JSON, changed an input, displayed the changed value after a click, followed a link, returned and continued using the retained context. This found and fixed replay of an already-consumed script navigation.

Example Domain, the Rust website and Hacker News returned readable pages in approximately 1.2, 3.1 and 1.4 seconds during this run. These three examples do not prove broad JS or website compatibility. The optimized executable increased from 18,845,280 to 19,480,416 bytes: 635,136 bytes (about 620 KiB, 3.4%).

Detailed synthetic logs and native screenshots are outside the app repository in `ta-1278-source/validation/javascript-20261027/`. No user media, messages or phone captures were used for these checks.
