#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 --installer <path> --target <triple> [--output <path>]" >&2
}

installer=""
target=""
output=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --installer)
      if [[ $# -lt 2 || -z "$2" ]]; then
        usage
        exit 2
      fi
      installer="$2"
      shift 2
      ;;
    --target)
      if [[ $# -lt 2 || -z "$2" ]]; then
        usage
        exit 2
      fi
      target="$2"
      shift 2
      ;;
    --output)
      if [[ $# -lt 2 || -z "$2" ]]; then
        usage
        exit 2
      fi
      output="$2"
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

if [[ -z "$installer" || -z "$target" ]]; then
  usage
  exit 2
fi

if [[ ! -f "$installer" ]]; then
  echo "installer not found: $installer" >&2
  exit 1
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
manifest_path="$repo_root/Cargo.toml"
version="$(awk -F'"' '/^version[[:space:]]*=/ { print $2; exit }' "$manifest_path")"
if [[ -z "$version" ]]; then
  echo "workspace version not found in $manifest_path" >&2
  exit 1
fi

case "$target" in
  x86_64-*) deb_arch="amd64" ;;
  aarch64-*) deb_arch="arm64" ;;
  i686-*|i586-*) deb_arch="i386" ;;
  riscv64gc-*|riscv64-*) deb_arch="riscv64" ;;
  sparc64-*) deb_arch="sparc64" ;;
  *) deb_arch="$(dpkg --print-architecture)" ;;
esac

if [[ -z "$output" ]]; then
  output="$repo_root/dist/KR580-Setup-$version-$target.deb"
fi

package_root="$(mktemp -d)"
trap 'rm -rf "$package_root"' EXIT
install_root="$package_root/usr/lib/kr580"
bin_root="$package_root/usr/bin"
app_root="$package_root/usr/share/applications"
icon_root="$package_root/usr/share/icons/hicolor/256x256/apps"
control_root="$package_root/DEBIAN"

mkdir -p "$install_root" "$bin_root" "$app_root" "$icon_root" "$control_root" "$(dirname "$output")"
cp "$installer" "$install_root/KR580-Setup"
chmod 755 "$install_root/KR580-Setup"
ln -s ../lib/kr580/KR580-Setup "$bin_root/kr580-setup"
cp "$repo_root/assets/icons/icon-256.png" "$icon_root/kr580.png"
cat > "$app_root/kr580-setup.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=KR580 Setup
Comment=Install KR580 emulator
Exec=kr580-setup
Icon=kr580
Terminal=false
Categories=Development;Emulator;
DESKTOP

installed_size="$(du -sk "$package_root/usr" | awk '{print $1}')"
cat > "$control_root/control" <<CONTROL
Package: kr580-setup
Version: $version
Section: devel
Priority: optional
Architecture: $deb_arch
Installed-Size: $installed_size
Maintainer: WhoSowSee
Description: KR580 graphical installer
 Standalone installer for the KR580 desktop emulator.
CONTROL

fakeroot dpkg-deb --build "$package_root" "$output"
printf 'Built deb package: %s\n' "$output"
