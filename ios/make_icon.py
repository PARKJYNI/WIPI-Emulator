# 픽셀 폴더폰 앱 아이콘 생성기 — 32x32 픽셀 그리드를 1024x1024로 확대 (PIL 불필요, 순수 파이썬)
import struct, zlib, sys

GRID = 32
SCALE = 32  # 32*32 = 1024

# 레트로 팔레트 (주황 폰 + 청록 배경)
BG      = (0x12, 0x8C, 0x7F)  # teal 배경
BG_HI   = (0x17, 0xA3, 0x94)  # 배경 하이라이트(대각선 패턴)
BODY    = (0xFF, 0x8A, 0x3D)  # 폰 본체 주황
BODY_SH = (0xD9, 0x66, 0x1F)  # 본체 그림자/테두리
HINGE   = (0xB5, 0x4F, 0x12)  # 힌지
BEZEL   = (0xFF, 0xE3, 0xC2)  # 화면 베젤 크림
SCREEN  = (0xA8, 0xE6, 0xFF)  # 화면 하늘색
SCR_DK  = (0x2A, 0x3A, 0x5C)  # 화면 픽셀(어두운 네이비)
HEART   = (0xFF, 0x4D, 0x6D)  # 하트
KEY     = (0xFF, 0xE3, 0xC2)  # 키 크림
KEY_SH  = (0xE6, 0xB8, 0x8A)  # 키 아래 그림자
ANT     = (0x8C, 0x3D, 0x0A)  # 안테나

px = [[BG for _ in range(GRID)] for _ in range(GRID)]

def rect(x0, y0, x1, y1, c):
    for y in range(y0, y1):
        for x in range(x0, x1):
            px[y][x] = c

# 배경에 은은한 대각 점 패턴
for y in range(GRID):
    for x in range(GRID):
        if (x + y) % 8 == 0:
            px[y][x] = BG_HI

# ── 안테나 (우상단)
rect(22, 1, 24, 5, ANT)
rect(21, 1, 25, 2, ANT)

# ── 상판(화면부) rows 4..15, cols 7..25
rect(7, 4, 25, 15, BODY)
# 테두리 음영
rect(7, 4, 25, 5, BODY_SH); rect(7, 14, 25, 15, BODY_SH)
rect(7, 4, 8, 15, BODY_SH); rect(24, 4, 25, 15, BODY_SH)
# 베젤 + 화면
rect(9, 6, 23, 13, BEZEL)
rect(10, 7, 22, 12, SCREEN)
# 화면 콘텐츠: 픽셀 하트 + 노트 (화면 영역: x 10..21, y 7..11)
heart_rows = {
    7: [12, 13, 15, 16],
    8: [11, 12, 13, 14, 15, 16, 17],
    9: [12, 13, 14, 15, 16],
    10: [13, 14, 15],
    11: [14],
}
for hy, xs in heart_rows.items():
    for hx in xs:
        px[hy][hx] = HEART
# 음표
px[7][20] = SCR_DK; px[8][20] = SCR_DK; px[9][20] = SCR_DK
px[9][19] = SCR_DK; px[7][21] = SCR_DK

# ── 힌지 rows 15..17
rect(8, 15, 24, 17, HINGE)

# ── 하판(키패드부) rows 17..30, cols 6..26 (살짝 넓게)
rect(6, 17, 26, 30, BODY)
rect(6, 17, 26, 18, BODY_SH)
rect(6, 29, 26, 30, BODY_SH)
rect(6, 17, 7, 30, BODY_SH); rect(25, 17, 26, 30, BODY_SH)

# 키패드 3열 x 4행, 키 4px 폭 x 2px 높이, 간격 1px — cols 8..24, rows 19..29
for row in range(4):
    for col in range(3):
        x0 = 8 + col * 5 + 1
        y0 = 19 + row * 3 - 1 + 1
        rect(x0, y0, x0 + 4, y0 + 2, KEY)
        rect(x0, y0 + 1, x0 + 4, y0 + 2, KEY_SH)

# ── PNG 출력 (nearest 확대)
W = H = GRID * SCALE
raw = bytearray()
for y in range(H):
    raw.append(0)  # filter: None
    gy = y // SCALE
    row = px[gy]
    for x in range(W):
        r, g, b = row[x // SCALE]
        raw += bytes((r, g, b))

def chunk(tag, data):
    c = struct.pack(">I", len(data)) + tag + data
    return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

out = b"\x89PNG\r\n\x1a\n"
out += chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 2, 0, 0, 0))
out += chunk(b"IDAT", zlib.compress(bytes(raw), 9))
out += chunk(b"IEND", b"")

with open(sys.argv[1], "wb") as f:
    f.write(out)
print(f"OK {W}x{H} -> {sys.argv[1]}")
