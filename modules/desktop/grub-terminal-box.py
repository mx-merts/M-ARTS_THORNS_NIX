"""GRUB terminal kutusu: yuvarlak koseli, yari saydam, ince cizgili panel.

GRUB "terminal_box_*.png" adli 9 parcadan (nw n ne w c e sw s se) bir
kutu kurar: koseler oldugu gibi, kenarlar ve orta doseme olarak cizilir.
Elegant bu dosyalari istiyor ama repoda yok; E (duzenle) ve C (konsol)
ekranlarindaki yazilar arka planin ustune basiliyor, okunmuyordu.

Kullanim: grub-terminal-box.py <cikis> <dolgu RRGGBBAA> <cizgi RRGGBBAA> <yaricap> <cizgi-kalinligi>
Sadece Python standart kutuphanesi (zlib, struct): ek paket yok.
"""
import os
import struct
import sys
import zlib


def png(path, w, h, pixels):
    """pixels: satir satir (r, g, b, a) listesi."""
    raw = b"".join(
        b"\x00" + bytes(c for px in pixels[y * w:(y + 1) * w] for c in px)
        for y in range(h)
    )

    def chunk(kind, data):
        return (struct.pack(">I", len(data)) + kind + data
                + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF))

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)))
        f.write(chunk(b"IDAT", zlib.compress(raw, 9)))
        f.write(chunk(b"IEND", b""))


def main():
    out = sys.argv[1]
    fill = [int(sys.argv[2][i:i + 2], 16) for i in range(0, 8, 2)]
    stroke = [int(sys.argv[3][i:i + 2], 16) for i in range(0, 8, 2)]
    radius, width = int(sys.argv[4]), float(sys.argv[5])
    size = 2 * radius + 1  # tam kutu: 2 kose + 1 piksel orta

    def coverage(x, y):
        # 4x4 ornek: kutunun icinde mi, cizginin (kenardan "width" px) icinde mi
        outer = inner = 0
        for sx in range(4):
            for sy in range(4):
                px, py = x + (sx + 0.5) / 4, y + (sy + 0.5) / 4
                dx = max(radius - px, 0, px - (radius + 1))
                dy = max(radius - py, 0, py - (radius + 1))
                d = (dx * dx + dy * dy) ** 0.5
                if d <= radius:
                    outer += 1
                    if d <= radius - width:
                        inner += 1
        return outer / 16, inner / 16

    def pixel(x, y):
        # Cizgi, dolgunun ustune ("over") bindirilir
        o, i = coverage(x, y)
        fa = fill[3] / 255 * i
        sa = stroke[3] / 255 * (o - i)
        a = sa + fa * (1 - sa)
        if a == 0:
            return (0, 0, 0, 0)
        rgb = [round((stroke[c] * sa + fill[c] * fa * (1 - sa)) / a) for c in range(3)]
        return (rgb[0], rgb[1], rgb[2], round(a * 255))

    full = [[pixel(x, y) for x in range(size)] for y in range(size)]

    def crop(x0, y0, w, h):
        return [full[y][x] for y in range(y0, y0 + h) for x in range(x0, x0 + w)]

    k, m = radius, size - radius  # kose boyu, karsi kosenin basladigi yer
    parts = {
        "nw": (0, 0, k, k), "n": (k, 0, 1, k), "ne": (m, 0, k, k),
        "w": (0, k, k, 1), "c": (k, k, 1, 1), "e": (m, k, k, 1),
        "sw": (0, m, k, k), "s": (k, m, 1, k), "se": (m, m, k, k),
    }
    os.makedirs(out, exist_ok=True)
    for name, (x0, y0, w, h) in parts.items():
        png(os.path.join(out, "terminal_box_%s.png" % name), w, h, crop(x0, y0, w, h))


if __name__ == "__main__":
    main()
