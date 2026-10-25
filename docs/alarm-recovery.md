# Alarm evidence and implementation

This pass replaces the generic alarm text dialogs with native-style controls. It does **not** complete organizer reverse engineering.

## Code evidence

Source: firmware 17.00.17.02, `user.bin`, SHA-256 `8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca`. Addresses below use base `0x88000000`; file offsets omit that base. The firmware is never executed on the host.

Run `python3 scripts/recover-alarm.py --disassemble`. This checks the source hash and table pointers, extracts `assets/ui/native-alarm.json`, and records bounded Thumb listings and byte-range hashes in [the report](evidence/alarm-recovery.json) and [listing](evidence/alarm-thumb.txt). These checks reproduce the extraction; they are not an execution-equivalence proof.

| Evidence | Finding |
|---|---|
| Form descriptor `user+0x4ad54c`, handler `0x88118aec` | Inline time and name controls; name limit 20 UTF-16 units. Center softkey resource `0x40000b28` means Save. |
| Focus dispatch `0x881185c0..0x8811866a` | Time/name invoke save helper `0x881184c8`; repeat/ringtone/snooze open child controls. |
| Repeat handler `0x88118e18..0x88118e68` | Back reads checked rows and stores the mask in the draft. None/all/custom map to modes 0/1/2. This does not save NV. |
| Tables `user+0x4ad9a8`, `user+0x4acf7d` | Monday–Sunday labels and bits `[2,4,8,16,32,64,1]`. |
| Snooze handler `0x881190f0..0x8811914c`, table `user+0x4ad9e0` | Five choices: 2, 5, 10, 15, 20 minutes. Center stores the selected index; Back closes without storing. |
| String `0x4018016a` | Says at most five alarms. The replica uses this limit; the native capacity-check branch has not been traced. |

## Handset observations

Sessions 330–357 contain 27 actual connections (338 was rejected locally before any device operation): 27 version checks, 83 decoded captures, 28 key acknowledgments, 26 completed guarded plans and one capture-only session. Every injected key used an observed screenshot precondition. Some first keys only woke the screen; acknowledgments alone were not treated as navigation success.

- The list has a New Alarm row. Center on an existing entry toggles it; Edit is in the left-softkey menu. The menu is a bottom popup with Edit, Delete and Delete All.
- The new form showed 09:00, no repeat, name 闹钟, Clock1 and 10 minutes. These are observed values, not proven factory defaults.
- Time and name are inline controls. Center saves on these fields and opens a picker on repeat/snooze. Name focus displays a delete softkey.
- Repeat shows seven checkboxes and applies to the draft on Back. A checked Monday displays the short label 一 in the form.
- Snooze focus and saved selection are separate. Moving from 10 to 15 and pressing Back retained 10.
- The form has stacked label/value fields, a scrollbar and context-sensitive softkeys. Approximate field spacing is 69 pixels; pickers use 42-pixel rows.

**Temporary state change:** session 337 pressed Center on the time field while investigating its action. This saved one default 09:00 test alarm. Sessions 339–343 selected and deleted only that newly created third record. The original two entries remained unchanged and off. A second form was sampled without saving and canceled in 356; 357 exited the alarm app. Original alarm names/screens stay in the private external archive, not in public reports or app fixtures. No call or SMS was sent.

Seven reviewed non-private screens were added to the public collection, now 172 images. Raw sessions are SHA-256 checked in `../ta-1278-source/handset-sessions/20261025/`.

## Replica behavior and limits

`phone/alarm.rs` owns alarm actions, extracted tables and local scheduling. `views/alarm.rs` owns drawing. Existing organizer JSON remains compatible, including older 一次/每天/工作日/周末 values and snooze strings containing spaces.

Implemented: New row, Center toggle, edit/delete options, inline time/name, 20-unit name limit, weekday checkboxes, snooze commit/cancel, whole-form cancel, five-record limit, local persistence, and queued alerts with snooze across midnight. Alerts preserve the interrupted editor/menu and consume their own input.

Local scheduling runs only while the app runs, using UTC+8 wall time. One-shot alarms disable on firing. Snoozed alerts are in memory only; quitting loses them. Alert layout, stop/snooze key assignment, missed-minute policy, time digit advance/rejection, combined weekday punctuation and Delete All confirmation behavior are host policies. The native wake service, exact time widget, interruption handling and these unsampled branches have not been fully recovered. Clock rollback and sleep catch-up are not native-equivalent. Ringtone selection/audio are explicitly unavailable. The observed native title carried over after returning from pickers; the replica uses the stable alarm title.

[Five labeled visual comparisons](comparisons/index.json) cover the form and weekday/snooze pickers. Fonts, toggle shapes, dynamic form heights and scrolling still differ. A test passing or a screen looking close is not full handset acceptance.
