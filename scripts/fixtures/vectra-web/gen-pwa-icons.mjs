#!/usr/bin/env node
/**
 * One-shot: escribe PNG RGB en vectra-web/icons/ (sin dependencias npm).
 * Uso: node scripts/fixtures/vectra-web/gen-pwa-icons.mjs
 */
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUT_DIR = path.join(__dirname, "../../../vectra-web/icons");

const BG = { r: 0x0a, g: 0x0a, b: 0x0a };
const FG = { r: 0xf5, g: 0xf5, b: 0xf7 };

function crc32(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i += 1) {
    c ^= buf[i];
    for (let k = 0; k < 8; k += 1) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
  }
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
  return Buffer.concat([len, typeBuf, data, crc]);
}

function createPng(width, height, maskable) {
  const rowBytes = 1 + width * 3;
  const raw = Buffer.alloc(rowBytes * height);
  const margin = maskable ? Math.floor(width * 0.1) : Math.floor(width * 0.25);
  const innerW = width - 2 * margin;
  const innerH = height - 2 * margin;
  const barW = Math.max(2, Math.floor(innerW * 0.12));
  const topY = margin + Math.floor(innerH * 0.15);
  const botY = margin + Math.floor(innerH * 0.85);
  const midX = Math.floor(width / 2);

  for (let y = 0; y < height; y += 1) {
    const rowOff = y * rowBytes;
    raw[rowOff] = 0;
    for (let x = 0; x < width; x += 1) {
      const px = rowOff + 1 + x * 3;
      let color = BG;
      if (maskable) {
        const inBar =
          (x >= midX - barW && x <= midX + barW && y >= topY && y <= botY) ||
          (y >= topY && y <= topY + barW && x >= midX - barW && x <= midX);
        if (inBar) {
          color = FG;
        }
      } else if (
        x >= margin &&
        x < width - margin &&
        y >= margin &&
        y < height - margin
      ) {
        const t = (x - margin) / Math.max(1, innerW - 1);
        color = {
          r: Math.round(BG.r + (FG.r - BG.r) * t * 0.35),
          g: Math.round(BG.g + (FG.g - BG.g) * t * 0.35),
          b: Math.round(BG.b + (FG.b - BG.b) * t * 0.35),
        };
      }
      raw[px] = color.r;
      raw[px + 1] = color.g;
      raw[px + 2] = color.b;
    }
  }

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;

  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const compressed = zlib.deflateSync(raw, { level: 9 });
  return Buffer.concat([
    signature,
    chunk("IHDR", ihdr),
    chunk("IDAT", compressed),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

fs.mkdirSync(OUT_DIR, { recursive: true });
const files = [
  ["icon-192.png", 192, false],
  ["icon-512.png", 512, false],
  ["icon-192-maskable.png", 192, true],
  ["icon-512-maskable.png", 512, true],
];
for (const [name, size, maskable] of files) {
  fs.writeFileSync(path.join(OUT_DIR, name), createPng(size, size, maskable));
}
