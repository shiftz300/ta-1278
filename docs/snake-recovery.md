# Snake: code-verified rules

Firmware **17.00.17.02**. These findings come from bounded Thumb disassembly and initialized data, with handset screenshots as a separate check. They do not establish a complete reverse engineering of every game state.

Run `python3 scripts/recover-snake.py` to check the two source hashes, selected instruction operands and resource pointers, then regenerate the maps, timer table and [evidence manifest](evidence/snake-recovery.json). The manifest stores addresses, raw bytes and hashes for the reviewed code ranges. It does not execute the firmware. On macOS, add `--disassemble` to regenerate the [bounded Thumb listing](evidence/snake-thumb.txt) with installed Xcode LLVM. `+hwdiv` enables instruction decoding and is not a claim about the chip model. Rust tests check the replica against these recovered values; they are not a CPU-level equivalence proof.

| Rule | Native evidence | Replica |
|---|---|---|
| Board | `0x88690988..0x886909ec`: 12 columns, 16 rows, coordinates ×20 | Full 240×320 playfield, without a status or softkey bar |
| Maps | `0x88690bbc..0x88690c10` copies 192 bytes from `user.bin+0x2032b8` and following maps | Six exposed maps use the actual bytes; a seventh compiled map stays hidden |
| Start | `0x8869023e..0x886902a6`: three nodes at (8,6), (7,6), (6,6), moving right | Same coordinates and direction |
| Wrap/collision | `0x886906e0..0x88690758`; `0x8869044c..0x886904e0` | Wraps both axes; every nonzero wall cell and every existing body node collide, including the old tail |
| Turns | `0x88099ba2..0x88099bec`: reject either direction on the current axis | Repeating Right after queuing Up while moving Right does not cancel the queued turn |
| Food | `0x886902ce..0x886902fa`; score addition at `0x88690794` | x=0–11, y=2–15; grows one cell and adds 1 point. Random sequence is a local replacement |
| Bonus | `0x886907a2..0x886907e8`, `0x8869087e..0x886908fe` | Positive score divisible by six can spawn a bonus; collection adds countdown ×3 without growth. Countdown uses native tick arithmetic |
| Speed | `0x88690a6e..0x88690a82`, table at initialized RAM `0x81c3bedc` | Levels 1–8 pass **225, 200, 175, 150, 125, 100, 90, 88** to the timer. Host treats these as milliseconds; physical scheduler timing has not been measured |
| Level | `0x88099e04..0x88099f90` | Eight bars; ends clamp. Back cancels; Center saves and discards the paused game, then shows the sampled completion sheet |
| Game over | `0x88690a36..0x88690aec`: states 2/3; resource `0x401c002c` | Board retained; box (30,70,180,50), fill `0x0127`, red `0xf800`, font 20. Center inert; Back returns to New game |
| Scores | `0x8869062c..0x886906a8` | Five descending entries per exposed map, saved in host `snake.json` |
| Digits | `0x886907f6..0x8869087a` | Ones at (105,3), further digits extend left in 16-pixel steps |
| Art | Resource group `0x401f`, draw calls listed in the manifest | Original background, body/head strips, food, wall tile and numeric sprites |

The initialized-data source is the previously extracted `init_81c20000.bin`, SHA-256 `76b95ae0d1948b677a38d8e6311b65f154642fb19723a4dcdcc8ec0a2da04210`. This is immutable firmware initialization data, not a live RAM or private NV dump. Level 1 is the native missing-setting fallback; level 8 was this handset's saved preference.

## Handset checks

Sessions 318→319→321 showed eight bars, seven after Left, and eight again after Back/reopen. The content rectangle matched exactly after cancellation. Session 323 showed five scores. Session 327 showed **关 / 开**, in that order. No settings were committed in this sampling pass; the phone was left at the game menu.

Public references: [level preview](handset-samples/20261025/snake-level-seven.png), [cancel result](handset-samples/20261025/snake-level-cancel-restored.png), [scores](handset-samples/20261025/snake-high-scores.png), [sound](handset-samples/20261025/snake-sound.png). No handset score values are copied into the replica's saved data.

[Game comparison](comparisons/snake-native-matched-comparison.png) uses a deliberately positioned test fruit and one movement step to match the sampled frame. This checks artwork and geometry; it does not prove matching random sequences. Native references are lossy JPEG captures, while replica PNGs are CPU framebuffers presented through wgpu.

## Display follow-up — 2026-10-26

[Eight new public captures](evidence/snake-display.json) verify the static game-over overlay, its Center/Back behavior, level confirmation and game-type menu behavior. The generic full-screen “提示 / 游戏结束 / 得分” dialog was incorrect and is removed. Accepting a type retains the Continue entry and focuses 游戏类型. The type picker title is 贪吃蛇. Accepting a level displays 全部完成！ with center 确认 before returning to New game.

Level selection now reads the validated saved setting; loading initializes the runtime and picker together, and a new game uses that same setting. A disk round-trip test checks level 8, cancelled preview, committed level 7, reload, displayed preview and new-game speed. The existing desktop file already held level 8; the reported mismatch was not reproduced as lost storage. No handset preferences or scores are copied to desktop storage.

The unread-SMS status icon is suppressed on the full-screen playfield. Center softkey labels in Snake, settings, organizer and tools use the shared measured footer placement. This audit does not establish all-screen pixel parity. Other generic notices for unavailable hardware and invalid input remain local substitutes.

## Remaining gaps

The kernel 55-word random update and two-draw coordinate mapping have now been verified and implemented; initial RAM state, seed and pointer separation remain local policies. [RNG evidence](evidence/snake-random.json). The saved sound preference now enables synthesized local cues, not identified native waveforms. Transient death/win effects, extreme signed score overflow, full-board completion and all suspend/interruption paths are not reproduced. Five-score persistence is tested locally; exact native NV timing is not claimed. Window scheduling can differ from the phone even with the recovered interval table.
