#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/kr580-release-names.XXXXXX")"
cleanup() {
  case "$work_dir" in
    "${TMPDIR:-/tmp}"/kr580-release-names.*) rm -rf -- "$work_dir" ;;
  esac
}
trap cleanup EXIT

touch \
  "$work_dir/KR580-Setup-x86_64.zip" \
  "$work_dir/KR580-Setup-x86_64.deb" \
  "$work_dir/KR580-2.4.0-aarch64-apple-darwin.dmg" \
  "$work_dir/KR580-2.4.0-amd64.snap" \
  "$work_dir/KR580-2.4.0-x86_64-unknown-linux-gnu.rpm" \
  "$work_dir/KR580-2.4.0-aarch64-unknown-linux-gnu.rpm" \
  "$work_dir/KR580-2.4.0-x86_64-unknown-linux-gnu.pkg.tar.zst"

bash "$script_dir/version_release_artifacts.sh" "$work_dir" v2.4.0
bash "$script_dir/version_release_artifacts.sh" "$work_dir" v2.4.0

test -f "$work_dir/KR580-Setup-v2.4.0-x86_64.zip"
test -f "$work_dir/KR580-Setup-v2.4.0-x86_64.deb"
test -f "$work_dir/KR580-2.4.0-aarch64-apple-darwin.dmg"
test -f "$work_dir/KR580-2.4.0-amd64.snap"
test -f "$work_dir/KR580-2.4.0-x86_64-unknown-linux-gnu.rpm"
test -f "$work_dir/KR580-2.4.0-aarch64-unknown-linux-gnu.rpm"
test -f "$work_dir/KR580-2.4.0-x86_64-unknown-linux-gnu.pkg.tar.zst"
test "$(find "$work_dir" -maxdepth 1 -type f | wc -l)" -eq 7

bad_dir="$work_dir/bad"
mkdir "$bad_dir"
touch "$bad_dir/KR580-2.3.0-amd64.snap"
if bash "$script_dir/version_release_artifacts.sh" "$bad_dir" v2.4.0; then
  echo "mismatched package version was accepted" >&2
  exit 1
fi
for extension in rpm pkg.tar.zst; do
  bad_dir="$work_dir/bad-$extension"
  mkdir "$bad_dir"
  touch "$bad_dir/KR580-2.3.0-x86_64-unknown-linux-gnu.$extension"
  if bash "$script_dir/version_release_artifacts.sh" "$bad_dir" v2.4.0; then
    echo "mismatched $extension version was accepted" >&2
    exit 1
  fi
done
printf 'Release artifact names: ok\n'
