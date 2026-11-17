#!/bin/bash
# Development: screenshots of every app state with the mock backend (no Tauri needed).
#   scripts/shots.sh <out dir> [states] [themes] [langs] [WxH]     (lists comma-separated)
# Starts the Vite dev server if it is not running, then runs scripts/shots.mjs.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p "${1:?out dir}"
started=""
if ! curl -s -o /dev/null http://localhost:1420/; then
  pnpm exec vite --port 1420 --strictPort >/dev/null 2>&1 & started=$!
  for _ in $(seq 1 50); do curl -s -o /dev/null http://localhost:1420/ && break; sleep 0.2; done
fi
node scripts/shots.mjs "$@"
if [ -n "$started" ]; then kill "$started"; fi
