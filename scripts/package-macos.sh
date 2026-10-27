#!/bin/sh
# Build a local, ad-hoc signed app bundle. No installation or device access.
set -eu
cd "$(dirname "$0")/.."
if test -n "${1:-}"; then
  binary="target/$1/release/phone-desktop"
  if test "${2:-}" != "--no-build"; then
    cargo build --release --locked -p phone-desktop --target "$1"
  fi
else
  cargo build --release --locked -p phone-desktop
  binary="target/release/phone-desktop"
fi
version=$("$binary" --version)
version=${version#TA1278 }
bundle="target/TA1278.app"
mkdir -p "$bundle/Contents/MacOS"
cp "$binary" "$bundle/Contents/MacOS/phone-desktop.new"
mv -f "$bundle/Contents/MacOS/phone-desktop.new" "$bundle/Contents/MacOS/phone-desktop"
helper=$("$binary" --print-media-helper)
test -n "$helper"
cp "$helper" "$bundle/Contents/MacOS/ta1278-media"
codesign --force --sign - --timestamp=none "$bundle/Contents/MacOS/ta1278-media"
cat > "$bundle/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>phone-desktop</string>
<key>CFBundleIdentifier</key><string>dev.shiftz.ta1278</string>
<key>CFBundleName</key><string>TA1278</string>
<key>CFBundleDisplayName</key><string>TA1278</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>$version</string>
<key>CFBundleShortVersionString</key><string>$version</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
notices="$bundle/Contents/Resources/Notices"
# Replace generated notices so removed resources leave no stale bundle files.
rm -rf "$notices"
mkdir -p "$notices/fonts" "$notices/input" "$notices/audio" "$notices/resources"
cp third-party-licenses/resources/*.txt "$notices/resources/"
cp THIRD_PARTY_NOTICES.md "$notices/"
cp third-party-licenses/audio/*.txt "$notices/audio/"
cp -R third-party-licenses/audio/sources "$notices/audio/"
cp assets/README.md assets/sources.json "$notices/"
cp assets/ui/owner-wallpaper.json "$notices/"
cp assets/input/t9/manifest.json "$notices/input/t9-manifest.json"
cp assets/fonts/*.txt "$notices/fonts/"
cp assets/input/*LICENSE* assets/input/*AUTHORS* assets/input/CC-BY-SA-4.0.txt assets/input/FrequencyWords-README.md "$notices/input/"
codesign --force --sign - --timestamp=none "$bundle"
codesign --verify --deep --strict "$bundle"
printf 'Ready: %s\n' "$bundle"
