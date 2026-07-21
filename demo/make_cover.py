# 데모 게임 표지(big.icon) 생성 — 크림 배경 + 픽셀 하트 (PIL 불필요)
import struct, zlib, sys

GRID, SCALE = 16, 8  # 128x128
BG = (0xFF, 0xE3, 0xC2)
HEART = (0xFF, 0x4D, 0x6D)

px = [[BG for _ in range(GRID)] for _ in range(GRID)]

heart_rows = {
    3: [4, 5, 9, 10],
    4: [3, 4, 5, 6, 8, 9, 10, 11],
    5: [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    6: [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    7: [3, 4, 5, 6, 7, 8, 9, 10, 11],
    8: [4, 5, 6, 7, 8, 9, 10],
    9: [5, 6, 7, 8, 9],
    10: [6, 7, 8],
    11: [7],
}
for y, xs in heart_rows.items():
    for x in xs:
        px[y][x] = HEART

W = H = GRID * SCALE
raw = bytearray()
for y in range(H):
    raw.append(0)
    for x in range(W):
        r, g, b = px[y // SCALE][x // SCALE]
        raw += bytes((r, g, b))

def chunk(tag, data):
    c = struct.pack(">I", len(data)) + tag + data
    return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

out = b"\x89PNG\r\n\x1a\n"
out += chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 2, 0, 0, 0))
out += chunk(b"IDAT", zlib.compress(bytes(raw), 9))
out += chunk(b"IEND", b"")
open(sys.argv[1], "wb").write(out)
print(f"OK {W}x{H}")
