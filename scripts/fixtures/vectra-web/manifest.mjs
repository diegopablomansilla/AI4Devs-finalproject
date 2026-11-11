import fs from "node:fs";
import path from "node:path";

export const MANIFEST_ICON_FILES = [
  "icon-192.png",
  "icon-512.png",
  "icon-192-maskable.png",
  "icon-512-maskable.png",
];

export function pngIhdrDimensions(buffer) {
  if (!Buffer.isBuffer(buffer) || buffer.length < 24) {
    return null;
  }
  const sig = buffer.subarray(0, 8);
  const pngSig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  if (!sig.equals(pngSig)) {
    return null;
  }
  const width = buffer.readUInt32BE(16);
  const height = buffer.readUInt32BE(20);
  return { width, height };
}

export function iconPathFromManifestSrc(src) {
  const m = /^\/static\/\{\{VERSION\}\}\/icons\/(.+)$/.exec(src);
  if (!m) {
    return null;
  }
  return m[1];
}

export function validateManifestObject(manifest) {
  const errors = [];
  if (manifest == null || typeof manifest !== "object" || Array.isArray(manifest)) {
    return { ok: false, errors: ["manifest no es un objeto JSON"] };
  }
  if (manifest.name !== "Vectra") {
    errors.push('name debe ser "Vectra"');
  }
  if (manifest.short_name !== "Vectra") {
    errors.push('short_name debe ser "Vectra"');
  }
  if (manifest.display !== "standalone") {
    errors.push('display debe ser "standalone"');
  }
  if (manifest.background_color !== "#0A0A0A") {
    errors.push('background_color debe ser "#0A0A0A"');
  }
  if (manifest.theme_color !== "#0A0A0A") {
    errors.push('theme_color debe ser "#0A0A0A"');
  }
  if (manifest.start_url !== "/") {
    errors.push('start_url debe ser "/"');
  }
  if (manifest.lang !== "es") {
    errors.push('lang debe ser "es"');
  }
  if (!Array.isArray(manifest.icons)) {
    errors.push("icons debe ser un array");
    return { ok: false, errors };
  }

  const hasSizePurpose = (w, h, purposePart) =>
    manifest.icons.some((icon) => {
      if (!icon || typeof icon !== "object") {
        return false;
      }
      const sizes = String(icon.sizes ?? "");
      const purpose = String(icon.purpose ?? "");
      return sizes === `${w}x${h}` && purpose.includes(purposePart);
    });

  if (!hasSizePurpose(192, 192, "any")) {
    errors.push("falta ícono 192x192 con purpose any");
  }
  if (!hasSizePurpose(512, 512, "any")) {
    errors.push("falta ícono 512x512 con purpose any");
  }
  if (!hasSizePurpose(192, 192, "maskable")) {
    errors.push("falta ícono 192x192 con purpose maskable");
  }
  if (!hasSizePurpose(512, 512, "maskable")) {
    errors.push("falta ícono 512x512 con purpose maskable");
  }

  for (const icon of manifest.icons) {
    const src = String(icon?.src ?? "");
    if (!src.startsWith("/static/{{VERSION}}/icons/") || !src.endsWith(".png")) {
      errors.push(`src de ícono inválido: ${src}`);
    }
    if (icon?.type !== "image/png") {
      errors.push(`type debe ser image/png para ${src}`);
    }
  }

  return { ok: errors.length === 0, errors };
}

export function validateManifestIconsOnDisk(manifest, webRoot) {
  const errors = [];
  for (const icon of manifest.icons ?? []) {
    const rel = iconPathFromManifestSrc(String(icon.src ?? ""));
    if (!rel) {
      errors.push(`src no mapea a disco: ${icon.src}`);
      continue;
    }
    const file = path.join(webRoot, "icons", rel);
    if (!fs.existsSync(file)) {
      errors.push(`falta archivo de ícono: ${file}`);
      continue;
    }
    const dims = pngIhdrDimensions(fs.readFileSync(file));
    if (!dims) {
      errors.push(`PNG inválido: ${rel}`);
      continue;
    }
    const [w, h] = String(icon.sizes).split("x").map((n) => Number(n));
    if (dims.width !== w || dims.height !== h) {
      errors.push(
        `${rel}: IHDR ${dims.width}x${dims.height} no coincide con sizes ${icon.sizes}`,
      );
    }
  }
  return { ok: errors.length === 0, errors };
}
