# Capturing the handset screen

2026-10-25. **Live screenshots and remote keys now work on this handset.** Normal-runtime USB is `2e04:0019`, diagnostic interface 1, IN `83`, OUT `04`. The firmware replied `17.00.17.02`. See [live sampling](handset-observations.md) for observations and limits. The earlier investigation below is retained as history.

## Verified local evidence

Source: original `kernel.bin`, firmware 17.00.17.02, SHA-256 `b871d5a004c9251f3bb7cc58ea637d486f05d680efd3c39be83c38051f11e303`.

Run `python3 scripts/recover-lcd-diagnostic.py`. It validates the source and writes bounded Thumb disassembly plus range hashes to [evidence](evidence/lcd-diagnostic.json). The ELF is an analysis wrapper; it is never executed. Offsets below include the DHTB header.

| Kernel file offset | Evidence |
|---|---|
| `0x163c68` | Direct call to LCD diagnostic registration in an initialization sequence. Execution of that sequence in the current handset session is unverified. |
| `0x1fbe4..0x1fbf0` | Registration checks command ID below 100, then stores the handler pointer in a command table. |
| `0x1a5360..0x1a5378` | Diagnostic registration routine loads a handler pointer and request type `0x33`, then branches to the registration function. |
| `0x1a54f0` | Handler pointer `0x8100afc9`, corresponding to Thumb code at file offset `0x1a51c8` with mapping base `0x80e65e00`. |
| `0x1a51ec..0x1a523e` | Queries LCD information, derives a two-byte-per-pixel buffer size, allocates temporary buffers and copies display data. |
| `0x1a5242..0x1a5272` | Passes copied pixels through conversion and JPEG encoding routines, matching the related reference implementation below. |
| `0x1a52d6..0x1a532c` | Splits encoded output into at most `0x8000`-byte chunks, writes an 8-byte header and response type `0xa0`, and calls a send routine. |
| `0x1a5380..0x1a54ee` | Associated RGB/YUV, JPEG, LCD diagnostic and buffer-read trace strings. |

The response subtype scheme also matches the reference: first chunk has bit 7 set and the total chunk count in low bits; intermediate chunks have sequence numbers; a final non-first chunk has subtype zero. Live version, screenshot and key exchanges subsequently validated this framing on the reviewed UCOM interface. Allocation and JPEG work occur in RAM; no flash programming appears in this bounded handler. Capture briefly uses the running display and is not a proof of zero timing effects.

## Reference and limits

Related W19 source at fixed commit `d48d55e028c13b6f50c291cbb7ea29201c3c8432`:

- [lcd_diag.c](https://github.com/obaidi2005/ZW217_W19_V4.1_yisai/blob/d48d55e028c13b6f50c291cbb7ea29201c3c8432/MS_Customize/source/product/driver/lcd/lcd_diag.c): obtains LCD buffer information, copies RGB565 data, encodes JPEG and sends chunks.
- [cmddef.h](https://github.com/obaidi2005/ZW217_W19_V4.1_yisai/blob/d48d55e028c13b6f50c291cbb7ea29201c3c8432/MS_Ref/export/inc/cmddef.h): request 51, response 160 and the 8-byte message header.

This is related platform source, not this handset's source tree. The local machine-code match is the evidence for the handler in this firmware. Reference files were read as text only; no upstream code was installed or executed. The full reference text is not redistributed here.

[FPDoom's UMS9117 instructions](https://github.com/ilyakurdyukov/fpdoom/blob/main/ums9117/README.md) load their own program through FDL. [The custom FDL documentation](https://github.com/ilyakurdyukov/spreadtrum_flash/blob/main/t117_fdl/README.md) describes RAM reads in that loader. Neither establishes a way to capture the running original phone UI. Replacing execution with a loader is different from attaching to the original OS.

## Earlier blocker (resolved in this session)

At 14:27:51 +08:00, a read-only cached descriptor enumeration found `1782:4d00`, USB 2.0, bcdDevice `0202`. It did not open a device handle or send commands. This matches the previously observed acquisition interface but cannot distinguish BootROM, an FDL or another runtime by VID/PID alone.

1. Start the original system normally and inspect its USB interfaces. The owner has been asked to reconnect with the battery installed and normal charging mode selected.
2. Establish whether a diagnostic channel is available while the original UI remains running, and determine its framing and registration path. A mass-storage interface alone is not this channel.
3. Only after validating the native handler and transport, request a screenshot of a non-private menu or synthetic SMS editor, decode locally and check dimensions/content.
4. If no runtime channel exists, investigate its activation path or a RAM-only monitor without assuming the previous loader can coexist with the OS. This is additional reverse engineering, not a working solution yet.

For immediate visual references, camera photos/video of controlled handset interactions avoid this transport problem. They are faster to obtain but less useful for exact pixel/color comparison. A working diagnostic capture would be better for repeated capture; it still would not recover all hidden interaction logic by itself.

## Automation follow-up

[Capture and replay scripts](handset-automation.md) now include passive USB watching, version-gated capture and image-guarded key plans. Native type-1 dispatch at file `0x1f4c2` selects `0x1f5b8`, which calls the key handler at `0x1f30c`. The handler reads subtype at request+7, key value at request+12, and sends down/up events `0x201`/`0x202` to keypad task 2. Subtype 0 emits both, 1 emits down, 2 emits up. See the regenerated evidence ranges. Directions, both softkeys, center, star, hash, 4 and 6 now have visible runtime results. Held keys and the remaining digit behaviors still need sampling.

Related fixed-version references: [diag.c](https://github.com/obaidi2005/ZW217_W19_V4.1_yisai/blob/d48d55e028c13b6f50c291cbb7ea29201c3c8432/MS_Ref/source/base/src/diag.c), [comm_def.h](https://github.com/obaidi2005/ZW217_W19_V4.1_yisai/blob/d48d55e028c13b6f50c291cbb7ea29201c3c8432/MS_Ref/export/inc/comm_def.h), [sio.c](https://github.com/obaidi2005/ZW217_W19_V4.1_yisai/blob/d48d55e028c13b6f50c291cbb7ea29201c3c8432/chip_drv/chip_module/misc/sio.c). They support the message layout; the separate live version/image/key logs establish compatibility with this handset.
