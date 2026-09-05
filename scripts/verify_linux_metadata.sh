#!/usr/bin/env bash
set -euo pipefail

for command in desktop-file-validate update-desktop-database update-mime-database; do
  if ! command -v "$command" >/dev/null; then
    echo "required command not found: $command" >&2
    exit 1
  fi
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
assets="$repo_root/crates/ui/assets/linux"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/kr580-linux-metadata.XXXXXX")"
cleanup() {
  case "$work_dir" in
    "${TMPDIR:-/tmp}"/kr580-linux-metadata.*) rm -rf -- "$work_dir" ;;
  esac
}
trap cleanup EXIT

applications="$work_dir/share/applications"
mime="$work_dir/share/mime"
mkdir -p "$applications" "$mime/packages"

render_desktop() {
  local source="$1"
  local target="$2"
  grep -Fq '@EXEC@' "$source"
  sed 's|@EXEC@|/opt/kr580/kr580|g' "$source" > "$target"
  ! grep -Fq '@EXEC@' "$target"
  desktop-file-validate "$target"
}

render_desktop "$assets/kr580.desktop" "$applications/kr580.desktop"
render_desktop \
  "$assets/kr580-file-handler.desktop" \
  "$applications/kr580-file-handler.desktop"
render_desktop \
  "$assets/kr580-package.desktop" \
  "$applications/kr580-package.desktop"
install -m 644 \
  "$assets/application-x-kr580.xml" \
  "$mime/packages/application-x-kr580.xml"

XDG_DATA_HOME="$work_dir/share" XDG_DATA_DIRS="$work_dir/share" \
  update-mime-database "$mime"
XDG_DATA_HOME="$work_dir/share" XDG_DATA_DIRS="$work_dir/share" \
  update-desktop-database "$applications"

grep -Fq 'application/x-kr580:*.580' "$mime/globs2"
grep -Fq 'application/x-kr580:*.krs' "$mime/globs2"
grep -Fq 'application/x-kr580=' "$applications/mimeinfo.cache"
printf 'Linux desktop metadata: ok\n'
