# RPM and Arch Linux packages

The release workflow builds native application packages in addition to the
standalone Linux setup ZIPs and DEBs. Download the package matching the machine
from [GitHub Releases](https://github.com/WhoSowSee/KR580/releases).

| Format | Targets | Installation checked in CI |
|---|---|---|
| RPM | `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` | Fedora 44 |
| `.pkg.tar.zst` | `x86_64-unknown-linux-gnu` | Arch Linux |

Packages require glibc 2.39 or newer and a graphical desktop. Rust is only a
build dependency. RPM compatibility with other distributions is subject to
their library versions and dependency names; CI currently checks Fedora.
Arch Linux ARM is not part of the Arch package matrix.

## Install, update and remove

Replace `<version>` and choose the matching target in these commands. Run them
from the directory containing the downloaded package.

Fedora:

```sh
sudo dnf install "./KR580-<version>-x86_64-unknown-linux-gnu.rpm"
kr --version
kr
sudo dnf remove kr580
```

Arch Linux:

```sh
sudo pacman -Syu
sudo pacman -U "./KR580-<version>-x86_64-unknown-linux-gnu.pkg.tar.zst"
kr --version
kr
sudo pacman -R kr580
```

Install a newer downloaded package with the same install command to update.
These are release assets, with no project package repository or AUR entry
configured. The application's Help menu can open the release page when an
update is available; the package manager applies the update.

## Installed files and settings

Both formats install the same application payload:

| Path | Purpose |
|---|---|
| `/usr/bin/kr580`, `/usr/bin/kr` | GUI and terminal launcher wrappers |
| `/usr/lib/kr580/kr580`, `/usr/lib/kr580/kr` | Compiled executables |
| `/usr/share/applications/kr580.desktop` | Desktop launcher and Open With handler |
| `/usr/share/mime/packages/application-x-kr580.xml` | `.580` and `.krs` MIME definitions |
| `/usr/share/icons/hicolor/256x256/{apps,mimetypes}/` | Application and document icons |
| `/usr/share/licenses/kr580/LICENSE` | Project license |

The desktop and MIME files are rendered from the canonical assets under
`crates/ui/assets/linux/`. Distribution file triggers refresh the desktop and
MIME caches. RPM also refreshes both caches in its final-removal scriptlet,
with ordered tool dependencies so DNF can remove unused tools afterwards.
Installing a package does not assign a new user default handler;
use Open With or the application's Set as default action when desired.

The wrappers set `KR580_PACKAGE_KIND=rpm` or `arch`, and
`KR580_GUI_EXECUTABLE=/usr/bin/kr580`. This preserves the package environment
when opening files through the terminal launcher or registered associations.
`package_kind.rs` shares recognition of these markers with Nix between settings
storage and the updater. Settings use `$XDG_CONFIG_HOME/kr580/settings.json`
when the XDG path is absolute and nonempty, otherwise
`~/.config/kr580/settings.json`. Uninstalling preserves user settings and
documents. The packages contain no `install.json`, setup or uninstaller.

Runtime dependencies include X11/Wayland, fontconfig, graphics loaders, D-Bus,
the Zenity dialog fallback, and freedesktop metadata tools. Graphics drivers and
the desktop portal backend are supplied by the installed desktop environment.

## Build locally

Use Python 3.11 or newer, GNU binutils, and a GNU Linux release build produced
against glibc 2.39 or older. CI uses native Ubuntu 24.04 runners for both CPU
architectures and checks the ELF machine and symbol requirements before
packaging. A build against newer glibc is rejected if it requires newer symbols.

For an x86_64 RPM, install `rpm` tools and build from the repository root:

```sh
target=x86_64-unknown-linux-gnu
cargo build --locked --release -p kr580 --bin kr580 --bin kr --target "$target"
stage=$(mktemp -d)
python3 scripts/linux_packages.py prepare rpm "$target" "$stage"
rpmbuild -bb --target x86_64 --define "_topdir $stage" "$stage/SPECS/kr580.spec"
python3 scripts/linux_packages.py verify rpm "$target" "$stage/RPMS/x86_64/"*.rpm
```

For a native aarch64 build, use `aarch64-unknown-linux-gnu`, `--target aarch64`
for rpmbuild, and `RPMS/aarch64`. These are native builds; the existing setup
matrix continues to use cross compilation separately.

On Arch Linux, run makepkg as an ordinary user with the recipe's runtime
dependencies available, or allow makepkg to install them with `--syncdeps`:

```sh
target=x86_64-unknown-linux-gnu
cargo build --locked --release -p kr580 --bin kr580 --bin kr --target "$target"
repo=$PWD
stage=$(mktemp -d)
python3 scripts/linux_packages.py prepare arch "$target" "$stage"
(cd "$stage" && PKGEXT=.pkg.tar.zst makepkg --syncdeps)
python3 "$repo/scripts/linux_packages.py" verify arch "$target" "$stage/"*.pkg.tar.zst
```

`--binary-dir <directory>` overrides the default `target/<target>/release`
input directory for either prepare or verify. Staging must be empty. Package
metadata comes from the tracked Cargo manifests.
RPM encodes a prerelease separator as `~` (`1.2.3~rc.1`); Arch removes it
(`1.2.3rc.1`). Release filenames retain the original Cargo version.
Hyphens inside prerelease/build identifiers become dots in package metadata.
The recipes under `scripts/packaging/` disable stripping and debug splitting so verification can
compare both executables byte for byte. Arch's source archive has a real SHA-256
checksum. RPM retains automatic ELF dependency generation in addition to the
explicit dynamically loaded GUI dependencies.

## Verification and release wiring

`scripts/linux_packages.py verify` checks the package identity, version,
architecture, license, declared dependencies, executable ELF/glibc requirements,
and the exact payload contents and permissions. Unexpected files, links,
duplicates and missing entries fail verification.

`scripts/test_linux_packages.py` tests those rejection paths without native
packaging tools. Run `just test-packaging` on Unix, or the portable Python part:

```sh
python3 -B -m unittest discover -s scripts -p test_linux_packages.py -v
```

`.github/workflows/build-linux.yml` keeps the seven existing setup targets and
adds two native package jobs. RPMs are installed in disposable `fedora:44`
containers. The x86_64 job creates the Arch package with an unprivileged makepkg
user in `archlinux:base-devel`, then installs it with pacman. Its binary-only
packaging step uses `--nodeps`; the subsequent real installation resolves and
checks all runtime dependencies.

`scripts/smoke_linux_package.sh` requires a disposable root Docker container.
It verifies the launcher version, desktop/MIME caches and dynamic libraries,
runs the GUI as an ordinary user under Xvfb, removes the package, and checks
that managed files/cache entries disappear while user data remains. Do not run
this script against a host installation.

The reusable workflow uploads three additional release files:
`KR580-<version>-<target>.rpm` for both targets and
`KR580-<version>-x86_64-unknown-linux-gnu.pkg.tar.zst`. The parent release job
waits for the whole Linux workflow, validates their version against the tag,
and publishes them alongside the existing formats. Tool versions and resolved
container image digests are printed in the build log.
