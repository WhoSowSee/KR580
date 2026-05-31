#!/usr/bin/env python3
"""Build square.580 — a KR580 program that draws an 8×8 hollow square
(outline only, no fill) on the original-protocol monitor.

Real KP580 monitor protocol (from `KP580_Help.chm` →
`Prog_Wrk_Peref.htm`): port 00h consumes 2- or 3-byte commands.
  - first byte: bit7 = 0 → text command; bit7 = 1 → graphics command;
    bits 0..6 carry colour intensity (0..127).
  - text:     [0 ccccccc] [char_oem]                (39×20 grid)
  - graphics: [1 ccccccc] [x] [y]                   (256×256 grid)

This program switches no global mode (each command is self-contained).
It emits 28 graphics commands `FF, X, Y` covering the four edges of an
8×8 square anchored at origin (0,0):
  - top    (y=0):     X = 0..7
  - bottom (y=7):     X = 0..7
  - left   (x=0):     Y = 1..6
  - right  (x=7):     Y = 1..6

The source uses a tiny two-pass assembler so opcodes can be relayed
without manual address arithmetic.
"""
import struct
from pathlib import Path


SIZES = {
    "MVI_A": 2, "MVI_B": 2, "MVI_C": 2, "MVI_D": 2, "MVI_E": 2,
    "MOV_A_B": 1, "MOV_A_C": 1, "MOV_A_D": 1, "MOV_A_E": 1,
    "OUT": 2,
    "INR_B": 1, "INR_C": 1, "INR_D": 1, "INR_E": 1,
    "DCR_B": 1, "DCR_C": 1, "DCR_D": 1,
    "JNZ": 3, "JMP": 3,
    "CALL": 3, "RET": 1,
    "LXI_SP": 3,
    "HLT": 1,
    "NOP": 1,
    "XRA_A": 1,
}


def _u8(x):
    assert isinstance(x, int) and 0 <= x <= 0xFF, f"u8 out of range: {x!r}"
    return bytes([x])


def _u16(x, labels):
    if isinstance(x, str):
        x = labels[x]
    assert 0 <= x <= 0xFFFF, f"u16 out of range: {x:#x}"
    return bytes([x & 0xFF, (x >> 8) & 0xFF])


def encode(mnem, args, labels):
    if mnem == "MVI_A":   return bytes([0x3E]) + _u8(args[0])
    if mnem == "MVI_B":   return bytes([0x06]) + _u8(args[0])
    if mnem == "MVI_C":   return bytes([0x0E]) + _u8(args[0])
    if mnem == "MVI_D":   return bytes([0x16]) + _u8(args[0])
    if mnem == "MVI_E":   return bytes([0x1E]) + _u8(args[0])
    if mnem == "MOV_A_B": return bytes([0x78])
    if mnem == "MOV_A_C": return bytes([0x79])
    if mnem == "MOV_A_D": return bytes([0x7A])
    if mnem == "MOV_A_E": return bytes([0x7B])
    if mnem == "OUT":     return bytes([0xD3]) + _u8(args[0])
    if mnem == "INR_B":   return bytes([0x04])
    if mnem == "INR_C":   return bytes([0x0C])
    if mnem == "INR_D":   return bytes([0x14])
    if mnem == "INR_E":   return bytes([0x1C])
    if mnem == "DCR_B":   return bytes([0x05])
    if mnem == "DCR_C":   return bytes([0x0D])
    if mnem == "DCR_D":   return bytes([0x15])
    if mnem == "JNZ":     return bytes([0xC2]) + _u16(args[0], labels)
    if mnem == "JMP":     return bytes([0xC3]) + _u16(args[0], labels)
    if mnem == "CALL":    return bytes([0xCD]) + _u16(args[0], labels)
    if mnem == "RET":     return bytes([0xC9])
    if mnem == "LXI_SP":  return bytes([0x31]) + _u16(args[0], labels)
    if mnem == "HLT":     return bytes([0x76])
    if mnem == "NOP":     return bytes([0x00])
    if mnem == "XRA_A":   return bytes([0xAF])
    raise AssertionError(f"unknown mnemonic: {mnem}")


# Graphics command first byte = 0xFF (bit7=1 + max colour 0x7F).
WHITE = 0xFF

# emit_pixel(): A=X, B=Y → OUT FFh, OUT A (X), OUT B (Y)
# Pre: caller loaded X into A, Y into B.

