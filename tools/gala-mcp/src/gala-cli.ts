// Thin wrapper that shells out to the `gala` binary.

import { spawn } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const PACKAGE_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..');

/** Path to the gala binary: $GALA_BIN, else <repo>/target/debug/gala. */
export function galaBinPath(): string {
  const fromEnv = process.env.GALA_BIN;
  if (fromEnv && fromEnv.length > 0) return fromEnv;
  return resolve(PACKAGE_DIR, '../../target/debug/gala');
}

export interface GalaProcess {
  exitCode: number | null;
  stdout: string;
  stderr: string;
  timedOut: boolean;
  spawnError?: string;
}

export function runGala(args: string[], timeoutMs = 60_000): Promise<GalaProcess> {
  return new Promise((done) => {
    let stdout = '';
    let stderr = '';
    let timedOut = false;
    let settled = false;
    const child = spawn(galaBinPath(), args, { stdio: ['ignore', 'pipe', 'pipe'] });
    const timer = setTimeout(() => {
      timedOut = true;
      child.kill('SIGKILL');
    }, timeoutMs);
    child.stdout.setEncoding('utf8');
    child.stderr.setEncoding('utf8');
    child.stdout.on('data', (chunk: string) => {
      stdout += chunk;
    });
    child.stderr.on('data', (chunk: string) => {
      stderr += chunk;
    });
    child.on('error', (err) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      done({ exitCode: null, stdout, stderr, timedOut, spawnError: err.message });
    });
    child.on('close', (code) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      done({ exitCode: code, stdout, stderr, timedOut });
    });
  });
}

/** Writes `source` to a fresh temp directory as `fileName`, runs fn, then removes the directory. */
export async function withSourceFile<T>(
  source: string,
  fileName: string,
  fn: (path: string) => Promise<T>
): Promise<T> {
  const dir = await mkdtemp(join(tmpdir(), 'gala-mcp-'));
  try {
    const safe = basename(fileName).replace(/[^A-Za-z0-9_.-]/g, '_') || 'input.gala';
    const name = safe.endsWith('.gala') ? safe : `${safe}.gala`;
    const path = join(dir, name);
    await writeFile(path, source, 'utf8');
    return await fn(path);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}
