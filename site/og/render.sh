#!/bin/bash
# Renders art.html into the link previews: public/og/og-{en,ru}.png for the site and
# ../../assets/social-preview.png for the GitHub repository settings. Needs Chromium and ImageMagick.
set -euo pipefail
cd "$(dirname "$0")"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
shot() { # name width height: rendered at 2x, scaled down for crisp text
  chromium --headless=new --disable-gpu --hide-scrollbars --window-size="$2,$3" --force-device-scale-factor=2 \
    --virtual-time-budget=1500 --screenshot="$tmp/$1.png" "file://$PWD/art.html#$1" >/dev/null 2>&1
  magick "$tmp/$1.png" -filter Lanczos -resize "$2x$3" -strip -quality 92 "$4"
}
mkdir -p ../public/og
shot og-en 1200 630 ../public/og/og-en.jpg
shot og-ru 1200 630 ../public/og/og-ru.jpg
shot social 1280 640 ../../assets/social-preview.jpg
magick identify -format '%f %wx%h %b\n' ../public/og/*.jpg ../../assets/social-preview.jpg
