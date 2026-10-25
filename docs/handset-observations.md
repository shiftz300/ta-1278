# Live handset observations

2026-10-25, TA-1278, firmware 17.00.17.02. **Sampling in progress; not complete reverse engineering or replica acceptance.**

## Connection and evidence

Normal charging mode had no phone data interface. The owner confirmed the native `#*8378#8#` menu opened and explicitly accepted its test-mode side effects before selecting Open. After reconnecting, USB `2e04:0019` exposed four vendor interfaces. Interface 1 uses bulk IN `83`, OUT `04`, packet size 512. Native UCOM open was required before a version reply; version, screenshot and remote-key requests then worked.

`recover-runtime-entry.py` exports the native entry table and side-effect paths. `recover-lcd-diagnostic.py` exports screenshot/key handlers and UCOM control. Original firmware is unchanged. The **phone's test settings and contacts have changed**: this is not an untouched factory session. Test mode remains enabled for capture.

The JPEG prefix sometimes arrives intact and sometimes damaged. Only bounded, verified header reconstruction is used. Raw responses and failed attempts are retained outside the repository. No firmware or captured user data was uploaded during this sampling session.

## Observed interactions

| Area | Actual observation | Remaining work |
|---|---|---|
| Remote input | Both softkeys, directions, center, digits 0–9, star and hash produce visible expected transitions. Multi-step direction replay and image guards work. A blinking cursor caused one guard failure, correctly preventing a key. | Physical held-key equivalence, timing and global exit keys. |
| Main menu | Three-column icon grid with circular focus. A complete rightward cycle has 18 top-level entries; see the inventory below. | Other submenu, vertical edge behavior and alternate menu modes. Current wallpaper is personal state, not evidence of defaults. |
| SMS layout | Recipient at top, body bubble grows upward from the lower screen, counter below, black background. | Recipient validation, full draft lifecycle and selection. Long text now has boundary samples. |
| Pinyin | `6` shows `o/n/m`; `64` shows `ni/mi/ng`. Center first moves from syllables to character candidates; a second Center commits `你`. A four-row association popup follows. | A later 64 run committed 呢 instead; candidate ordering is not proven fixed by one sample. Candidate paging and full navigation remain. The replacement dictionary need not reproduce original T9 internals. |
| SMS count | Preedit `ni` gives `158/1`; committing `你` gives `69/1`; newline gives `68/1`. Deleting newline returns to `69/1`; deleting the character restores `160/1`. | 160 numeric characters: 0/1; 161: 145/2. A Chinese character plus digits: 70 UTF-16 units gives 0/1; 71 gives 63/2. GSM extension boundaries remain. |
| SMS menus | Options: insert, attachments, remove attachments (disabled here), clear text, save draft. Insert submenu: smiley, symbol, template. Smiley picker is 4 columns × 3 rows; cancel returns directly to the editor. | Template picker and insertion now observed; attachment and exit-dialog paths remain. |
| Symbols | Star opens a 7-column grid; newline is bottom-right. Five Down then six Right selected it; Center inserted a newline. Bubble increased by one text row and shrank after deletion. | Grid wrap and all symbols. |
| Input modes | Short hash cycles pinyin → stroke → Abc → abc → ABC → 123 → pinyin. | Three rapid 2 taps give C in Abc; Right commits. Long hash, other tap tables and timing thresholds remain. |
| Browser | Start page: enter URL, bookmarks, history, offline pages. URL editor starts with `http://`; right soft key deletes, then becomes Cancel when empty. | Loading, forms and Back gained post-SIM evidence below. Select controls, downloads and error cases remain. No existing history was opened. |
| Browser menus | Start options: home, settings, close. Exit prompts “是否关闭浏览器？”. Settings have eight selectable rows; downward focus wraps after the last. | Dropdown choices and dialogs; configuration was not changed. |
| Browser settings | SIM1/SIM2 access points, home type, custom home, download images, font size, clear, restore defaults. Section headings do not take focus. | Clear/restore actions intentionally not executed on existing data. |

All synthetic SMS tests had no recipient and were not sent. Early test characters and boundary-test bodies were cleared before leaving. One later template-only Save as draft action was exercised; persistence remains unverified, as noted below. Existing inbox previews appeared on the app's root page; those screenshots are private local debugging material and are excluded from repository samples. Do not publish personal wallpaper, message previews, history or media filenames.


## Main menu and settings

