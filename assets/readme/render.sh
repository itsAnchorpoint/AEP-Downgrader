#!/bin/bash
# Renders art.html into the README's buttons and support banner (PNG at 2x, transparent around the shapes).
# Needs Chromium and ImageMagick. Buttons of one language get the same height, so they line up in a row.
set -euo pipefail
cd "$(dirname "$0")"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
shot() { # name width height
  chromium --headless=new --disable-gpu --hide-scrollbars --default-background-color=00000000 \
    --window-size="$2,$3" --force-device-scale-factor=2 --virtual-time-budget=1500 \
    --screenshot="$tmp/$1.png" "file://$PWD/art.html#$1" >/dev/null 2>&1
}
for lang in en ru; do
  for b in download boosty tribute; do
    shot "$b-$lang" 520 110
    magick "$tmp/$b-$lang.png" -trim +repage "$tmp/$b-$lang.t.png"
  done
  h=$(magick "$tmp"/*-"$lang".t.png -format '%h\n' info: | sort -n | tail -1)
  for b in download boosty tribute; do
    w=$(magick "$tmp/$b-$lang.t.png" -format %w info:)
    magick "$tmp/$b-$lang.t.png" -background none -gravity center -extent "${w}x$h" "$b-$lang.png"
  done
  shot "banner-$lang" 1200 330
  cp "$tmp/banner-$lang.png" "banner-$lang.png"
done
magick identify -format '%f %wx%h\n' ./*.png
