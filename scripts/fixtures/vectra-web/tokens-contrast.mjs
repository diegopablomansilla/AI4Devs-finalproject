/**
 * Oráculo WCAG 2.1 relativo a tokens.css (sin dependencias npm).
 */

const REQUIRED_TOKENS = [
  "--vx-color-bg",
  "--vx-color-surface",
  "--vx-color-surface-solid",
  "--vx-color-text",
  "--vx-color-text-muted",
  "--vx-color-accent",
  "--vx-color-error",
  "--vx-color-success",
  "--vx-color-focus-ring",
  "--vx-color-on-accent",
  "--vx-color-on-error",
  "--vx-color-on-success",
  "--vx-space-1",
  "--vx-space-2",
  "--vx-space-3",
  "--vx-space-4",
  "--vx-radius-sm",
  "--vx-radius-md",
  "--vx-touch-min",
  "--vx-surface-blur",
];

const WCAG_AA_NORMAL = 4.5;

/**
 * @param {string} css
 * @returns {Record<string, string>}
 */
export function parseRootVars(css) {
  const vars = {};
  const rootMatch = css.match(/:root\s*\{([^}]*)\}/s);
  if (!rootMatch) {
    return vars;
  }
  const block = rootMatch[1];
  const re = /(--vx-[a-z0-9-]+)\s*:\s*([^;]+);/gi;
  let m;
  while ((m = re.exec(block)) !== null) {
    vars[m[1]] = m[2].trim();
  }
  return vars;
}

/**
 * @param {string} raw
 * @returns {{ r: number, g: number, b: number, a: number } | null}
 */
export function parseColor(raw) {
  const s = raw.trim().toLowerCase();
  const hex = s.match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/);
  if (hex) {
    let h = hex[1];
    if (h.length === 3) {
      h = h
        .split("")
        .map((c) => c + c)
        .join("");
    }
    return {
      r: parseInt(h.slice(0, 2), 16),
      g: parseInt(h.slice(2, 4), 16),
      b: parseInt(h.slice(4, 6), 16),
      a: 1,
    };
  }
  const rgba = s.match(
    /^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)(?:\s*,\s*([\d.]+))?\s*\)$/,
  );
  if (rgba) {
    return {
      r: Number(rgba[1]),
      g: Number(rgba[2]),
      b: Number(rgba[3]),
      a: rgba[4] !== undefined ? Number(rgba[4]) : 1,
    };
  }
  return null;
}

/**
 * @param {{ r: number, g: number, b: number, a?: number }} c
 */
function srgbChannel(c) {
  const n = c / 255;
  return n <= 0.03928 ? n / 12.92 : ((n + 0.055) / 1.055) ** 2.4;
}

/**
 * @param {{ r: number, g: number, b: number }} c
 */
