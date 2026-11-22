#!/bin/bash
# Renders background.html into background.tiff (660x460 and its @2x page) for the disk image window.
# Needs Chromium and ImageMagick. Finder draws the icons and labels: app at (154,236), Applications
# at (506,236), labels (always black) centred 84 pt lower on the plates. Finder's title bar takes
# the bottom 28 pt, and its tab, path and status bars, when shown, cover everything below y≈346:
# keep what matters above that line.
set -euo pipefail
cd "$(dirname "$0")"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
for s in 1 2; do
  chromium --headless=new --disable-gpu --hide-scrollbars --window-size=660,460 \
    --force-device-scale-factor=$s --screenshot="$tmp/bg$s.png" "file://$PWD/background.html" >/dev/null 2>&1
done
magick \( "$tmp/bg1.png" -set units PixelsPerInch -set density 72 \) \
       \( "$tmp/bg2.png" -set units PixelsPerInch -set density 144 \) -compress lzw background.tiff
magick identify -format '%wx%h @ %x dpi\n' background.tiff
