// GalaAgent: a small wrapper around the `gala` CLI for agent code.
// Each call writes the source to a temp file, runs the CLI, and parses its output.

import { execFile } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

/** Default binary: <repo>/target/debug/gala, relative to this file (sdk/typescript/src). */
const DEFAULT_GALA_BIN = resolve(dirname(fileURLToPath(import.meta.url)), '../../../target/debug/gala');

export interface GalaAgentOptions {
  /** Path to the gala binary. Defaults to $GALA_BIN, then <repo>/target/debug/gala. */
  galaBin?: string;
  /** Per-invocation timeout in milliseconds. Default 60000. */
  timeoutMs?: number;
}

export interface Span {
  file_id: number;
  start: number;
  end: number;
  file_path?: string;
}

export interface Diagnostic {
  code: string;
  severity: 'error' | 'warning' | 'note' | 'help';
  message: string;
  primary_span: Span;
  labels: Array<Span & { message: string }>;
  notes: string[];
  suggested_fixes: Array<{ description: string; edits: Array<Span & { new_text: string }> }>;
  explain_url?: string;
  [extra: string]: unknown;
}

export interface CheckResult {
  ok: boolean;
  errors: number;
  warnings: number;
  diagnostics: Diagnostic[];
  schema: 'gala.check.v1';
}

export interface RunOptions {
  shots?: number;
  seed?: number;
}

export interface RunResult {
  ok: boolean;
  exitCode: number | null;
  /** Lines printed by print(), in order. */
  output: string[];
  /** Final value of the entry point (`result: <value>`), or null. */
  result: string | null;
  /** Shot count for measured entry points, or null. */
  shots: number | null;
  /** Outcome counts for measured entry points, keyed as printed by the CLI. */
  histogram: Record<string, number>;
  stderr: string;
  error?: string;
}

export interface GradResult {
  supported: false;
  reason: string;
}

export interface PropertyVerification {
  /** Always "unsupported" today: the CLI exposes no property runner. */
  status: 'unsupported';
  reason: string;
  requested: string[];
  /** Compile diagnostics, when the source does not type-check. */
  diagnostics: Diagnostic[];
}

interface ExecOutput {
  code: number | null;
  stdout: string;
  stderr: string;
  spawnError?: string;
  timedOut: boolean;
}

const GRAD_REASON =
  'The gala CLI exposes no gradient command yet (no `gala grad`), so gradients cannot be computed. ' +
  'No value is returned rather than a fabricated one.';

const PROPERTY_REASON =
  'The gala CLI exposes no property runner (`gala test` runs only #[test] functions). ' +
  'Properties were not checked, so nothing is reported as passing.';

export class GalaAgent {
  readonly galaBin: string;
  readonly timeoutMs: number;

  constructor(options: GalaAgentOptions = {}) {
    this.galaBin = options.galaBin ?? process.env.GALA_BIN ?? DEFAULT_GALA_BIN;
    this.timeoutMs = options.timeoutMs ?? 60_000;
  }

  /** Type-checks Gala source and returns structured diagnostics. */
  async check(code: string, options: { fileName?: string } = {}): Promise<CheckResult> {
    return this.withSource(code, options.fileName, async (path) => {
      const out = await this.exec(['check', path, '--json']);
      if (out.spawnError || out.timedOut) throw new Error(this.describeFailure('check', out));
      const doc = parseJsonObject(out.stdout);
      if (!doc || doc.schema !== 'gala.check.v1' || !Array.isArray(doc.diagnostics)) {
        throw new Error(this.describeFailure('check', out));
      }
      return doc as unknown as CheckResult;
    });
  }

  /** Runs the entry point on the simulator. Failures are reported in the result, not thrown. */
  async run(code: string, options: RunOptions = {}): Promise<RunResult> {
    const extra: string[] = [];
    if (options.shots !== undefined) extra.push('--shots', String(options.shots));
    if (options.seed !== undefined) extra.push('--seed', String(options.seed));
    return this.withSource(code, undefined, async (path) => {
      const out = await this.exec(['run', path, ...extra]);
      const base = { stderr: out.stderr.trim() };
      if (out.spawnError || out.timedOut || out.code !== 0) {
        return {
          ok: false,
          exitCode: out.code,
          output: [],
          result: null,
          shots: null,
          histogram: {},
          ...base,
          error: this.describeFailure('run', out),
        };
      }
      return { ok: true, exitCode: 0, ...parseRunStdout(out.stdout), ...base };
    });
  }

