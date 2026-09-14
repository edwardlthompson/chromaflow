#!/usr/bin/env python3
"""Fusion analog + D_LED HID fill. MIT. No OpenRGB C++. No init on ticks."""

from __future__ import annotations

import sys

import hid

VID, PID = 0x048D, 0x5702
CH = (
    (0x20, 0x01),
    (0x21, 0x02),
    (0x22, 0x04),
    (0x23, 0x08),
    (0x24, 0x10),
    (0x25, 0x20),
    (0x26, 0x40),
    (0x27, 0x80),
)


def feat(dev: hid.device, data: list[int]) -> None:
    buf = (data + [0] * 64)[:64]
    if dev.send_feature_report(bytes(buf)) < 1:
        raise RuntimeError("Fusion HID feature failed")


def open_fusion() -> hid.device:
    rows = hid.enumerate(VID, PID)
    if not rows:
        raise RuntimeError("Fusion USB 048d:5702 missing")
    dev = hid.device()
    dev.open_path(rows[0]["path"])
    return dev


def analog(dev: hid.device, rgb: bytes) -> None:
    r, g, b = rgb
    for a1, a2 in CH:
        feat(
            dev,
            [0xCC, a1, a2, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 90, 0, b, g, r, 0, 0, 0, 0, 0]
            + [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        )


def digital(dev: hid.device, rgb: bytes) -> None:
    r, g, b = rgb
    for addr, mask in ((0x25, 0x20), (0x26, 0x40)):
        feat(dev, [0xCC, addr, 0xFF, 0x07, 0, 0, 0, 0, 0, 0, 0, 0x01, 0xFF, 0, b, g, r])
        pkt = [0xCC, addr, mask, 32, 0, 0, 0, 0, 0, 0, 0, 0x01, 90, 0]
        while len(pkt) < 64:
            pkt.extend([b, g, r])
        feat(dev, pkt[:64])


def apply(dev: hid.device) -> None:
    feat(dev, [0xCC, 0x28, 0xFF, 0x07])


def paint(dev: hid.device, kind: str, rgb: bytes) -> None:
    if kind == "uniform":
        digital(dev, rgb)
        analog(dev, rgb)
    elif kind == "soft":
        analog(dev, rgb)
    else:
        digital(dev, rgb)
    apply(dev)


def serve() -> int:
    dev = open_fusion()
    try:
        for raw in sys.stdin:
            parts = raw.strip().split()
            if not parts or parts[0] == "quit":
                break
            if len(parts) != 2 or parts[0] not in ("digital", "uniform", "soft"):
                print("err usage", flush=True)
                continue
            kind, hex6 = parts[0], parts[1].lstrip("#")
            if len(hex6) != 6:
                print("err color", flush=True)
                continue
            paint(dev, kind, bytes.fromhex(hex6))
            print("ok", flush=True)
    finally:
        dev.close()
    return 0


def main() -> int:
    if len(sys.argv) >= 2 and sys.argv[1] == "serve":
        return serve()
    if len(sys.argv) < 3 or sys.argv[1] not in ("digital", "uniform", "soft"):
        print("usage: fusion-hid.py digital|uniform|soft RRGGBB | serve", file=sys.stderr)
        return 2
    kind, hex6 = sys.argv[1], sys.argv[2].strip().lstrip("#")
    if len(hex6) != 6:
        raise RuntimeError("color must be RRGGBB")
    rgb = bytes.fromhex(hex6)
    dev = open_fusion()
    try:
        paint(dev, kind, rgb)
    finally:
        dev.close()
    print(f"set Fusion {kind} {hex6}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
