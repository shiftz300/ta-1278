#!/bin/sh
# Build a local, unsigned app bundle. No installation or device access.
set -eu
cd "$(dirname "$0")/.."
cargo build --locked -p phone-desktop
bundle="target/TA1278.app"
mkdir -p "$bundle/Contents/MacOS"
cp target/debug/phone-desktop "$bundle/Contents/MacOS/phone-desktop"
cat > "$bundle/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>phone-desktop</string>
<key>CFBundleIdentifier</key><string>dev.shiftz.ta1278</string>
<key>CFBundleName</key><string>TA1278</string>
<key>CFBundleDisplayName</key><string>TA1278</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
printf 'Ready: %s\n' "$bundle"
