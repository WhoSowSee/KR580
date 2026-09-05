#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 [debug|release] [--target <triple>] [--dist-dir <path>]" >&2
}

profile="release"
target=""
dist_dir=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    debug|release)
      profile="$1"
      shift
      ;;
    --target)
      if [[ $# -lt 2 || -z "$2" ]]; then
        usage
        exit 2
      fi
      target="$2"
      shift 2
      ;;
    --dist-dir)
      if [[ $# -lt 2 || -z "$2" ]]; then
        usage
        exit 2
      fi
      dist_dir="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      exit 2
      ;;
  esac
done

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS is required to build a DMG" >&2
  exit 1
fi

case "$target" in
  "")
    platform="macos-$(uname -m)"
    architecture="$(uname -m)"
    ;;
  aarch64-apple-darwin)
    platform="$target"
    architecture="arm64"
    ;;
  x86_64-apple-darwin)
    platform="$target"
    architecture="x86_64"
    ;;
  *)
    echo "unsupported macOS target: $target" >&2
    exit 2
    ;;
esac

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
manifest_path="$repo_root/Cargo.toml"
cargo_bin="${KR580_CARGO:-cargo}"
profile_args=()
target_args=()

if [[ "$profile" == "release" ]]; then
  profile_args+=(--release)
fi
if [[ -n "$target" ]]; then
  target_args+=(--target "$target")
fi
if [[ -z "$dist_dir" ]]; then
  dist_dir="$repo_root/dist"
fi

"$cargo_bin" build "${profile_args[@]}" "${target_args[@]}" \
  --locked -p kr580 --bin k580 --manifest-path "$manifest_path"

if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
  case "$CARGO_TARGET_DIR" in
    /*) target_root="$CARGO_TARGET_DIR" ;;
    *) target_root="$PWD/$CARGO_TARGET_DIR" ;;
  esac
else
  target_root="$repo_root/target"
fi
if [[ -n "$target" ]]; then
  binary="$target_root/$target/$profile/k580"
else
  binary="$target_root/$profile/k580"
fi
if [[ ! -x "$binary" ]]; then
  echo "built application not found: $binary" >&2
  exit 1
fi

version="$(awk -F'"' '/^version[[:space:]]*=/ { print $2; exit }' "$manifest_path")"
if [[ -z "$version" ]]; then
  echo "workspace version not found in $manifest_path" >&2
  exit 1
fi

staging_dir="$(mktemp -d "${TMPDIR:-/tmp}/kr580-dmg.XXXXXX")"
cleanup() {
  case "$staging_dir" in
    "${TMPDIR:-/tmp}"/kr580-dmg.*) rm -rf -- "$staging_dir" ;;
  esac
}
trap cleanup EXIT

app="$staging_dir/KR580.app"
contents="$app/Contents"
macos="$contents/MacOS"
resources="$contents/Resources"
mkdir -p "$macos" "$resources" "$dist_dir"

install -m 755 "$binary" "$macos/KR580"
install -m 644 "$repo_root/crates/ui/assets/icons/KR580.icns" "$resources/KR580.icns"
install -m 644 "$repo_root/crates/ui/assets/icons/KR580Document.icns" "$resources/KR580Document.icns"
sed "s/@VERSION@/$version/g" \
  "$repo_root/crates/ui/assets/macos/Info.plist" > "$contents/Info.plist"
ln -s /Applications "$staging_dir/Applications"

plutil -lint "$contents/Info.plist"
lipo -verify_arch "$architecture" "$macos/KR580"

dmg="$dist_dir/KR580-$version-$platform.dmg"
hdiutil create -ov -format UDZO -volname KR580 -srcfolder "$staging_dir" "$dmg"
bash "$script_dir/verify_macos_dmg.sh" \
  --dmg "$dmg" \
  --architecture "$architecture" \
  --version "$version"
printf 'Built application image: %s\n' "$dmg"