export function relativeLuminance(c) {
  const r = srgbChannel(c.r);
  const g = srgbChannel(c.g);
  const b = srgbChannel(c.b);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/**
 * @param {{ r: number, g: number, b: number, a: number }} fg
 * @param {{ r: number, g: number, b: number, a: number }} bg
 */
export function compositeOver(fg, bg) {
  const a = fg.a + bg.a * (1 - fg.a);
  if (a === 0) {
    return { r: 0, g: 0, b: 0, a: 0 };
  }
  const r = (fg.r * fg.a + bg.r * bg.a * (1 - fg.a)) / a;
  const g = (fg.g * fg.a + bg.g * bg.a * (1 - fg.a)) / a;
  const b = (fg.b * fg.a + bg.b * bg.a * (1 - fg.a)) / a;
  return { r, g, b, a: 1 };
}

/**
 * @param {string} fgRaw
 * @param {string} bgRaw
 */
export function contrastRatio(fgRaw, bgRaw) {
  const fg = parseColor(fgRaw);
  const bg = parseColor(bgRaw);
  if (!fg || !bg) {
    return 0;
  }
  let fgRgb = fg;
  let bgRgb = bg;
  if (fg.a < 1) {
    fgRgb = compositeOver(fg, bg);
  }
  const l1 = relativeLuminance(fgRgb);
  const l2 = relativeLuminance(bgRgb);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * @param {Record<string, string>} vars
 * @returns {{ name: string, fg: string, bg: string }[]}
 */
export function contrastPairsFromVars(vars) {
  const bg = vars["--vx-color-bg"];
  const surface = vars["--vx-color-surface"];
  const solid = vars["--vx-color-surface-solid"];
  const composed =
    surface && bg
      ? (() => {
          const fg = parseColor(surface);
          const b = parseColor(bg);
          if (!fg || !b) {
            return null;
          }
          const c = compositeOver(fg, b);
          return `rgb(${Math.round(c.r)}, ${Math.round(c.g)}, ${Math.round(c.b)})`;
        })()
      : null;

  return [
    { name: "text on bg", fg: vars["--vx-color-text"], bg },
    {
      name: "text on glass surface",
      fg: vars["--vx-color-text"],
      bg: composed,
    },
    { name: "text on surface-solid", fg: vars["--vx-color-text"], bg: solid },
    { name: "text-muted on bg", fg: vars["--vx-color-text-muted"], bg },
    {
      name: "text-muted on glass surface",
      fg: vars["--vx-color-text-muted"],
      bg: composed,
    },
    {
      name: "text-muted on surface-solid",
      fg: vars["--vx-color-text-muted"],
      bg: solid,
    },
    {
      name: "on-accent on accent",
      fg: vars["--vx-color-on-accent"],
      bg: vars["--vx-color-accent"],
    },
    {
      name: "on-error on error",
      fg: vars["--vx-color-on-error"],
      bg: vars["--vx-color-error"],
    },
    {
      name: "on-success on success",
      fg: vars["--vx-color-on-success"],
      bg: vars["--vx-color-success"],
    },
  ];
}

/**
 * @param {Record<string, string>} vars
 * @returns {{ ok: boolean, errors: string[] }}
 */
export function validateVxContract(vars) {
  const errors = [];
  for (const name of REQUIRED_TOKENS) {
    if (!vars[name]) {
      errors.push(`falta ${name}`);
    }
  }
  if (vars["--vx-color-bg"] !== "#0A0A0A" && vars["--vx-color-bg"] !== "#0a0a0a") {
    errors.push(`--vx-color-bg debe ser #0A0A0A (obtenido: ${vars["--vx-color-bg"]})`);
  }
  if (vars["--vx-touch-min"] !== "44px") {
    errors.push(`--vx-touch-min debe ser 44px (obtenido: ${vars["--vx-touch-min"]})`);
  }
  const surf = parseColor(vars["--vx-color-surface"] ?? "");
  if (!surf || surf.a >= 1) {
    errors.push("--vx-color-surface debe ser translúcida (alfa < 1)");
  }
  const blur = vars["--vx-surface-blur"];
  if (!blur || blur === "0" || blur === "0px") {
    errors.push("--vx-surface-blur no debe ser 0");
  }
  return { ok: errors.length === 0, errors };
}

/**
 * @param {Record<string, string>} vars
 * @returns {{ ok: boolean, failures: { name: string, ratio: number }[] }}
 */
export function validateContrastPairs(vars) {
  const pairs = contrastPairsFromVars(vars);
  const failures = [];
  for (const p of pairs) {
    if (!p.fg || !p.bg) {
      failures.push({ name: p.name, ratio: 0 });
      continue;
    }
    const ratio = contrastRatio(p.fg, p.bg);
    if (ratio < WCAG_AA_NORMAL) {
      failures.push({ name: p.name, ratio });
    }
  }
  return { ok: failures.length === 0, failures };
}

export { REQUIRED_TOKENS, WCAG_AA_NORMAL };
