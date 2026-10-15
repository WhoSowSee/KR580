import os
import re
import struct
import subprocess


TARGETS = {
    "x86_64-unknown-linux-gnu": ("x86_64", 62),
    "aarch64-unknown-linux-gnu": ("aarch64", 183),
}
GLIBC_MINIMUM = "2.39"
GLIBC_LIBRARIES = {
    "libc.so.6",
    "libm.so.6",
    "libpthread.so.0",
    "libdl.so.2",
    "librt.so.1",
    "libutil.so.1",
    "ld-linux-x86-64.so.2",
    "ld-linux-aarch64.so.1",
}


def command(*args, text=True):
    return subprocess.check_output(args, text=text, env={**os.environ, "LC_ALL": "C"})


def version_tuple(version):
    return tuple(map(int, version.split(".")))


def inspect_binary(binary, target):
    with binary.open("rb") as stream:
        header = stream.read(20)
    if len(header) < 20 or header[:6] != b"\x7fELF\x02\x01":
        raise ValueError(f"Expected a little-endian ELF64 executable: {binary}")
    if struct.unpack("<H", header[18:20])[0] != TARGETS[target][1]:
        raise ValueError(f"Unexpected ELF machine for {target}: {binary}")
    dynamic = command("readelf", "--wide", "--dynamic", str(binary))
    needed = set(re.findall(r"\(NEEDED\).*?\[(.*?)\]", dynamic))
    unknown = needed - GLIBC_LIBRARIES - {"libgcc_s.so.1"}
    if unknown or "libc.so.6" not in needed:
        raise ValueError(
            f"Unexpected linked runtime libraries for {binary}: {sorted(needed)}"
        )
    versions = re.findall(
        r"Name: GLIBC_(\S+)",
        command("readelf", "--wide", "--version-info", str(binary)),
    )
    if not versions or any(
        not re.fullmatch(r"\d+(?:\.\d+)+", version) for version in versions
    ):
        raise ValueError(f"Cannot determine public glibc requirements: {binary}")
    minimum = max(versions, key=version_tuple)
    if version_tuple(minimum) > version_tuple(GLIBC_MINIMUM):
        raise ValueError(
            f"{binary} requires glibc {minimum}, exceeding {GLIBC_MINIMUM}"
        )
    return minimum, "libgcc_s.so.1" in needed
