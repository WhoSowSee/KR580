#!/usr/bin/env bash
# One-shot helper to regenerate the runtime icon set. Run from anywhere,
# only when one of the source PNGs changes:
#
#   ./scripts/generate_icons.sh
#
# Sources (in `crates/ui/assets/icons/`):
#   - `icon.png`     — application icon master.
#   - `file-580.png` — `.580` file-type icon master.
#   - `installer-setup.png` — standalone setup icon master.
#   - `installer-uninstall.png` — installed uninstaller icon master.
#
# Outputs in `crates/ui/assets/icons/`, keeping the published crates.io
# package self-contained:
#   - `icon-{16,32,48,64,128,256}.png` — standalone cross-platform PNGs.
#   - `file-580-256.png`               — Linux/macOS document icon source.
#   - `KR580.icns`, `KR580Document.icns` — macOS application/document icons.
#   - `icon.ico`                       — multi-resolution Windows app icon.
#   - `file-580.ico`                   — multi-resolution `.580` file-type icon.
#   - `installer-setup.ico`            — multi-resolution setup `.exe` icon.
#   - `installer-uninstall.ico`        — multi-resolution uninstaller `.exe` icon.
#
# Requires ImageMagick (`magick` on v7+, `convert` on v6).

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
out_dir="$repo_root/crates/ui/assets/icons"

if command -v magick >/dev/null 2>&1; then
    convert_cmd=(magick)
elif command -v convert >/dev/null 2>&1; then
    convert_cmd=(convert)
else
    echo "ImageMagick is required (install 'magick' or 'convert')." >&2
    exit 1
fi

if ! command -v iconutil >/dev/null 2>&1 && ! command -v python3 >/dev/null 2>&1; then
    echo "iconutil or python3 is required to create ICNS files" >&2
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

build_icns() {
    local source="$1"
    local destination="$2"
    local target="$tmp_dir/$(basename "$destination")"
    local iconset="$tmp_dir/$(basename "$target" .icns).iconset"
    mkdir -p "$iconset"
    render_layer "$source" 16 "$iconset/icon_16x16.png"
    render_layer "$source" 32 "$iconset/icon_16x16@2x.png"
    render_layer "$source" 32 "$iconset/icon_32x32.png"
    render_layer "$source" 64 "$iconset/icon_32x32@2x.png"
    render_layer "$source" 128 "$iconset/icon_128x128.png"
    render_layer "$source" 256 "$iconset/icon_128x128@2x.png"
    render_layer "$source" 256 "$iconset/icon_256x256.png"
    render_layer "$source" 512 "$iconset/icon_256x256@2x.png"
    render_layer "$source" 512 "$iconset/icon_512x512.png"
    render_layer "$source" 1024 "$iconset/icon_512x512@2x.png"
    if command -v iconutil >/dev/null 2>&1; then
        iconutil --convert icns --output "$target" "$iconset"
    else
        python3 - "$iconset" "$target" <<'PY'
import struct
import sys
from pathlib import Path

iconset = Path(sys.argv[1])
target = Path(sys.argv[2])
entries = [
    (b"icp4", "icon_16x16.png"),
    (b"icp5", "icon_32x32.png"),
    (b"icp6", "icon_32x32@2x.png"),
    (b"ic07", "icon_128x128.png"),
    (b"ic08", "icon_256x256.png"),
    (b"ic09", "icon_512x512.png"),
    (b"ic10", "icon_512x512@2x.png"),
]
payload = bytearray()
for kind, filename in entries:
    image = (iconset / filename).read_bytes()
    payload.extend(kind)
    payload.extend(struct.pack(">I", len(image) + 8))
    payload.extend(image)
target.write_bytes(b"icns" + struct.pack(">I", len(payload) + 8) + payload)
PY
    fi
    if command -v python3 >/dev/null 2>&1; then
        python3 - "$target" <<'PY'
import struct
import sys
from pathlib import Path

data = Path(sys.argv[1]).read_bytes()
if len(data) < 8 or data[:4] != b"icns" or struct.unpack(">I", data[4:8])[0] != len(data):
    raise SystemExit(f"invalid ICNS container: {sys.argv[1]}")
PY
    elif [[ "$(od -An -tx1 -N4 "$target" | tr -d '[:space:]')" != "69636e73" ]]; then
        echo "invalid ICNS container: $target" >&2
        exit 1
    fi
    mv "$target" "$destination"
    echo "Wrote $destination"
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

# The standalone layer is used for Linux MIME integration and macOS packaging.
render_layer "$file_source" 256 "$out_dir/file-580-256.png"
echo "Wrote $out_dir/file-580-256.png"

# The 128 size is included because Explorer's "Extra large icons" view uses it.
file_ico_sizes=(256 128 96 64 48 40 32 24 20 16)
build_ico "$file_source" "$file_ico" "${file_ico_sizes[@]}"
build_icns "$app_source" "$out_dir/KR580.icns"
build_icns "$file_source" "$out_dir/KR580Document.icns"

# ---- Setup / uninstaller icons --------------------------------------------
installer_ico_sizes=(256 128 96 64 48 40 32 24 20 16)

setup_source="$out_dir/installer-setup.png"
setup_ico="$out_dir/installer-setup.ico"
if [ ! -f "$setup_source" ]; then
    echo "Missing source icon: $setup_source" >&2
    exit 1
fi
build_ico "$setup_source" "$setup_ico" "${installer_ico_sizes[@]}"

uninstall_source="$out_dir/installer-uninstall.png"
uninstall_ico="$out_dir/installer-uninstall.ico"
if [ ! -f "$uninstall_source" ]; then
    echo "Missing source icon: $uninstall_source" >&2
    exit 1
fi
build_ico "$uninstall_source" "$uninstall_ico" "${installer_ico_sizes[@]}"
