import { spawn, execFileSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(e2eDir, "..");
const stackPath = path.join(e2eDir, ".stack.json");

function fail(message) {
  console.error(message);
  process.exit(1);
}

function getFreePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.listen(0, "127.0.0.1", () => {
      const addr = server.address();
      const port = typeof addr === "object" && addr ? addr.port : 0;
      server.close((err) => (err ? reject(err) : resolve(port)));
    });
    server.on("error", reject);
  });
}

function resolveVortexBin() {
  if (process.env.VORTEX_BIN) {
    return path.resolve(process.env.VORTEX_BIN);
  }
  const debug = path.join(repoRoot, "vortex-core", "target", "debug", "vortex");
  if (fs.existsSync(debug)) {
    return debug;
  }
  fail(
    "e2e harness: falta VORTEX_BIN o vortex-core/target/debug/vortex (cargo build --bin vortex)",
  );
}

function resolveMeiliBin() {
  if (process.env.MEILI_BIN) {
    return path.resolve(process.env.MEILI_BIN);
  }
  const local = path.join(repoRoot, "meilisearch");
  if (fs.existsSync(local)) {
    return local;
  }
  fail(
    "e2e harness: falta MEILI_BIN o ./meilisearch en la raíz del repo",
  );
}

function writeConfig(tempDir, bind, meiliUrl, webPath) {
  const datos = path.join(tempDir, "datos");
  fs.mkdirSync(datos, { recursive: true });
  const configPath = path.join(tempDir, "vortex.toml");
  const body = `
[paths]
datos = "${datos.replace(/\\/g, "/")}"
modelos = "/tmp/vortex-e2e-modelos"
backups = "/tmp/vortex-e2e-backups"
web = "${webPath.replace(/\\/g, "/")}"

[server]
bind = "${bind}"
hostname = "localhost"

[negocio]
moneda = "ARS"

[meilisearch]
url = "${meiliUrl}"

[ollama]
url = "http://127.0.0.1:11434"

[backup]
hora = "03:00"
retencion = 14

[logs]
nivel = "info"
retencion = 14
`.trim();
  fs.writeFileSync(configPath, body);
  return configPath;
}

async function waitHttpOk(url, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(url, { signal: AbortSignal.timeout(2000) });
      if (res.ok) {
        return;
      }
    } catch {
      /* retry */
    }
    await new Promise((r) => setTimeout(r, 400));
  }
  fail(`e2e harness: timeout esperando ${url}`);
}

function killPid(pid) {
  if (!pid || pid <= 0) {
    return;
  }
  try {
    process.kill(pid, "SIGTERM");
  } catch {
    /* already dead */
  }
}

export default async function globalSetup() {
  const vortexBin = resolveVortexBin();
  const meiliBin = resolveMeiliBin();
  const seedDir = path.join(repoRoot, "seed");
  if (!fs.existsSync(seedDir)) {
    fail(`e2e harness: falta directorio seed en ${seedDir}`);
  }
  const webPath = path.join(repoRoot, "vectra-web");
  if (!fs.existsSync(webPath)) {
    fail(`e2e harness: falta vectra-web en ${webPath}`);
  }

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vortex-e2e-"));
  const vortexPort = await getFreePort();
  const meiliPort = await getFreePort();
  const bind = `127.0.0.1:${vortexPort}`;
  const baseURL = `http://${bind}`;
  const meiliUrl = `http://127.0.0.1:${meiliPort}`;
  const configPath = writeConfig(tempDir, bind, meiliUrl, webPath);

  const meiliDb = path.join(tempDir, "meili-data");
  fs.mkdirSync(meiliDb, { recursive: true });
  const meili = spawn(
    meiliBin,
    ["--http-addr", `127.0.0.1:${meiliPort}`, "--db-path", meiliDb],
    {
      env: { ...process.env, MEILI_NO_ANALYTICS: "true" },
      stdio: "ignore",
      detached: false,
      cwd: tempDir,
    },
  );
  meili.on("error", (err) => fail(`meilisearch: ${err.message}`));

  await waitHttpOk(`${meiliUrl}/health`, 60_000);

  try {
    execFileSync(
      vortexBin,
      ["--config", configPath, "seed", seedDir],
      { stdio: "inherit", cwd: repoRoot },
    );
  } catch (err) {
    killPid(meili.pid);
    const detail = err instanceof Error ? err.message : String(err);
    fail(`e2e harness: vortex seed falló (${detail})`);
  }

  const serve = spawn(vortexBin, ["--config", configPath, "serve"], {
    stdio: "ignore",
    detached: false,
    cwd: repoRoot,
  });
  serve.on("error", (err) => {
    killPid(meili.pid);
    fail(`vortex serve: ${err.message}`);
  });

  await waitHttpOk(`${baseURL}/health`, 120_000);

  const stack = {
    baseURL,
    tempDir,
    configPath,
    vortexPid: serve.pid,
    meiliPid: meili.pid,
  };
  fs.writeFileSync(stackPath, JSON.stringify(stack, null, 2));

  return async () => {
    killPid(stack.vortexPid);
    killPid(stack.meiliPid);
    try {
      fs.rmSync(stack.tempDir, { recursive: true, force: true });
    } catch {
      /* best effort */
    }
    try {
      fs.unlinkSync(stackPath);
    } catch {
      /* best effort */
    }
  };
}