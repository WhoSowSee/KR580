#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 || ! -f /.dockerenv || $(id -u) -ne 0 ]]; then
  echo "usage (inside a disposable root Docker container): $0 <rpm|arch> <package>" >&2
  exit 2
fi

package_format="$1"
package="$2"
case "$package_format" in
  rpm)
    dnf5 --assumeyes install python3 shadow-utils util-linux xorg-x11-server-Xvfb xorg-x11-xauth
    rpm --version
    dnf5 --assumeyes --no-gpgchecks install "$package"
    ;;
  arch)
    pacman -Syu --noconfirm --needed python xorg-server-xvfb xorg-xauth
    pacman -U --noconfirm "$package"
    ;;
  *) exit 2 ;;
esac

expected=$(python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["workspace"]["package"]["version"])')
test "$(/usr/bin/kr --version)" = "kr $expected"
test ! -e /usr/lib/kr580/install.json
test ! -e /usr/lib/kr580/k580-installer
desktop-file-validate /usr/share/applications/kr580.desktop
grep -Fq 'application/x-kr580:*.580' /usr/share/mime/globs2
grep -Fq 'application/x-kr580:*.krs' /usr/share/mime/globs2
grep -Fq 'kr580.desktop' /usr/share/applications/mimeinfo.cache
python3 - <<'PY'
import ctypes
for name in ["libfontconfig.so.1", "libX11.so.6", "libXcursor.so.1", "libXrandr.so.2",
             "libXi.so.6", "libxcb.so.1", "libxkbcommon.so.0", "libxkbcommon-x11.so.0",
             "libwayland-client.so.0", "libwayland-cursor.so.0", "libwayland-egl.so.1",
             "libGL.so.1", "libEGL.so.1", "libvulkan.so.1", "libdbus-1.so.3"]:
    ctypes.CDLL(name)
PY

useradd --create-home kr580-smoke
runuser -u kr580-smoke -- bash -euo pipefail -c '
  mkdir -p "$HOME/.config/kr580" "$HOME/runtime"
  chmod 700 "$HOME/runtime"
  export XDG_RUNTIME_DIR="$HOME/runtime"
  printf "preserve user data\n" > "$HOME/.config/kr580/package-smoke-sentinel"
  status=0
  timeout 10s xvfb-run -a env ICED_BACKEND=tiny-skia /usr/bin/kr580 > "$HOME/gui.log" 2>&1 || status=$?
  if [[ $status -ne 124 ]]; then
    cat "$HOME/gui.log" >&2
    echo "GUI exited before the smoke-test timeout: $status" >&2
    exit 1
  fi
  test ! -e /usr/lib/kr580/settings.json
'

case "$package_format" in
  rpm) dnf5 --assumeyes remove kr580 ;;
  arch) pacman -R --noconfirm kr580 ;;
esac
test ! -e /usr/bin/kr
test ! -e /usr/bin/kr580
test ! -e /usr/share/applications/kr580.desktop
test ! -e /usr/share/mime/packages/application-x-kr580.xml
test "$(cat /home/kr580-smoke/.config/kr580/package-smoke-sentinel)" = "preserve user data"
if [[ -f /usr/share/applications/mimeinfo.cache ]] && grep -Fq 'kr580.desktop' /usr/share/applications/mimeinfo.cache; then
  echo "Desktop cache still references the removed package" >&2
  exit 1
fi
if [[ -f /usr/share/mime/globs2 ]] && grep -Fq 'application/x-kr580:' /usr/share/mime/globs2; then
  echo "MIME cache still references the removed package" >&2
  exit 1
fi
printf '%s package installation, GUI, metadata, removal, and user-data preservation: ok\n' "$package_format"
