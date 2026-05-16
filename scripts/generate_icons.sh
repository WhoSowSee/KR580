#!/usr/bin/env bash
# One-shot helper to regenerate the runtime icon set from the source
# `icon.png`. Run from anywhere, only when the source image changes:
#
#   ./scripts/generate_icons.sh
#
# Output files in `assets/icons/` are checked into the repository so the
# application binary does not need to decode or resize the master image at
# build time or at run time.
#
# Requires ImageMagick (`magick` on v7+, `convert` on v6).

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
out_dir="$repo_root/assets/icons"
source="$out_dir/icon.png"
ico_path="$out_dir/icon.ico"

if [ ! -f "$source" ]; then
    echo "Missing source icon: $source" >&2
    exit 1
fi

if command -v magick >/dev/null 2>&1; then
    convert_cmd=(magick)
elif command -v convert >/dev/null 2>&1; then
    convert_cmd=(convert)
else
    echo "ImageMagick is required (install 'magick' or 'convert')." >&2
    exit 1
fi

mkdir -p "$out_dir"

# Standalone cross-platform PNGs (used for the runtime window icon and any
# future installer / desktop-entry packaging).
png_sizes=(16 32 48 64 128 256)
for size in "${png_sizes[@]}"; do
    target="$out_dir/icon-${size}.png"
    "${convert_cmd[@]}" "$source" \
        -filter Lanczos \
        -resize "${size}x${size}" \
        -strip \
        "$target"
    echo "Wrote $target"
done

# Multi-resolution Windows .ico. Covers every typical Explorer / taskbar /
# Start-menu DPI scaling combination on Windows 10/11. ImageMagick stores the
# 256x256 frame as PNG automatically, which is the modern convention. Frames
# are passed in largest-first so default Windows previewers (Photos, Paint,
# IconViewer) display the 256x256 layer when the file is opened directly.
ico_sizes=(256 96 64 48 40 32 24 20 16)
ico_inputs=()
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
for size in "${ico_sizes[@]}"; do
    layer="$tmp_dir/icon-${size}.png"
    "${convert_cmd[@]}" "$source" \
        -filter Lanczos \
        -resize "${size}x${size}" \
        -strip \
        "$layer"
    ico_inputs+=("$layer")
done

"${convert_cmd[@]}" "${ico_inputs[@]}" "$ico_path"
echo "Wrote $ico_path"
