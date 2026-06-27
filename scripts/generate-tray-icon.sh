#!/bin/bash
# macOS menu bar template icon: black silhouette on transparent background.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/src-tauri/icons/32x32.png"
OUT="$ROOT/src-tauri/icons/tray-icon.png"

magick "$SRC" -resize 18x18 \
  \( +clone -alpha extract -write mpr:alpha +delete \) \
  -fill black -colorize 100% \
  mpr:alpha -alpha off -compose CopyOpacity -composite \
  -background none -gravity center -extent 22x22 \
  "$OUT"

echo "Wrote $OUT"
