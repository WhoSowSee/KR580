#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 --dmg <path> --architecture <arm64|x86_64> --version <version>" >&2
}

dmg=""
architecture=""
version=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dmg)
      if [[ $# -lt 2 || -z "$2" ]]; then
        usage
        exit 2
      fi
      dmg="$2"
      shift 2
      ;;
    --architecture)
      if [[ $# -lt 2 || -z "$2" ]]; then
        usage
        exit 2
      fi
      architecture="$2"
      shift 2
      ;;
    --version)
      if [[ $# -lt 2 || -z "$2" ]]; then
        usage
        exit 2
      fi
      version="$2"
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
  echo "macOS is required to inspect a DMG" >&2
  exit 1
fi
for command in hdiutil iconutil lipo plutil; do
  if ! command -v "$command" >/dev/null; then
    echo "required command not found: $command" >&2
    exit 1
  fi
done
if [[ ! -f "$dmg" || -z "$version" ]]; then
  usage
  exit 2
fi
case "$architecture" in
  arm64|x86_64) ;;
  *)
    usage
    exit 2
    ;;
esac

mount_dir="$(mktemp -d "${TMPDIR:-/tmp}/kr580-mount.XXXXXX")"
icon_check_dir="$(mktemp -d "${TMPDIR:-/tmp}/kr580-icons.XXXXXX")"
mounted=false
cleanup() {
  if [[ "$mounted" == true ]]; then
    if hdiutil detach "$mount_dir" >/dev/null 2>&1; then
      mounted=false
    fi
  fi
  if [[ "$mounted" == false ]]; then
    rmdir -- "$mount_dir" 2>/dev/null || true
  fi
  case "$icon_check_dir" in
    "${TMPDIR:-/tmp}"/kr580-icons.*) rm -rf -- "$icon_check_dir" ;;
  esac
}
trap cleanup EXIT

hdiutil verify "$dmg"
hdiutil attach -nobrowse -readonly -mountpoint "$mount_dir" "$dmg" >/dev/null
mounted=true

app="$mount_dir/KR580.app"
plist="$app/Contents/Info.plist"
binary="$app/Contents/MacOS/kr580"
test -x "$binary"
test -f "$app/Contents/Resources/KR580.icns"
test -f "$app/Contents/Resources/KR580Document.icns"
iconutil --convert iconset \
  --output "$icon_check_dir/KR580.iconset" \
  "$app/Contents/Resources/KR580.icns"
iconutil --convert iconset \
  --output "$icon_check_dir/KR580Document.iconset" \
  "$app/Contents/Resources/KR580Document.icns"
test -f "$icon_check_dir/KR580.iconset/icon_16x16.png"
test -f "$icon_check_dir/KR580.iconset/icon_512x512@2x.png"
test -f "$icon_check_dir/KR580Document.iconset/icon_16x16.png"
test -f "$icon_check_dir/KR580Document.iconset/icon_512x512@2x.png"
test -L "$mount_dir/Applications"
test "$(readlink "$mount_dir/Applications")" = "/Applications"
plutil -lint "$plist"
lipo -verify_arch "$architecture" "$binary"
test "$(plutil -extract CFBundleExecutable raw -o - "$plist")" = "kr580"
test "$(plutil -extract CFBundleIdentifier raw -o - "$plist")" = "dev.kr580.emulator"
test "$(plutil -extract CFBundleShortVersionString raw -o - "$plist")" = "$version"
test "$(plutil -extract UTExportedTypeDeclarations.0.UTTypeIdentifier raw -o - "$plist")" = "dev.kr580.snapshot"
test "$(plutil -extract UTExportedTypeDeclarations.1.UTTypeIdentifier raw -o - "$plist")" = "dev.kr580.subprogram"

hdiutil detach "$mount_dir" >/dev/null
mounted=false
printf 'macOS application image: ok\n'
