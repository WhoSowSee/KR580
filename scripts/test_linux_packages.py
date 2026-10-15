import io
from pathlib import Path
import re
import struct
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import linux_packaging as packages
from linux_packaging import elf, verify


class LinuxPackageTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.binaries = self.root / "binaries"
        self.binaries.mkdir()
        for name in ("kr580", "kr"):
            (self.binaries / name).write_bytes((name + " release executable").encode())

    def prepare(
        self, package_format="arch", target="x86_64-unknown-linux-gnu", staging=None
    ):
        staging = staging or self.root / "stage"
        with patch.object(packages, "inspect_binary", return_value=("2.34", True)):
            packages.prepare(package_format, target, staging, self.binaries)
        return staging

    def test_workspace_metadata_and_source_checksum_are_used(self):
        staging = self.prepare()
        recipe = (staging / "PKGBUILD").read_text()
        metadata, _, _, _ = self.context()
        self.assertIn("pkgver=" + metadata["package_version"], recipe)
        self.assertIn(packages.sha256(staging / "payload.tar"), recipe)
        self.assertNotIn("SKIP", recipe)
        self.assertNotIn("@PAYLOAD", recipe)

    def context(self, package_format="arch"):
        with patch.object(packages, "inspect_binary", return_value=("2.34", True)):
            return packages.context(
                package_format, "x86_64-unknown-linux-gnu", self.binaries
            )

    def test_version_comes_from_workspace_instead_of_a_release_tag(self):
        original = packages.ROOT
        for name in ["Cargo.toml", "crates/ui/Cargo.toml"]:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes((original / name).read_bytes())
        manifest = self.root / "Cargo.toml"
        text = re.sub(
            r'(?m)^version = "[^"]+"$',
            'version = "1.2.3"',
            manifest.read_text(),
            count=1,
        )
        manifest.write_text(text)
        with patch.object(packages, "ROOT", self.root):
            metadata, _, _, _ = self.context()
        self.assertEqual(metadata["version"], "1.2.3")

    def test_both_rpm_targets_and_arch_limit_are_enforced(self):
        for target in packages.TARGETS:
            staging = self.prepare("rpm", target, self.root / target)
            self.assertTrue((staging / "SPECS/kr580.spec").is_file())
        with self.assertRaisesRegex(ValueError, "x86_64"):
            packages.context("arch", "aarch64-unknown-linux-gnu", self.binaries)

    def test_prerelease_versions_fit_both_package_manager_syntaxes(self):
        self.assertEqual(packages.package_version("rpm", "1.2.3-rc.1"), "1.2.3~rc.1")
        self.assertEqual(packages.package_version("arch", "1.2.3-rc.1"), "1.2.3rc.1")
        self.assertEqual(
            packages.package_version("rpm", "1.2.3-rc-1+build-2"), "1.2.3~rc.1+build.2"
        )
        self.assertEqual(
            packages.package_version("arch", "1.2.3-rc-1+build-2"), "1.2.3rc.1+build.2"
        )
        for package_format in ["rpm", "arch"]:
            self.assertEqual(packages.package_version(package_format, "1.2.3"), "1.2.3")
            with self.assertRaises(ValueError):
                packages.package_version(package_format, "1.2.3\nother")

    def test_nonempty_staging_is_preserved(self):
        staging = self.root / "stage"
        staging.mkdir()
        (staging / "owned").write_text("keep")
        with self.assertRaisesRegex(ValueError, "must be empty"):
            self.prepare(staging=staging)
        self.assertEqual((staging / "owned").read_text(), "keep")

    def test_launchers_select_user_settings_and_public_gui_for_both_formats(self):
        for package_format in ["rpm", "arch"]:
            files = packages.payload_files(package_format, self.binaries)
            self.assertEqual(len(files), 9)
            self.assertNotIn("install.json", " ".join(files))
            for name in ["kr580", "kr"]:
                wrapper, mode = files[f"usr/bin/{name}"]
                self.assertIn(f"KR580_PACKAGE_KIND={package_format}".encode(), wrapper)
                self.assertIn(b"KR580_GUI_EXECUTABLE=/usr/bin/kr580", wrapper)
                self.assertIn(b'"$@"', wrapper)
                self.assertEqual(mode, 0o755)
                self.assertEqual(
                    files[f"usr/lib/kr580/{name}"][0],
                    (self.binaries / name).read_bytes(),
                )
            desktop = files["usr/share/applications/kr580.desktop"][0]
            self.assertIn(b"Exec=/usr/bin/kr580 %f", desktop)
            self.assertNotIn(b"@EXEC@", desktop)

    def test_elf_architecture_and_glibc_baseline_are_checked(self):
        binary = self.binaries / "kr580"
        for machine, target in [
            (62, "x86_64-unknown-linux-gnu"),
            (183, "aarch64-unknown-linux-gnu"),
        ]:
            binary.write_bytes(
                b"\x7fELF\x02\x01" + b"\0" * 12 + struct.pack("<H", machine)
            )
            dynamic = "(NEEDED) [libc.so.6]\n(NEEDED) [libgcc_s.so.1]"
            with patch.object(
                elf, "command", side_effect=[dynamic, "Name: GLIBC_2.34"]
            ):
                self.assertEqual(elf.inspect_binary(binary, target), ("2.34", True))
            with patch.object(
                elf, "command", side_effect=[dynamic, "Name: GLIBC_2.40"]
            ):
                with self.assertRaisesRegex(ValueError, "exceeding"):
                    elf.inspect_binary(binary, target)
            with self.assertRaisesRegex(ValueError, "machine"):
                other = (
                    "aarch64-unknown-linux-gnu"
                    if machine == 62
                    else "x86_64-unknown-linux-gnu"
                )
                elf.inspect_binary(binary, other)
        binary.write_bytes(b"MZ" + b"\0" * 30)
        with self.assertRaisesRegex(ValueError, "ELF64"):
            elf.inspect_binary(binary, "x86_64-unknown-linux-gnu")

    def arch_info(self):
        metadata, _, arch, _ = self.context()
        fields = {
            "pkgname": metadata["name"],
            "pkgver": metadata["package_version"] + "-1",
            "arch": arch,
            "license": metadata["license"],
        }
        text = "".join(f"{key} = {value}\n" for key, value in fields.items())
        return text + "".join(
            f"depend = {name}\n"
            for name in ["glibc>=2.39", *packages.DEPENDENCIES["arch"], "libgcc"]
        )

    def test_arch_metadata_rejects_wrong_architecture_missing_runtime_and_duplicate_fields(
        self,
    ):
        metadata, _, arch, _ = self.context()
        info = self.arch_info()
        verify.check_arch_metadata(info, metadata, arch, True)
        for changed in [
            info.replace("arch = x86_64", "arch = aarch64"),
            info.replace("depend = fontconfig\n", ""),
            info + "pkgname = other\n",
        ]:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                verify.check_arch_metadata(changed, metadata, arch, True)

    def test_rpm_metadata_checks_declared_and_automatic_runtime_dependencies(self):
        metadata, _, _, _ = self.context("rpm")
        fields = f"kr580\n{metadata['package_version']}\n1\nx86_64\nMIT\n"
        requires = {
            "glibc >= 2.39",
            "libc.so.6()(64bit)",
            "libgcc_s.so.1()(64bit)",
            "libc.so.6(GLIBC_2.34)(64bit)",
            *packages.DEPENDENCIES["rpm"],
        }
        for removed in [
            None,
            "fontconfig",
            "libgcc_s.so.1()(64bit)",
            "libc.so.6(GLIBC_2.34)(64bit)",
        ]:
            actual = requires - ({removed} if removed else set())
            with patch.object(
                verify, "command", side_effect=["OK", fields, "\n".join(actual)]
            ):
                if removed:
                    with self.assertRaises(ValueError):
                        verify.check_rpm_metadata(
                            Path("app.rpm"), metadata, "x86_64", [("2.34", True)]
                        )
                else:
                    verify.check_rpm_metadata(
                        Path("app.rpm"), metadata, "x86_64", [("2.34", True)]
                    )

    def archive(self, files, extra=None):
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode="w") as archive:
            for name, (content, mode) in files.items():
                entry = tarfile.TarInfo(name)
                entry.size, entry.mode = len(content), mode
                archive.addfile(entry, io.BytesIO(content))
            if extra:
                archive.addfile(extra)
        return stream.getvalue()

    def test_arch_verification_checks_metadata_and_payload_together(self):
        files = packages.payload_files("arch", self.binaries)
        info = self.arch_info().encode()
        files[".PKGINFO"] = (info, 0o644)
        duplicate = tarfile.TarInfo(".PKGINFO")
        link = tarfile.TarInfo(".PKGINFO")
        link.type, link.linkname = tarfile.SYMTYPE, "/etc/passwd"
        cases = [
            ("valid", files, None),
            (
                "missing",
                {key: value for key, value in files.items() if key != ".PKGINFO"},
                None,
            ),
            ("empty", {**files, ".PKGINFO": (b"", 0o644)}, None),
            (
                "wrong arch",
                {
                    **files,
                    ".PKGINFO": (
                        info.replace(b"arch = x86_64", b"arch = aarch64"),
                        0o644,
                    ),
                },
                None,
            ),
            (
                "tampered binary",
                {**files, "usr/lib/kr580/kr580": (b"tampered", 0o755)},
                None,
            ),
            ("duplicate", files, duplicate),
            (
                "link",
                {key: value for key, value in files.items() if key != ".PKGINFO"},
                link,
            ),
        ]
        with patch.object(packages, "inspect_binary", return_value=("2.34", True)):
            for name, contents, extra in cases:
                with (
                    self.subTest(name=name),
                    patch.object(
                        verify, "command", return_value=self.archive(contents, extra)
                    ),
                ):
                    args = (
                        "arch",
                        "x86_64-unknown-linux-gnu",
                        self.root / "app.pkg.tar.zst",
                        self.binaries,
                    )
                    if name == "valid":
                        verify.verify(*args)
                    else:
                        with self.assertRaises(ValueError):
                            verify.verify(*args)

    def test_payload_rejects_modified_binary_license_permissions_missing_files_and_links(
        self,
    ):
        expected = packages.payload_files("rpm", self.binaries)
        verify.check_payload(self.archive(expected), expected, "rpm")
        for name in [
            "usr/lib/kr580/kr580",
            "usr/share/licenses/kr580/LICENSE",
            "usr/bin/kr",
        ]:
            changed = dict(expected)
            changed[name] = (b"tampered", expected[name][1])
            with self.assertRaisesRegex(ValueError, "differ"):
                verify.check_payload(self.archive(changed), expected, "rpm")
        changed = dict(expected)
        changed["usr/bin/kr"] = (expected["usr/bin/kr"][0], 0o777)
        with self.assertRaises(ValueError):
            verify.check_payload(self.archive(changed), expected, "rpm")
        changed.pop("usr/bin/kr")
        with self.assertRaisesRegex(ValueError, "Missing"):
            verify.check_payload(self.archive(changed), expected, "rpm")
        for name in ["../escape", "usr/bin/extra", "usr/lib/kr580/install.json"]:
            extra = tarfile.TarInfo(name)
            extra.type, extra.linkname = tarfile.SYMTYPE, "/etc/passwd"
            with self.assertRaisesRegex(ValueError, "Unexpected"):
                verify.check_payload(self.archive(expected, extra), expected, "rpm")


if __name__ == "__main__":
    unittest.main()
