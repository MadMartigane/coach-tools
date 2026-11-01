#!/usr/bin/env bash
# Regenerates the versioned PNG icons from their SVG sources with ImageMagick.
# An offline step, not part of the build: `just build` ships the committed PNGs as they are.
#
# Usage: scripts/generate-icons.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

readonly ICON_DIR="app/public"

die() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

check_imagemagick() {
  if ! command -v magick >/dev/null 2>&1; then
    die "magick not found; install ImageMagick with its SVG renderer to regenerate the icons"
  fi
}

# Renders $ICON_DIR/<name>.svg at its intrinsic size, then checks the PNG geometry.
# -alpha off writes opaque PNGs; -strip keeps reruns byte-identical (no render-time metadata).
# Arguments: <name> <expected WIDTHxHEIGHT>.
render_icon() {
  local name="$1" expected_geometry="$2"
  local source="$ICON_DIR/$name.svg" target="$ICON_DIR/$name.png"
  local actual_geometry

  magick "$source" -alpha off -strip "$target"
  actual_geometry="$(magick identify -format '%wx%h' "$target")"
  if [[ "$actual_geometry" != "$expected_geometry" ]]; then
    die "$target is $actual_geometry, expected $expected_geometry"
  fi
  magick identify "$target"
}

main() {
  check_imagemagick
  render_icon icon-192 192x192
  render_icon icon-512 512x512
  render_icon icon-maskable-512 512x512
  render_icon apple-touch-icon 180x180
}

main "$@"