ASM = [
    (None,           "LXI_SP", 0xF000),
    # B = Y for current row, C = loop counter, D = saved X for sides

    # ---- top edge (y=0): for X=0..7, emit (FF, X, 0) ----
    (None,           "MVI_E",  0x00),       # E = Y
    (None,           "MVI_D",  0x00),       # D = X starts at 0
    (None,           "MVI_C",  0x08),       # C = 8 pixels
    ("top_loop",     "CALL",   "emit_pixel"),
    (None,           "INR_D",  ),
    (None,           "DCR_C",  ),
    (None,           "JNZ",    "top_loop"),

    # ---- bottom edge (y=7): for X=0..7, emit (FF, X, 7) ----
    (None,           "MVI_E",  0x07),
    (None,           "MVI_D",  0x00),
    (None,           "MVI_C",  0x08),
    ("bot_loop",     "CALL",   "emit_pixel"),
    (None,           "INR_D",  ),
    (None,           "DCR_C",  ),
    (None,           "JNZ",    "bot_loop"),

    # ---- left side (x=0): for Y=1..6, emit (FF, 0, Y) ----
    (None,           "MVI_D",  0x00),
    (None,           "MVI_E",  0x01),
    (None,           "MVI_C",  0x06),
    ("left_loop",    "CALL",   "emit_pixel"),
    (None,           "INR_E",  ),
    (None,           "DCR_C",  ),
    (None,           "JNZ",    "left_loop"),

    # ---- right side (x=7): for Y=1..6, emit (FF, 7, Y) ----
    (None,           "MVI_D",  0x07),
    (None,           "MVI_E",  0x01),
    (None,           "MVI_C",  0x06),
    ("right_loop",   "CALL",   "emit_pixel"),
    (None,           "INR_E",  ),
    (None,           "DCR_C",  ),
    (None,           "JNZ",    "right_loop"),

    (None,           "HLT",    ),

    # ---- emit_pixel: OUT FFh, OUT D(X), OUT E(Y) ----
    ("emit_pixel",   "MVI_A",  WHITE),
    (None,           "OUT",    0x00),
    (None,           "MOV_A_D",),
    (None,           "OUT",    0x00),
    (None,           "MOV_A_E",),
    (None,           "OUT",    0x00),
    (None,           "RET",    ),
]


def assemble(asm, base=0x0000):
    labels = {}
    addr = base
    for entry in asm:
        label, mnem, *_args = entry
        if label is not None:
            assert label not in labels, f"duplicate label: {label}"
            labels[label] = addr
        addr += SIZES[mnem]
    out = bytearray()
    addr = base
    for entry in asm:
        _, mnem, *args = entry
        chunk = encode(mnem, args, labels)
        assert len(chunk) == SIZES[mnem]
        out += chunk
        addr += len(chunk)
    return bytes(out), labels


def write_tlv(buf, tag, value):
    buf.append(tag)
    buf += struct.pack("<I", len(value))
    buf += value


def main():
    program, labels = assemble(ASM)
    print(f"program: {len(program)} bytes")
    for name, addr in sorted(labels.items(), key=lambda kv: kv[1]):
        print(f"  {addr:04X}  {name}")

    ram = bytearray(0x10000)
    ram[: len(program)] = program

    payload = bytearray()
    write_tlv(payload, 0x01, bytes(ram))                   # RAM
    write_tlv(payload, 0x02, b"\x00" * 7)                   # registers A B C D E H L
    write_tlv(payload, 0x03, bytes([0b00000010]))           # PSW: bit1=1 (8080 reserved)
    write_tlv(payload, 0x04, struct.pack("<H", 0))          # PC
    write_tlv(payload, 0x05, struct.pack("<H", 0))          # SP (program sets it explicitly)
    write_tlv(payload, 0x06, bytes([0, 0, 0, 0, 0]))        # interrupt state
    write_tlv(payload, 0x07, bytes([0]))                    # halt = false
    write_tlv(payload, 0x08, struct.pack("<Q", 0))          # cycle_count

    out = bytearray()
    out += b"K580"
    out += struct.pack("<H", 1)
    out += struct.pack("<I", len(payload))
    out += payload

    target = Path(__file__).resolve().parent.parent / "square.580"
    target.write_bytes(bytes(out))
    print(f"wrote {target} ({len(out)} bytes)")


if __name__ == "__main__":
    main()
