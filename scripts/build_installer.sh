#!/usr/bin/env bash
set -euo pipefail

profile="${1:-release}"
if [[ "$profile" != "release" && "$profile" != "debug" ]]; then
  echo "usage: $0 [debug|release]" >&2
  exit 2
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
manifest_path="$repo_root/Cargo.toml"
profile_args=()
if [[ "$profile" == "release" ]]; then
  profile_args+=(--release)
fi

KR580_WINDOWS_ICON_KIND= \
  cargo build "${profile_args[@]}" -p k580-ui --bin k580 --bin kr --manifest-path "$manifest_path"

payload_dir="$repo_root/target/$profile"
KR580_WINDOWS_ICON_KIND=uninstaller \
  cargo build "${profile_args[@]}" -p k580-ui --bin k580-uninstaller --manifest-path "$manifest_path"

KR580_INSTALLER_PAYLOAD_DIR="$payload_dir" \
KR580_WINDOWS_ICON_KIND=setup \
  cargo build "${profile_args[@]}" -p k580-ui --bin k580-installer --manifest-path "$manifest_path"

dist_dir="$repo_root/dist"
mkdir -p "$dist_dir"

version="$(awk -F'"' '/^version[[:space:]]*=/ { print $2; exit }' "$manifest_path")"
os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
setup="$dist_dir/KR580-Setup-$version-$os-$arch"

cp "$payload_dir/k580-installer" "$setup"
chmod 755 "$setup"
printf 'Built installer: %s\n' "$setup"