  /**
   * Gradient of `fn` with respect to `wrt`. Always unsupported today: the CLI has no
   * gradient command, so no number is returned.
   */
  async grad(fn: string, wrt: string | string[]): Promise<GradResult> {
    void fn;
    void wrt;
    return { supported: false, reason: GRAD_REASON };
  }

  /**
   * Checks quantum properties (unitary, reversible, uncomputes, grad_matches, effect_honesty).
   * Always reports "unsupported" today because the CLI exposes no property runner.
   * If the source does not type-check, its diagnostics are included.
   */
  async verifyProperties(code: string, properties: string[]): Promise<PropertyVerification> {
    const check = await this.check(code);
    return {
      status: 'unsupported',
      reason: PROPERTY_REASON,
      requested: [...properties],
      diagnostics: check.diagnostics,
    };
  }

  private async withSource<T>(
    code: string,
    fileName: string | undefined,
    fn: (path: string) => Promise<T>
  ): Promise<T> {
    const dir = await mkdtemp(join(tmpdir(), 'gala-agent-'));
    try {
      const name = sanitizeName(fileName ?? 'input.gala');
      const path = join(dir, name);
      await writeFile(path, code, 'utf8');
      return await fn(path);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  }

  private exec(args: string[]): Promise<ExecOutput> {
    return new Promise((done) => {
      execFile(
        this.galaBin,
        args,
        { timeout: this.timeoutMs, maxBuffer: 16 * 1024 * 1024, encoding: 'utf8' },
        (error, stdout, stderr) => {
          if (error && typeof (error as NodeJS.ErrnoException).code === 'string') {
            // Spawn failure (e.g. ENOENT) has a string code; non-zero exits have a numeric code.
            done({
              code: null,
              stdout,
              stderr,
              spawnError: error.message,
              timedOut: false,
            });
            return;
          }
          const killed = Boolean(error && (error as { killed?: boolean }).killed);
          done({
            code: error ? (typeof error.code === 'number' ? error.code : 1) : 0,
            stdout,
            stderr,
            timedOut: killed,
          });
        }
      );
    });
  }

  private describeFailure(what: string, out: ExecOutput): string {
    if (out.spawnError) return `gala ${what}: could not run ${this.galaBin}: ${out.spawnError}`;
    if (out.timedOut) return `gala ${what}: timed out after ${this.timeoutMs} ms`;
    const detail = (out.stderr || out.stdout).trim();
    return `gala ${what} failed (exit ${out.code})${detail ? `: ${detail}` : ''}`;
  }
}

function sanitizeName(name: string): string {
  const base = name.split(/[\\/]/).pop() ?? '';
  const safe = base.replace(/[^A-Za-z0-9_.-]/g, '_') || 'input.gala';
  return safe.endsWith('.gala') ? safe : `${safe}.gala`;
}

function parseJsonObject(text: string): Record<string, unknown> | undefined {
  try {
    const value: unknown = JSON.parse(text);
    return typeof value === 'object' && value !== null && !Array.isArray(value)
      ? (value as Record<string, unknown>)
      : undefined;
  } catch {
    return undefined;
  }
}

/** Splits `gala run` stdout into print lines, the result line, and the histogram. */
export function parseRunStdout(
  stdout: string
): Pick<RunResult, 'output' | 'result' | 'shots' | 'histogram'> {
  const trimmed = stdout.endsWith('\n') ? stdout.slice(0, -1) : stdout;
  const lines = trimmed.length === 0 ? [] : trimmed.split('\n');
  const output: string[] = [];
  const histogram: Record<string, number> = {};
  let result: string | null = null;
  let shots: number | null = null;
  for (const line of lines) {
    const resultLine = /^result: (.*)$/.exec(line);
    if (resultLine) {
      result = resultLine[1] ?? null;
      continue;
    }
    const header = /^results over (\d+) shots:$/.exec(line.trim());
    if (header) {
      shots = Number(header[1]);
      continue;
    }
    const bucket = shots !== null ? /^\s+(.+): (\d+)$/.exec(line) : null;
    if (bucket) {
      histogram[bucket[1] ?? ''] = Number(bucket[2]);
      continue;
    }
    output.push(line);
  }
  return { output, result, shots, histogram };
}

export default GalaAgent;
