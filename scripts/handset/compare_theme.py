"""Compare reviewed handset swatches with the independently recovered RGB565 table."""
import hashlib
import json
import statistics
from pathlib import Path
from PIL import Image


def main():
    root = Path(__file__).resolve().parents[2]
    palette = root/'assets/ui/native-theme.json'
    frame = root/'docs/handset-samples/20261025/theme.png'
    data = json.loads(palette.read_text())
    image = Image.open(frame).convert('RGB')
    rows = []
    for index, values in enumerate(data['accents_rgb565']):
        roi = [10+index*41, 32, 26+index*41, 48]
        pixels = list(image.crop(roi).getdata())
        median = [int(statistics.median(p[c] for p in pixels)) for c in range(3)]
        colors = [[(v >> 11 & 31)*255//31, (v >> 5 & 63)*255//63,
                   (v & 31)*255//31] for v in values]
        shade = min(range(7), key=lambda i: sum((a-b)**2 for a, b in zip(colors[i], median)))
        rows.append({'accent': index, 'sample_roi': roi, 'jpeg_median_rgb': median,
                     'closest_palette_shade': shade, 'rgb565': values[shade],
                     'expanded_rgb': colors[shade],
                     'channel_difference': [a-b for a, b in zip(median, colors[shade])]})
    result = {'source_kind': 'native JPEG compared with firmware RGB565 table',
              'palette_sha256': hashlib.sha256(palette.read_bytes()).hexdigest(),
              'frame_sha256': hashlib.sha256(frame.read_bytes()).hexdigest(), 'rows': rows,
              'limits': 'JPEG YUV conversion is lossy. Nearest palette shade is supporting evidence, not proof of exact LCD color conversion.'}
    (root/'docs/evidence/live-theme-comparison.json').write_text(json.dumps(result, indent=2)+'\n')
    print('Nearest palette shades:', [r['closest_palette_shade'] for r in rows])


if __name__ == '__main__':
    main()
