import { test as base } from "@playwright/test";
import { spawn, type ChildProcess } from "child_process";
import { createServer } from "net";
import { mkdtempSync, rmSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";

const ROOT = join(__dirname, "../..");

async function findFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const srv = createServer();
    srv.listen(0, () => {
      const addr = srv.address();
      if (addr && typeof addr === "object") {
        const port = addr.port;
        srv.close(() => resolve(port));
      } else {
        reject(new Error("Failed to get port"));
      }
    });
  });
}

type FlowlFixtures = {
  flowl: { baseURL: string };
};

export const test = base.extend<FlowlFixtures>({
  flowl: async ({}, use) => {
    const port = await findFreePort();
    const tempDir = mkdtempSync(join(tmpdir(), "flowl-e2e-"));
    const dbPath = join(tempDir, "flowl.db");

    const proc = spawn(join(ROOT, "target/debug/flowl"), [], {
      env: {
        ...process.env,
        FLOWL_PORT: String(port),
        FLOWL_DB_PATH: dbPath,
        FLOWL_MQTT_DISABLED: "true",
        FLOWL_LOG_LEVEL: "warn",
      },
      stdio: "pipe",
    });

    await waitForReady(proc, port);

    await use({ baseURL: `http://localhost:${port}` });

    proc.kill("SIGTERM");
    await new Promise<void>((resolve) => proc.on("close", resolve));
    rmSync(tempDir, { recursive: true, force: true });
  },
});

async function waitForReady(
  proc: ChildProcess,
  port: number,
  timeoutMs = 15_000,
): Promise<void> {
  const start = Date.now();
  const url = `http://localhost:${port}/health`;

  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch {
      // not ready yet
    }
    await new Promise((r) => setTimeout(r, 200));
  }

  proc.kill("SIGTERM");
  throw new Error(`flowl did not become ready within ${timeoutMs}ms`);
}

export { expect } from "@playwright/test";
