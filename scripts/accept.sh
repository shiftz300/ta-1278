#!/bin/sh
# Isolated local acceptance. Brief real audio at volume 1/10; no device or external service.
set -eu
cd "$(dirname "$0")/.."
command -v python3 >/dev/null
command -v ffmpeg >/dev/null
accept_dir=$(mktemp -d "${TMPDIR:-/tmp}/ta1278-acceptance.XXXXXX")
export TA1278_ACCEPTANCE_MEDIA="$accept_dir/phone"
export TA1278_ACCEPTANCE_ROOT="$accept_dir/result"
python3 - "$TA1278_ACCEPTANCE_MEDIA" <<'PY'
import math
import pathlib
import struct
import sys
import wave

music = pathlib.Path(sys.argv[1]) / "Music"
music.mkdir(parents=True)
for name, frequency in (("First", 330), ("Second", 440)):
    with wave.open(str(music / f"{name}.wav"), "wb") as output:
        output.setparams((1, 2, 16000, 0, "NONE", "not compressed"))
        output.writeframes(b"".join(
            struct.pack("<h", round(320 * math.sin(2 * math.pi * frequency * i / 16000)))
            for i in range(8 * 16000)
        ))
PY
ffmpeg -nostdin -loglevel error -i "$TA1278_ACCEPTANCE_MEDIA/Music/Second.wav" \
    -codec:a libmp3lame -b:a 32k "$TA1278_ACCEPTANCE_MEDIA/Music/Second.mp3"
printf 'Evidence: %s\n' "$accept_dir"
if cargo test --locked -p phone-desktop accept::complete_use_and_restart -- \
    --ignored --exact --nocapture --test-threads=1 > "$accept_dir/test.log" 2>&1; then
    cat "$accept_dir/test.log"
else
    cat "$accept_dir/test.log"
    exit 1
fi
