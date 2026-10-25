# Handset regression plans

These plans send keys to the original phone. They are not desktop replica tests.

## Browser plans

- `browser-select-cancel.json`: passed in session 261 (three steps, two assertions). Start on the loaded [Selenium web form](https://www.selenium.dev/selenium/web/web-form.html), with its single-select focused and set to Two, at the original top scroll position. Reopens the list, checks first-row focus with Two still marked, moves focus to One, cancels and verifies Two remains. No submission or upload.
- `browser-link-back.json`: passed in session 263 (two steps, two assertions). Start on [CERN's home page](http://info.cern.ch/), first link focused and page at top. Opens the first website link, checks the target body, then returns and checks the parent body/focus. It performs public GET requests. No prior private history is inspected.

Both require the reviewed Chinese UI and grey-green theme. A network delay, changed public page or changed viewport can stop the replay. Use the `run` command below with the chosen plan and a new output directory. Read the corresponding `*-result.json` for hashes. Do not replay from a different page or blindly restart after a guard failure.

## SMS plan

`sms-numeric-boundary.json` passed on firmware 17.00.17.02: 172 steps and seven image assertions. See [result](sms-numeric-boundary-result.json). It checks numeric mode, counts at 1/159/160/161 characters, clear text and return to pinyin.

Start with Chinese language, the observed grey-green theme, a new empty SMS with no recipient, body focus and pinyin input. The diagnostic connection must already be enabled as described in [capture setup](../handset-automation.md). Do not enable test mode casually: it changes other settings and fills contacts.

```sh
python3 scripts/handset_capture.py run \
  --profile docs/handset-profiles/ta1278-17.00.17.02.json \
  --plan docs/handset-plans/sms-numeric-boundary.json \
  --out /private/tmp/ta1278-regression-UNIQUE
```

The first action checks the empty-editor state. Each subsequent action checks its expected page region; seven steps also check their results. This route never sends a message or saves a draft. It clears only its generated body and leaves pinyin selected. A mismatch stops immediately; inspect the latest image before deciding how to recover. Do not restart blindly in a partly completed editor.

Exact JPEG-derived hashes require the same theme, language and layout. Some guards intentionally cover only a stable title for bounded typing/navigation; they are not a semantic guarantee against unrelated dialogs. Keep the handset untouched during a replay.

Raw sessions stay outside Git. To archive completed numbered captures:

```sh
PYTHONPATH=scripts python3 -m handset.archive_sessions \
  --through 93 --out /Users/shiftz/dev/ta-1278-source/handset-sessions/20261025
PYTHONPATH=scripts python3 -m handset.export_samples
```

Only explicitly reviewed, non-private frames are exported. Archive copies are hash-checked and existing different files are never overwritten.
