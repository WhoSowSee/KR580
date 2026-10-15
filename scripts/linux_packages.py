import argparse
from pathlib import Path

from linux_packaging import prepare
from linux_packaging.elf import TARGETS
from linux_packaging.verify import verify


def main():
    parser = argparse.ArgumentParser(
        description="Prepare and verify native KR580 RPM/Arch packages."
    )
    parser.add_argument("action", choices=("prepare", "verify"))
    parser.add_argument("format", choices=("rpm", "arch"))
    parser.add_argument("target", choices=TARGETS)
    parser.add_argument("path", type=Path)
    parser.add_argument("--binary-dir", type=Path)
    args = parser.parse_args()
    operation = prepare if args.action == "prepare" else verify
    operation(args.format, args.target, args.path, args.binary_dir)


if __name__ == "__main__":
    main()
