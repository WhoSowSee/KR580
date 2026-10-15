import io
from pathlib import PurePosixPath
import re
import tarfile

from . import DEPENDENCIES, context, payload_files
from .elf import GLIBC_MINIMUM, command, version_tuple


def parse_pkginfo(text):
    fields = {}
    for line in text.splitlines():
        if line and not line.startswith("#"):
            key, value = line.split(" = ", 1)
            fields.setdefault(key, []).append(value)
    return fields


def check_arch_metadata(text, metadata, arch, needs_gcc):
    fields = parse_pkginfo(text)
    expected = {
        "pkgname": [metadata["name"]],
        "pkgver": [metadata["package_version"] + "-1"],
        "arch": [arch],
        "license": [metadata["license"]],
    }
    dependencies = {f"glibc>={GLIBC_MINIMUM}", *DEPENDENCIES["arch"]}
    if needs_gcc:
        dependencies.add("libgcc")
    if any(fields.get(key) != value for key, value in expected.items()):
        raise ValueError(f"Unexpected Arch package metadata: {fields}")
    if set(fields.get("depend", [])) != dependencies:
        raise ValueError(
            f"Unexpected Arch runtime dependencies: {fields.get('depend')}"
        )


def check_rpm_metadata(package, metadata, arch, requirements):
    command("rpmkeys", "--checksig", "--nosignature", str(package))
    actual = command(
        "rpm",
        "-qp",
        "--queryformat",
        "%{NAME}\n%{VERSION}\n%{RELEASE}\n%{ARCH}\n%{LICENSE}\n",
        str(package),
    ).splitlines()
    if actual != [
        metadata["name"],
        metadata["package_version"],
        "1",
        arch,
        metadata["license"],
    ]:
        raise ValueError(f"Unexpected RPM metadata: {actual}")
    requires = set(command("rpm", "-qp", "--requires", str(package)).splitlines())
    required = {f"glibc >= {GLIBC_MINIMUM}", "libc.so.6()(64bit)", *DEPENDENCIES["rpm"]}
    if any(needs_gcc for _, needs_gcc in requirements):
        required.add("libgcc_s.so.1()(64bit)")
    versions = re.findall(r"\(GLIBC_(\d+(?:\.\d+)+)\)\(64bit\)", "\n".join(requires))
    if (
        not required <= requires
        or not versions
        or max(map(version_tuple, versions))
        < max(version_tuple(minimum) for minimum, _ in requirements)
    ):
        raise ValueError(
            f"RPM dependencies do not cover the application: {sorted(requires)}"
        )


def check_payload(payload, expected, package_format):
    allowed_metadata = (
        {".PKGINFO", ".BUILDINFO", ".MTREE"} if package_format == "arch" else set()
    )
    directories = {"", "."} | {
        str(parent) for name in expected for parent in PurePosixPath(name).parents
    }
    seen = set()
    pkginfo = None
    with tarfile.open(fileobj=io.BytesIO(payload), mode="r:") as archive:
        for entry in archive:
            name = entry.name.removeprefix("./")
            if entry.isdir() and name in directories:
                continue
            if (
                name in seen
                or not entry.isfile()
                or name not in expected.keys() | allowed_metadata
            ):
                raise ValueError(f"Unexpected package payload entry: {entry.name}")
            seen.add(name)
            if name in expected:
                content, mode = expected[name]
                if entry.mode != mode or archive.extractfile(entry).read() != content:
                    raise ValueError(f"Packaged content or permissions differ: {name}")
            elif name == ".PKGINFO":
                pkginfo = archive.extractfile(entry).read()
        if not expected.keys() <= seen:
            raise ValueError(f"Missing package files: {sorted(expected.keys() - seen)}")
        if package_format == "arch" and pkginfo is None:
            raise ValueError("Missing .PKGINFO")
    return pkginfo


def verify(package_format, target, package, binary_dir=None):
    metadata, binary_dir, arch, requirements = context(
        package_format, target, binary_dir
    )
    package = package.resolve()
    if package_format == "rpm":
        check_rpm_metadata(package, metadata, arch, requirements)
        payload = command("rpm2archive", "--nocompression", str(package), text=False)
    else:
        payload = command(
            "bsdtar", "--format", "pax", "-cf", "-", f"@{package}", text=False
        )
    pkginfo = check_payload(
        payload, payload_files(package_format, binary_dir), package_format
    )
    if package_format == "arch":
        check_arch_metadata(
            pkginfo.decode(), metadata, arch, any(gcc for _, gcc in requirements)
        )
    print(
        f"Verified {package.name}: {arch}, glibc>={GLIBC_MINIMUM}, application/desktop/MIME/icon identity"
    )
