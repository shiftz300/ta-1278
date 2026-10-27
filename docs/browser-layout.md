# Browser and editor layout

2026-10-25, handset firmware 17.00.17.02. Colors were retained. Layout and key routes were checked against new handset captures, not just firmware names.

## What changed

- HTML labels and fields share a line. Paragraphs, headings, lists, wrapped links and fieldsets keep their document order. Painting and focus scrolling use one geometry result.
- Browser body text uses the observed small/normal choices. The title strip shows the URL. Menus cover a dimmed page instead of replacing it with an empty screen.
- Settings have grouped, scrolling rows. Home type has one choice, `自定义`. Font size has two choices, `小号字体` and `普通字体`. Images toggle in place. Back from a picker returns to its parent row.
- URL and field editors place the underline and input-mode marker below the text. Footer icons have measured pixel geometry. The SMS recipient silhouette is a fitted replacement, not a recovered asset.
- Browser exit uses Center; the handset left softkey did nothing in the same dialog. Page-close confirmation has not received that same physical-key comparison.
- Clear contains `历史` and `缓存`, with cache checked in this handset's initial popup. The old history/bookmarks/offline list was wrong and was removed. Entry and cancel are reproduced. The desktop now toggles each checkbox with Center and commits with the left softkey: history clears the stored list; cache clears back/forward documents and pending load while preserving the visible page, bookmarks and offline saves. This commit rule is a local policy; native commit remains untested. No handset data was cleared.
- The final settings group and action read `恢复预设设置`. Native activation was not tested. The replica's cancel/confirm reset remains a host behavior.
- SMS now dims the inactive body, puts numeric mode beneath the recipient field, and uses the observed underline, bubble tail and footer positions.

## Evidence and comparisons

[Machine-readable evidence](evidence/browser-layout-20261025.json), [native observations](handset-observations.md), [coverage ledger](handset-coverage.json).

| Native / replica pair | Purpose |
|---|---|
| [HTML form](comparisons/browser-native-form-comparison.png) | Inline fields, radio rows and fieldset border |
| [Links](comparisons/browser-native-links-comparison.png) | Heading, wrapping and link focus |
| [Root menu](comparisons/browser-root-popup-comparison.png) | Page retained behind the menu |
| [Page menu](comparisons/browser-page-popup-comparison.png) | Six visible rows and submenu markers |
| [Font picker](comparisons/browser-font-picker-comparison.png) | Popup bounds and opaque panel |
| [Clear picker](comparisons/browser-clear-picker-comparison.png) | Actual choices; local commit policy now implemented |
| [Settings tail](comparisons/browser-settings-tail-comparison.png) | Section spacing and restore label |
| [URL editor](comparisons/browser-url-input-comparison.png) | Text, underline and input mode |
| [Exit](comparisons/browser-exit-comparison.png) | Dimmed page, prompt and Center action |
| [SMS recipient](comparisons/sms-empty-recipient-comparison.png) | Inactive body and recipient mode |
| [SMS body](comparisons/sms-empty-comparison.png) | Bubble, tail, count and softkeys |

Regenerate after the window smoke run:

```sh
python3 scripts/compare-handset-layout.py --replica /tmp/ta1278-smoke --out docs/comparisons
```

The form and CERN fixtures are reduced local examples based on public test pages already sampled on the handset. Links and submission targets are replaced with `about:` routes. Their original host captures and SHA-256 records are in the external archive and `docs/evidence/browser-test-pages.json`. This does not prove the handset received identical HTML bytes.

## Limits

This is still a small HTML adapter, not the original browser engine. CSS, JavaScript, WML, images, downloads, POST and complete network/error flows are missing. Font hinting, round controls, some spacing and icon details still differ. Native radio icons are not copied into the host's offline status bar. Cache state, APN values, current homepage and active theme are observed device settings, not proven factory defaults.

The 43-scene smoke test and unit tests check the replica. They do not establish full handset parity. See [remaining work](fidelity.md).

The later [equal-content SMS check](sms-composer-layout.md) corrects body font metrics, count baseline, caret height and scrolling. Its four fixtures are a more precise editor comparison than the earlier empty/different-text pairs above.

Current implementation details and the later firmware-byte audit: [browser capabilities](browser-capabilities.md).

## Popup edge correction

Compared again with handset `ta1278-native-106/page-options-down-3.png` and `ta1278-native-212/browser-page-options.png`: the right vertical scrollbar is present on the handset. It occupies x=236..239; the menu rows end at y=275, with black padding through y=281. The replica now covers that padding instead of exposing the dimmed webpage as a horizontal strip, and aligns the scrollbar to the right edge.
