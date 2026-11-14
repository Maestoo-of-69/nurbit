"""Draws the Nurbit source icon (1024x1024 PNG) with the standard library only.

Design: navy rounded square, white shield, blue medical cross inside.
Run:  python make_icon_source.py  -> icon-source.png
Then: npx tauri icon src-tauri/icons/icon-source.png  (from apps/desktop) generates all sizes.
"""

import struct
import zlib
from pathlib import Path

SIZE = 1024
SAMPLES = 3  # supersampling per axis for smooth (anti-aliased) edges
NAVY = (18, 38, 58)
WHITE = (255, 255, 255)
ACCENT = (31, 111, 235)


def in_rounded_square(x: float, y: float) -> bool:
    margin, radius = 40, 200
    lo, hi = margin, SIZE - margin
    cx = min(max(x, lo + radius), hi - radius)
    cy = min(max(y, lo + radius), hi - radius)
    return lo <= x <= hi and lo <= y <= hi and (x - cx) ** 2 + (y - cy) ** 2 <= radius**2


def in_shield(x: float, y: float) -> bool:
    # Flat top with straight sides, then a curved point at the bottom.
    top, side_end, bottom = 220, 560, 860
    half_width = 260
    dx = abs(x - SIZE / 2)
    if y < top or y > bottom:
        return False
    if y <= side_end:
        return dx <= half_width
    t = (y - side_end) / (bottom - side_end)  # 0 at side end, 1 at the tip
    return dx <= half_width * (1 - t**1.6)


def in_cross(x: float, y: float) -> bool:
    cx, cy, arm, thick = SIZE / 2, 500, 150, 55
    return (abs(x - cx) <= thick and abs(y - cy) <= arm) or (
        abs(y - cy) <= thick and abs(x - cx) <= arm
    )


def pixel(px: int, py: int) -> tuple[int, int, int, int]:
    totals = [0.0, 0.0, 0.0, 0.0]
    for sx in range(SAMPLES):
        for sy in range(SAMPLES):
            x = px + (sx + 0.5) / SAMPLES
            y = py + (sy + 0.5) / SAMPLES
            if not in_rounded_square(x, y):
                continue
            color = NAVY
            if in_shield(x, y):
                color = ACCENT if in_cross(x, y) else WHITE
            for i in range(3):
                totals[i] += color[i]
            totals[3] += 255
    n = SAMPLES * SAMPLES
    alpha = totals[3] / n
    if alpha == 0:
        return (0, 0, 0, 0)
    covered = totals[3] / 255
    return tuple(round(totals[i] / covered) for i in range(3)) + (round(alpha),)


def write_png(path: Path) -> None:
    rows = bytearray()
    for py in range(SIZE):
        rows.append(0)  # PNG filter type "none" for this row
        for px in range(SIZE):
            rows.extend(pixel(px, py))

    def chunk(kind: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + kind + data + struct.pack(
            ">I", zlib.crc32(kind + data) & 0xFFFFFFFF
        )

    header = struct.pack(">IIBBBBB", SIZE, SIZE, 8, 6, 0, 0, 0)  # 8-bit RGBA
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
    png += chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b"")
    path.write_bytes(png)


if __name__ == "__main__":
    out = Path(__file__).with_name("icon-source.png")
    write_png(out)
    print(f"wrote {out}")