The observed row-major order is: calendar, alarm, Snake, browser, music, radio, messages, call history, files, flashlight, settings, contacts, recorder, calculator, Other, counter, Alipay, Ximalaya. Right traversed all 18 and wrapped, observed twice in session 059. Alipay and Ximalaya remain owner exclusions. Other may contain additional apps; compiled registrations absent at top level are not proven absent from the device.

Settings > personalization contains greeting, sound, lock-screen background, theme and keyboard. The theme page has six horizontal swatches in purple/red-orange/orange/green/grey-green/blue order. Preview backgrounds stay black; focus rings change color. Left/right clamp at the ends. Canceling a purple preview restored the prior grey-green selection without confirming a change. Keyboard contains speed dial and lock-screen shortcuts. No theme change was saved.

## Additional SMS and music observations

- A long numeric body grows upward to the status bar and scrolls, keeping the counter below it. Unicode 70/71 and numeric 160/161 boundaries match the recovered counting function.
- A diagnostic right-soft-key down/up held for 1000 ms removed **one** character. This does not prove a physical long press has identical repeat behavior.
- Disabled menu rows still receive focus. Clear text immediately clears this test body and closes the menu; no confirmation appeared. In an empty body, Clear text and Save as draft are grey.
- Template insertion returned directly to the editor with `现在无法通话。我将稍后呼叫。` and count 56/1. Selecting Save as draft exited to the information list after asynchronous processing. A later New message opened empty. **Persistent storage and reopening this no-recipient draft are not yet verified**; the save action alone is not proof of a durable draft. No existing message was altered.
- Moving Up from the first body row transfers focus to the recipient. Down re-enters the body at the remembered first-row cursor in this case. The body showed pinyin again after this focus round trip; numeric input did not remain selected.
- Music opens a refresh page, then a root with Play all, Songs and Playlists. The root's options contain Manual refresh. Selecting a song started playback; Center paused it at 13 seconds. Player options contain Repeat, Shuffle, Queue, Add to playlist, Set as ringtone and Share. Repeat offers Off, All songs and Current song. Cancel returns to player options; backing out of the player returns directly to the music root with a current-track card. Playback remains paused. No media file was copied.

Completed numbered sessions are retained in the private external archive. Reviewed public images are listed in [sample index](handset-samples/20261025/index.json). Main-menu wallpaper, inbox previews, media names and raw damaged prefixes are excluded from those images.

## Browser loading and page state (before SIM insertion)

The reviewed public URL `http://example.com` was submitted once. It opened a near-white content area with an address bar, distinct from the black start/settings pages. No page title or body appeared during roughly three minutes of observation. Page details showed an untitled page and the expected URL. Navigation's Stop action was later grey. There was no captured explicit error dialog or evidence of successful HTML rendering. Test mode and the current radio state limit conclusions; this is not proof that the browser cannot render websites. No existing history or bookmarks were opened.

The URL editor wraps Right at the end to the beginning, and Left at the beginning to the end. Using Right as a generic multi-tap commit misplaced later letters; the wrong URL was never submitted. Waiting 1500 ms after the final tap of each character produced the correct URL. This is a tested sufficient delay, not a measured minimum timeout.

Page options have eight rows: Open, Navigation, Save, Download manager, Share link, Page details, Settings, Close. Down wraps. Open contains Home, Enter URL, Bookmarks, History, Offline pages; cancel returns to its parent menu. Navigation contains Refresh, Stop, Forward, Back, with availability reflecting this empty page. Page details is a separate view; cancel returns directly to the page. The page's right soft key prompts “关闭当前页面？”; confirmation returns to the start page. Exiting the start page separately prompts “是否关闭浏览器？”. No settings were reset, and no existing browsing data was cleared. The public test URL may now be in the phone's history.

## Post-SIM browser sampling

The owner inserted a working SIM, rebooted and reconnected. Session 204 shows SIM 2 registered on China Telecom with 4G/VoLTE. The diagnostic interface and version gate still worked without repeating test-mode Open. No APN or network setting was changed. This resolves the earlier missing-content blocker; that earlier session does not establish a browser rendering defect.

