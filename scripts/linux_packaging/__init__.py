import hashlib
import io
import os
from pathlib import Path
import re
import shlex
import tarfile
import tomllib

from .elf import GLIBC_MINIMUM, TARGETS, inspect_binary


ROOT = Path(__file__).resolve().parents[2]
RECIPES = ROOT / "scripts/packaging"
DEPENDENCIES = {
    "rpm": [
        "fontconfig",
        "libX11",
        "libXcursor",
        "libXrandr",
        "libXi",
        "libxcb",
        "libxkbcommon",
        "libxkbcommon-x11",
        "libwayland-client",
        "libwayland-cursor",
        "libwayland-egl",
        "libglvnd-glx",
        "libglvnd-egl",
        "vulkan-loader",
        "dbus-libs",
        "zenity",
        "desktop-file-utils",
        "shared-mime-info",
        "hicolor-icon-theme",
        "xdg-utils",
    ],
    "arch": [
        "bash",
        "fontconfig",
        "libx11",
        "libxcursor",
        "libxrandr",
        "libxi",
        "libxcb",
        "libxkbcommon",
        "libxkbcommon-x11",
        "wayland",
        "libglvnd",
        "vulkan-icd-loader",
        "dbus",
        "zenity",
        "desktop-file-utils",
        "shared-mime-info",
        "hicolor-icon-theme",
        "xdg-utils",
    ],
}


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def package_version(package_format, version):
    parts = re.fullmatch(
        r"(\d+\.\d+\.\d+)(?:-([0-9A-Za-z.-]+))?(?:\+([0-9A-Za-z.-]+))?", version
    )
    if not parts:
        raise ValueError(f"Invalid workspace release version: {version}")
    core, prerelease, build = parts.groups()
    if prerelease:
        core += ("~" if package_format == "rpm" else "") + prerelease.replace("-", ".")
    return core + ("+" + build.replace("-", ".") if build else "")


def context(package_format, target, binary_dir=None):
    arch = TARGETS[target][0]
    if package_format == "arch" and arch != "x86_64":
        raise ValueError("Arch Linux packages require the x86_64 GNU target")
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))[
        "workspace"
    ]["package"]
    package = tomllib.loads(
        (ROOT / "crates/ui/Cargo.toml").read_text(encoding="utf-8")
    )["package"]
    metadata = {key: workspace[key] for key in ("version", "license", "repository")}
    metadata.update(name=package["name"], description=package["description"])
    metadata["package_version"] = package_version(package_format, metadata["version"])
    binary_dir = binary_dir or ROOT / "target" / target / "release"
    requirements = [
        inspect_binary(binary_dir / name, target) for name in ("kr580", "kr")
    ]
    return metadata, binary_dir, arch, requirements


def payload_files(package_format, binary_dir):
    files = {}
    for name in ("kr580", "kr"):
        files[f"usr/lib/kr580/{name}"] = ((binary_dir / name).read_bytes(), 0o755)
        wrapper = (
            "#!/bin/sh\n"
            f"export KR580_PACKAGE_KIND={package_format}\n"
            "export KR580_GUI_EXECUTABLE=/usr/bin/kr580\n"
            f'exec /usr/lib/kr580/{name} "$@"\n'
        )
        files[f"usr/bin/{name}"] = (wrapper.encode(), 0o755)
    sources = {
        "usr/share/icons/hicolor/256x256/apps/kr580.png": "crates/ui/assets/icons/icon-256.png",
        "usr/share/icons/hicolor/256x256/mimetypes/application-x-kr580.png": "crates/ui/assets/icons/file-580-256.png",
        "usr/share/mime/packages/application-x-kr580.xml": "crates/ui/assets/linux/application-x-kr580.xml",
        "usr/share/licenses/kr580/LICENSE": "LICENSE",
    }
    files.update(
        {
            name: ((ROOT / source).read_bytes(), 0o644)
            for name, source in sources.items()
        }
    )
    desktop = (ROOT / "crates/ui/assets/linux/kr580-package.desktop").read_text(
        encoding="utf-8"
    )
    if desktop.count("@EXEC@") != 1:
        raise ValueError(
            "Expected one executable placeholder in the canonical Desktop Entry"
        )
    files["usr/share/applications/kr580.desktop"] = (
        desktop.replace("@EXEC@", "/usr/bin/kr580").encode(),
        0o644,
    )
    return files


def prepare(package_format, target, directory, binary_dir=None):
    metadata, binary_dir, arch, requirements = context(
        package_format, target, binary_dir
    )
    if directory.exists() and any(directory.iterdir()):
        raise ValueError(f"Package staging directory must be empty: {directory}")
    files = payload_files(package_format, binary_dir)
    sources = directory / "SOURCES" if package_format == "rpm" else directory
    sources.mkdir(parents=True, exist_ok=True)
    payload = sources / "payload.tar"
    with tarfile.open(payload, "w") as archive:
        for name, (content, mode) in sorted(files.items()):
            entry = tarfile.TarInfo(name)
            entry.size, entry.mode = len(content), mode
            entry.mtime = int(os.environ.get("SOURCE_DATE_EPOCH", "0"))
            archive.addfile(entry, io.BytesIO(content))
    fields = {key.upper(): value for key, value in metadata.items()}
    fields["GLIBC_MINIMUM"] = GLIBC_MINIMUM
    if package_format == "rpm":
        specs = directory / "SPECS"
        specs.mkdir()
        recipe, output = RECIPES / "kr580.spec.in", specs / "kr580.spec"
        fields = {key: value.replace("%", "%%") for key, value in fields.items()}
        fields["DEPENDENCIES"] = ", ".join(DEPENDENCIES["rpm"])
    else:
        recipe, output = RECIPES / "PKGBUILD.in", directory / "PKGBUILD"
        fields = {key: shlex.quote(value) for key, value in fields.items()}
        dependencies = [f"glibc>={GLIBC_MINIMUM}"] + DEPENDENCIES["arch"]
        if any(needs_gcc for _, needs_gcc in requirements):
            dependencies.append("libgcc")
        fields["DEPENDENCIES"] = " ".join(map(shlex.quote, dependencies))
        fields["PAYLOAD_SHA256"] = sha256(payload)
    output.write_text(
        re.sub(
            r"@([A-Z_][A-Z_0-9]*)@", lambda match: fields[match[1]], recipe.read_text()
        ),
        encoding="utf-8",
        newline="\n",
    )
    print(f"Prepared {package_format}: {output} ({arch})")
