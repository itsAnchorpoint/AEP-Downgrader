#!/bin/bash
# Renders the app screenshot in English and Russian: the report screen of the mock backend (dark theme),
# framed as a macOS window with a soft shadow.
#   assets/screenshot.png, assets/screenshot-ru.png       README.md, README.ru.md, link previews
#   site/public/shots/report-en.webp, report-ru.webp      the website
# Needs pnpm (the app's dev server and Chromium, see apps/desktop/scripts/shots.sh) and ImageMagick.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
bash "$root/apps/desktop/scripts/shots.sh" "$tmp" report dark en,ru 920x640 >/dev/null
W=1840; H=1280; R=26
mkdir -p "$root/site/public/shots"
for lang in en ru; do
  magick "$tmp/report-dark-$lang-920x640.png" \
    -fill '#ff5f57' -draw 'circle 48,44 60,44' -fill '#febc2e' -draw 'circle 88,44 100,44' -fill '#28c840' -draw 'circle 128,44 140,44' \
    \( -size ${W}x${H} xc:none -fill white -draw "roundrectangle 0,0 $((W-1)),$((H-1)) $R,$R" \) -alpha set -compose DstIn -composite \
    \( -size ${W}x${H} xc:none -fill none -stroke 'rgba(255,255,255,0.14)' -strokewidth 2 -draw "roundrectangle 1,1 $((W-2)),$((H-2)) $R,$R" \) \
    -compose Over -composite "$tmp/framed.png"
  magick "$tmp/framed.png" \( +clone -background black -shadow 55x36+0+28 \) +swap -background none -layers merge +repage \
    -resize 1400x "$tmp/shot-$lang.png"
  out="$root/assets/screenshot.png"; [ "$lang" = ru ] && out="$root/assets/screenshot-ru.png"
  cp "$tmp/shot-$lang.png" "$out"
  magick "$tmp/shot-$lang.png" -quality 86 -define webp:alpha-quality=90 "$root/site/public/shots/report-$lang.webp"
done
magick identify -format '%f %wx%h %b\n' "$root/assets/screenshot.png" "$root/assets/screenshot-ru.png" "$root"/site/public/shots/*.webp
