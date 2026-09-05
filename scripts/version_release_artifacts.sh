#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 || ! -d "$1" || -z "$2" ]]; then
  echo "usage: $0 <artifact-directory> <tag-version>" >&2
  exit 2
fi

artifact_dir="$1"
version="$2"
package_version="${version#v}"

shopt -s nullglob
for file in "$artifact_dir"/*; do
  base="$(basename "$file")"
  case "$base" in
    KR580-Setup-"$version"-*) continue ;;
    KR580-Setup-*)
      target="$artifact_dir/KR580-Setup-$version-${base#KR580-Setup-}"
      if [[ -e "$target" ]]; then
        echo "release artifact already exists: $target" >&2
        exit 1
      fi
      mv "$file" "$target"
      ;;
    KR580-*.dmg|KR580-*.snap)
      if [[ "$base" != KR580-"$package_version"-* ]]; then
        echo "release artifact version does not match $version: $base" >&2
        exit 1
      fi
      ;;
  esac
done
