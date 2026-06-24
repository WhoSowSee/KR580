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

KR580_WINDOWS_ICON_KIND= \
  "$cargo_bin" build "${profile_args[@]}" "${target_args[@]}" -p kr580 --bin k580 --bin kr --manifest-path "$manifest_path"

host_target_root="${CARGO_TARGET_DIR:-$repo_root/target}"
container_target_root="$host_target_root"
if [[ "$cargo_bin" == "cross" && -z "${CARGO_TARGET_DIR:-}" ]]; then
  container_target_root="/target"
fi
if [[ -n "$target" ]]; then
  payload_dir="$host_target_root/$target/$profile"
  build_payload_dir="$container_target_root/$target/$profile"
else
  payload_dir="$host_target_root/$profile"
  build_payload_dir="$container_target_root/$profile"
fi

KR580_WINDOWS_ICON_KIND=uninstaller \
  "$cargo_bin" build "${profile_args[@]}" "${target_args[@]}" -p kr580 --bin k580-uninstaller --manifest-path "$manifest_path"

KR580_INSTALLER_PAYLOAD_DIR="$build_payload_dir" \
KR580_WINDOWS_ICON_KIND=setup \
  "$cargo_bin" build "${profile_args[@]}" "${target_args[@]}" -p kr580 --bin k580-installer --manifest-path "$manifest_path"

mkdir -p "$dist_dir"

version="$(awk -F'"' '/^version[[:space:]]*=/ { print $2; exit }' "$manifest_path")"
if [[ -z "$version" ]]; then
  echo "workspace version not found in $manifest_path" >&2
  exit 1
fi

if [[ -n "$target" ]]; then
  platform="$target"
else
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"
  platform="$os-$arch"
fi

exe_ext=""
if [[ "$platform" == *windows* ]]; then
  exe_ext=".exe"
fi

source="$payload_dir/k580-installer$exe_ext"
setup="$dist_dir/KR580-Setup-$version-$platform$exe_ext"

cp "$source" "$setup"
chmod 755 "$setup"
printf 'Built installer: %s\n' "$setup"
