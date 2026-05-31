#!/usr/bin/env bash
# One-shot helper to regenerate the runtime icon set. Run from anywhere,
# only when one of the source PNGs changes:
#
#   ./scripts/generate_icons.sh
#
# Sources (in `assets/icons/`):
#   - `icon.png`     — application icon master.
#   - `file-580.png` — `.580` file-type icon master.
#
# Outputs (also in `assets/icons/`, all checked into the repository so
# the application binary does not need to decode or resize the master
# images at build time or at run time):
#   - `icon-{16,32,48,64,128,256}.png` — standalone cross-platform PNGs.
#   - `icon.ico`                       — multi-resolution Windows app icon.
#   - `file-580.ico`                   — multi-resolution `.580` file-type icon.
#
# Requires ImageMagick (`magick` on v7+, `convert` on v6).

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
out_dir="$repo_root/assets/icons"

if command -v magick >/dev/null 2>&1; then
    convert_cmd=(magick)
elif command -v convert >/dev/null 2>&1; then
    convert_cmd=(convert)
else
    echo "ImageMagick is required (install 'magick' or 'convert')." >&2
    exit 1
fi

mkdir -p "$out_dir"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

# Render one resampled PNG layer from the source master.
render_layer() {
    local source="$1"
    local size="$2"
    local target="$3"
    "${convert_cmd[@]}" "$source" \
        -filter Lanczos \
        -resize "${size}x${size}" \
        -strip \
        "$target"
}

# Build a multi-resolution Windows .ico from the source master. ImageMagick
# stores the 256x256 frame as PNG automatically, which is the modern
# convention. Frames are passed in largest-first so default Windows
# previewers (Photos, Paint, IconViewer) display the largest layer when
# the file is opened directly.
build_ico() {
    local source="$1"
    local ico_path="$2"
    shift 2
    local sizes=("$@")

    local layers=()
    local layer_dir
    layer_dir="$(mktemp -d -p "$tmp_dir")"
    for size in "${sizes[@]}"; do
        local layer="$layer_dir/icon-${size}.png"
        render_layer "$source" "$size" "$layer"
        layers+=("$layer")
    done

    "${convert_cmd[@]}" "${layers[@]}" "$ico_path"
    echo "Wrote $ico_path"
}

# ---- Application icon -------------------------------------------------------
app_source="$out_dir/icon.png"
app_ico="$out_dir/icon.ico"

if [ ! -f "$app_source" ]; then
    echo "Missing source icon: $app_source" >&2
    exit 1
fi

# Standalone cross-platform PNGs (used for the runtime window icon and any
# future installer / desktop-entry packaging).
app_png_sizes=(16 32 48 64 128 256)
for size in "${app_png_sizes[@]}"; do
    target="$out_dir/icon-${size}.png"
    render_layer "$app_source" "$size" "$target"
    echo "Wrote $target"
done

# Multi-resolution Windows .ico for the application. Covers every typical
# Explorer / taskbar / Start-menu DPI scaling combination on Windows 10/11.
app_ico_sizes=(256 96 64 48 40 32 24 20 16)
build_ico "$app_source" "$app_ico" "${app_ico_sizes[@]}"

# ---- `.580` file-type icon --------------------------------------------------
file_source="$out_dir/file-580.png"
file_ico="$out_dir/file-580.ico"

if [ ! -f "$file_source" ]; then
    echo "Missing source icon: $file_source" >&2
    exit 1
fi

# The `.580` icon is only consumed as a Windows PE resource (id 2) via
# `crates/ui/build.rs`, so only the multi-resolution ICO is needed. The
# 128 size is included because Explorer's "Extra large icons" view uses it.
file_ico_sizes=(256 128 96 64 48 40 32 24 20 16)
build_ico "$file_source" "$file_ico" "${file_ico_sizes[@]}"
