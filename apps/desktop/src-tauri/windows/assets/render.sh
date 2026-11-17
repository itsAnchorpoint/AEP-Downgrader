#!/bin/bash
# Renders the static art of the Windows installer window (art.html) into PNGs at 3x on a transparent
# background: fold.png (368 x 274 page, Fold at 56,56) and mark.png (22 x 22). The installer plug-in
# (../ui) embeds them and scales them to the screen. Needs Chromium.
set -euo pipefail
cd "$(dirname "$0")"
shot() { # part width height
  chromium --headless=new --disable-gpu --hide-scrollbars --default-background-color=00000000 \
    --window-size="$2,$3" --force-device-scale-factor=3 --screenshot="$PWD/$1.png" \
    "file://$PWD/art.html?part=$1" >/dev/null 2>&1
}
shot fold 368 274
shot mark 22 22
file fold.png mark.png