- Sessions 208–209: `http://example.com` loaded readable text. The host-fetched HTML currently contains a paragraph but no heading or link, so absent links on this page are not evidence of missing browser support. Host response hashes and their limits are recorded in [test-page sources](evidence/browser-test-pages.json).
- Sessions 215–236: `http://httpbin.org/forms/post` rendered text inputs, radio buttons, checkboxes, a time field and a textarea. Center on a text/tel/textarea field opens a separate black editor titled 编辑, initially in pinyin. Left soft key commits locally. Right soft key cancels an empty editor; with text it deletes. Only synthetic `test` and `test\na` were entered; the form was never submitted.
- Radio focus is orange; the selected value has a black inner dot. Selecting Medium clears Small. Checkbox focus is orange and the selected state has a tick; Bacon and Extra Cheese remain selected together. Focus and value are separate states.
- Down moves between controls and scrolls to reveal them. Some presses scroll while keeping focus on the current control (session 232). This is not simply one control per Down press. The top address strip and bottom softkeys stay fixed.
- The textarea editor accepted a newline through three quick 0 taps in lowercase mode. The page's compact preview showed only `test`, its first line. Retention after reopening the editor was not tested. A time field was focused but its editor/validation were not tested. One initial Down after a long idle was acknowledged without moving focus (219); the next Down moved it (220). Cause is unproven.
- Session 237: right soft key returned from the form to Example Domain, with no close confirmation. **Back takes precedence when page history exists**; the earlier close confirmation was observed with no prior page. This corrects an overgeneralization of the first sampling pass. Existing browsing history was not opened.
- Sessions 241–249: `http://info.cern.ch/` loaded a heading, bullets and links. Unfocused links are blue/underlined; the focused link is orange with white text. Center opened the first link, `http://info.cern.ch/hypertext/WWW/TheProject.html`. Highlighting spans wrapped link text. Back restored the first link's focus on the parent page. Navigation > Forward restored the child page. Disabled Stop still received focus. During loading, the prior body remained briefly visible beneath the new address and stop indicator (241); 242 captured the loaded body.

- Sessions 253–261: the Selenium public web form redirected from HTTP to HTTPS and rendered; the handset showed a lock icon. This is evidence for this page loading, not a security audit or proof of broad TLS compatibility. Disabled/readonly inputs were grey. Down traversed text, password, textarea, readonly and the index link before the select; the disabled input was skipped in this route.
- Its single-select opens a full-screen black list titled 选择, using the system accent rather than the webpage's orange. Center on Two immediately returns to the webpage with Two selected. Reopening puts the **focus on the first row**, while the radio mark remains on Two. Moving focus to One then canceling preserves Two. Sessions 260 and 261 have passing image assertions for this. Multi-select behavior is still firmware-derived, not live-tested.
- Two new replay plans passed: [select cancel](handset-plans/browser-select-cancel-result.json), three steps/two assertions; [link and Back](handset-plans/browser-link-back-result.json), two steps/two assertions. These have specific public-page and focus preconditions; they are not unattended tests from arbitrary handset state.

These observations are native handset evidence, not an assertion that the desktop implementation already matches. Multi-select, detailed layout metrics, browser errors, downloads, offline pages, persistent history and full network behavior still require further sampling. All test pages were closed and the browser exited in session 267. Public URLs may remain in history; it was not cleared. No form was submitted, file uploaded, call placed or SMS sent.

## Other native applications

These are partial routes, not full app acceptance. Public screenshots exclude original alarms, original contact lists, call records and personal files.

