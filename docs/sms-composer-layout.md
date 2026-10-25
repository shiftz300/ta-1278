# SMS composer layout

2026-10-26, firmware 17.00.17.02. This pass compares equal text and input modes with four existing public handset captures. It sends no handset commands and changes no original device data.

The previous replica used 20px for both Latin and Chinese text, fractional advances, a 6px body inset, a short caret and a lower count baseline. Its long editor also showed eight rows where the captured phone showed nine, moving the last text row. The corrected layout uses the measured positions below.

| Element | Current geometry |
|---|---|
| Body | x=0..199, lower edge y=254; one row starts y=220, two rows y=194; maximum starts y=24 |
| Text | 20px Latin, 24px CJK, whole-pixel advances; 26px rows. Long Latin/digit runs wrap before their first chunk after CJK |
| Last editor row | y=224; Latin glyph ascent is adjusted relative to the derived CJK font |
| Caret | y=225..251 on the last row; drawn for an empty active body too |
| Count | Visible glyphs y=261..274 |
| Input mode | Right aligned to x=193; green label and a fitted pencil for Latin/numeric modes |
| Center softkey | Text origin y=289 |
| Long editor scroll | Track x=236..239, y=24..281; proportional thumb |

The conversation composer uses the same text metrics and measures its viewport with the same wrap function. Recipient editing and other applications retain their separate layouts. Text navigation uses the rendered whole-pixel stops, including UTF-16 offsets.

## Comparisons

Left is the handset, right is the replica. Original image pixels are preserved; radio/status icons differ.

- [One Chinese character](comparisons/sms-native-one-comparison.png)
- [Newline](comparisons/sms-native-newline-comparison.png)
- [70-character mixed text](comparisons/sms-native-wrap-comparison.png)
- [161 digits and scrolling](comparisons/sms-native-scrolled-comparison.png)

Regenerate with:

```sh
python3 scripts/compare-sms-composer.py --replica /tmp/ta1278-smoke --out docs/comparisons
```

[Comparison manifest](comparisons/sms-composer-index.json) records source/frame/output hashes. Source body sequence for the 70-character case is consistent with archived session 090: the existing Chinese character is followed directly by numeric-key input, without an inserted newline.

## Limits

20/24px sizing, whole-pixel advances, word wrapping and ascent adjustments are fitted to the native images. They are not proven native font-engine code. The original SFON outlines are converted to OTF and rasterized locally; hinting, some glyph edges, the six-row bubble boundary and fitted pencil/delete icons still differ. Native images are lossy JPEG captures. Prediction behavior and every editor state are not covered by these four comparisons.

Two regression tests check single/multiple/scrolled row baselines, the fixed count/footer region, caret height, scrollbar, whole-pixel wrap and UTF-16 cursor stops. They supplement visual comparison rather than prove all-screen equivalence.

The later [shared UI pass](shared-ui.md) moves edit metrics and measurement into `ui/`, uses one softkey baseline (y=289) and one fitted Delete icon across editors. It preserves these message content metrics.
