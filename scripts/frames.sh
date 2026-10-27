#!/bin/sh
# Render synthetic application states without a window, audio, network or user files.
set -eu
cd "$(dirname "$0")/.."
TA1278_AUDIT_FRAMES=${1:-"$PWD/target/frames"}
export TA1278_AUDIT_FRAMES
cargo test -p phone-desktop --no-default-features --locked views:: -- --test-threads=1
printf 'Frames: %s\n' "$TA1278_AUDIT_FRAMES"