| Application | Observed route | Limits / final state |
|---|---|---|
| Other | Timer, stopwatch, flashlight, unit converter. | These entries are additional to the top-level registry. |
| Timer | Time and name editor, observed value 00:01:00. | No timer started; original/default value distinction unknown. |
| Stopwatch | Start from zero, left-soft-key lap, Center stop, left-soft-key reset. Running view offers Stop; stopped view offers Resume. | Reset to zero; no lingering stopwatch. |
| Unit converter | Temperature, weight, length, area, volume, currency. Temperature pairs: Fahrenheit/Celsius, Kelvin/Fahrenheit, Kelvin/Celsius. Down changes input field. | 32°F displayed -0.0000002222°C in this build. Left soft key exits the conversion view; it is not a clear-all action. Other formulas untested. |
| Calculator | Up=add, Right=multiply; `1+2=3`, then `3×4=12`; right soft key resets completed result to zero. | Left=divide, Down=subtract, Star=sign and Hash=decimal are shown on the keypad graphic but not all executed. Restored zero and exited. |
| Counter | Empty view: “这里没有可显示的内容。” | Current radio/test state; cannot infer ordinary numeric-counter behavior from its name. |
| Calendar | Month view and menu: add event, go to date, delete items before today. New event has title, time, reminder, location, ringtone, repeat, note; focus wraps. | New event canceled without saving; existing events not opened. Handset calendar showed 2026-01, which is not the host's capture date. |
| Alarm | New form: time, repeat, name, ringtone, snooze (observed 10 minutes). | Original alarms excluded from public samples. New form canceled, no alarm enabled or saved. |
| Snake | Start menu; new game, full-screen board, right-soft-key pause; Continue added to pause menu; exit to launcher. Game types: Classic, Box, Tunnel, Mill, Rails, Apartment. Level uses a bar selector. | One test game started and paused; no score/level settings changed intentionally. Physics, all levels and score persistence untested. |
| Radio | Frequency UI at 87.5 MHz; options: favorites, add favorite, speaker, auto search, fine tuning, set frequency, close radio. | Explicit Close radio executed. No station saved, search run or audio recorded. |
| Files | Phone storage root and details; options Details and Format. | No file opened or copied and no Format invoked. Displayed used/total 23.1/31.2 MB, free 8.0 MB. |
| Recorder | Idle 00:00:00, record control and list shortcut. | Recording was not started and existing recordings were not opened. |
| Contacts | Root menu, generated fixture details/options/delete confirmation, full-store error, then new form after freeing one test slot. | See fixture handling below. No new contact saved. |
| Call history | Root and options: message, details, add contact, block, delete, delete all. | Existing details not opened; no calls, messages or deletion. Full screenshots remain private. |
| Dial editor | A digit from Home opens dial/search. Eleven zeros produced a no-match view; all digits cleared and returned Home. | Matching original contacts can appear, including behind a later menu; those frames are private. No call or message was sent. |
| Flashlight | Top-level activation changed Other's state to On. Other's flashlight entry opens a dedicated power page; Center activates its Open/Close button. | Closed, then Other explicitly showed Off. This verifies UI state, not a camera measurement of the LED. |

## Contact test fixture and device changes

Test mode filled the phone contact store. The first Add contact attempt showed “手机存储已满”, despite free file storage. Native function `user+0x5ea9c4` uses number format `123%04d` at `0x5ead98` and name format `a%04d` at `0x5eada0`. The displayed `a0000` record had exactly `1230000`. Only that matching test-mode fixture was deleted, through its single-record confirmation. New contact then opened successfully. The form contains picture, surname, given name, mobile/home/work numbers, birthday, ringtone and group; it was canceled without saving. One test slot remains free; the rest of the test-mode-generated contacts and the owner's original contacts remain.

Other deliberate state effects: one template Save as draft action whose durable result is unverified; the public browser test URL may be retained in history; a Snake test game was started and paused; music remains paused at the tested track. No reset/erase/format, firmware writes, real calls, SMS sends or payments occurred during this sampling pass. Radio and flashlight are off, stopwatch zero, and the phone was returned Home in session 200. Test mode remains enabled; Close would not fully restore prior settings.

## Home and launcher

In this session Home left soft key opened the browser, right soft key opened Messages, and Center opened the app grid. Exiting an app opened from a Home shortcut returned Home; exiting one opened from the grid returned the grid. The grid opened with Music selected. Moving to Settings, returning Home and reopening the grid again selected Music; session 192 has a matching title assertion. This differs from simply retaining the last menu focus. Other home shortcuts and alternative configurations are still untested.

The owner clarified that the clock page's left/right actions are configurable. Sessions 274–278 verified Settings > Personalization > Keyboard > Lock-screen shortcuts: **right=Messages, left=Internet**. Thus opening the browser is this handset's current configuration, not evidence of a diagnostic key-code bug or a factory default. The clock page's center action opens the menu. Do not confuse its configurable side shortcuts with the center label 功能表, or infer that keyboard locking was enabled simply from the clock view.

The left shortcut picker has 18 entries, in order: None, Contacts, Call history, Messages, Music, Video, Radio, Alarm, Files, Multimedia, Calendar, Calculator, Internet, Recorder, Settings, Other, Counter, Alipay. Down wraps from Alipay to None. The picker initially focuses the assigned Internet entry; this differs from the browser single-select resetting focus to its first row. Focus moves independently of the selected marker. After scrolling through the list, cancel retained left=Internet/right=Messages (282); no setting was saved. Video and Multimedia being offered here does not prove their launch behavior; neither was launched. The owner-excluded Alipay app was not opened. Session 283 returned Home.

## Verified regression

[Numeric SMS boundary plan](handset-plans/README.md) completed 172 steps and seven image assertions in session 093. It restored an empty body and pinyin mode. This validates a narrow route, not full SMS behavior or replica equivalence. The host protocol/failure suite passed 14 tests. Menu reentry has one additional live postcondition assertion in session 192.

## Replaying a reviewed step

Use a real prior screenshot as the guard, with a stable region that includes enough UI state. Avoid the status clock, scrolling titles and blinking caret (JPEG block boundaries can affect adjacent pixels).

```sh
python3 scripts/handset_capture.py step \
  --profile docs/handset-profiles/ta1278-17.00.17.02.json \
  --reference /absolute/path/observed.png --roi 0 32 240 280 \
  --key RSOFT --name reviewed-return --out /absolute/path/new-session
```

Every session records the generated plan, version gate, request metadata, key ACKs and image hashes. A successful key ACK alone is not a visual assertion. Review the resulting screen before expanding an activation route; title-only guards are used only for bounded navigation within an already observed page.

## Browser layout follow-up, sessions 284–312

Firmware 17.00.17.02 was verified in all 29 sessions. There were 107 decoded captures, 39 key ACKs, 27 completed plans and two capture-only sessions. These are execution counts, not coverage percentages. Sessions 285, 301 and 306 had ACKed keys without an immediate screen change; the state was checked before another key was sent. A key ACK alone is not proof of a transition.

- Settings is a scrolling grouped form. Current APNs read CTNET; current home type is 自定义 and homepage is `http://cn.bing.com`. These are device values, not proven defaults. The home-type popup contains just 自定义.
- Download images is an inline checkbox. The first Center press unexpectedly toggled it ON instead of opening a picker (295); the next press restored the observed OFF state (296). Both states were captured. No other preference was changed.
- Font size has 小号字体 and 普通字体. Small was selected. Moving through settings and canceling the picker returned to its original row/value.
- The Clear popup contains 历史 and 缓存, with cache checked in the captured state. It was canceled without toggling or confirming. The final settings group/action is 恢复预设设置; it was only focused, never activated. The old replica's history/bookmarks/offline clear menu was not native evidence.
- Focused single-line settings rows expand to 36 pixels and use larger text. Two-line rows remain 52 pixels. The selected row ends at y=276. This changes scroll position; a uniform list height is wrong.
- Root/page menus and exit retain a dimmed page. Font/home/clear pickers also have opaque black panels extending beyond their highlighted rows. See measured bounds in `evidence/browser-layout-20261025.json`.
- In the browser-exit prompt, LSOFT left the image unchanged (302/303), then CENTER exited (304). A later settings/exit path completed in 312. The final menu wallpaper remains private and was not exported.

The public allowlist now contains 160 screenshots. Original responses through 312 are in the external `ta-1278-source/handset-sessions/20261025` archive; 8,185 numbered-session files were verified, with 8,220 total archive files. No call, SMS, form submission, flash, APN edit, reset or browser-data clear was performed in this follow-up. Native checkbox commit, restore, complete HTML layout and network error paths remain open.

## Snake code and cancel checks: sessions 313–329

Sessions 318/319/321 verify level 8 → preview 7 → Back/reopen 8; content ROI is byte-identical after cancel. Session 323 shows five high scores; session 327 shows sound OFF then ON. Idle wake presses returned ACK without changing the screen; only a separately inspected next step was issued. Sound and level were not committed. Phone left at the game menu.

Five reviewed screens were added (165 total). All raw sessions through 329 are in the external archive, with 8,366 files in its current manifest. No original score values were imported into host storage. Static code recovered maps, timing arguments, movement/score rules and artwork; see [Snake code evidence](snake-recovery.md).

## Alarm control follow-up, sessions 330–357

27 connections, 83 captures and 28 guarded key actions checked inline time/name, weekday checkboxes and snooze cancel. A first Center press on the new time field saved one default test alarm; only that new third record was deleted in sessions 339–343. The original two records were unchanged. The second form was canceled in 356; 357 returned to the menu. Session 338 was rejected before device actions because a later plan key lacked its own observed precondition.

Seven public form/picker images bring the reviewed total to 172. Original alarm entries and wallpaper remain private in the external archive. See [code, UI evidence and implementation limits](alarm-recovery.md).
