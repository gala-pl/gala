import { type SpawnOptions, spawn } from 'node:child_process';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import type {
  CheckResult,
  ExplainResult,
  FixResult,
  GirResult,
  LspQueryResult,
  RunResult,
  TestResult,
} from './types.js';
import {
  CheckResultSchema,
  ExplainResultSchema,
  FixResultSchema,
  GirResultSchema,
  LspQueryResultSchema,
  RunResultSchema,
  TestResultSchema,
} from './types.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = join(__filename, '..');

async function findGalaBinary(): Promise<string> {
  const candidates = [
    join(__dirname, '../../../target/release/gala'),
    join(__dirname, '../../../target/debug/gala'),
    'gala',
  ];
  for (const c of candidates) {
    try {
      const { stdout } = await runCommand(c, ['--version']);
      if (stdout.includes('gala')) return c;
    } catch {}
  }
  throw new Error('gala binary not found. Run `cargo build --release` or install gala.');
}

let galaBinaryCache: string | null = null;

async function getGalaBinary(): Promise<string> {
  if (galaBinaryCache) return galaBinaryCache;
  galaBinaryCache = await findGalaBinary();
  return galaBinaryCache;
}

interface RunOptions extends SpawnOptions {
  timeout?: number;
  input?: string;
}

async function runCommand(
  command: string,
  args: string[],
  options: RunOptions = {}
): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  return new Promise((resolve, reject) => {
    const proc = spawn(command, args, {
      ...options,
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';

    proc.stdout?.on('data', (data) => {
      stdout += data.toString();
    });
    proc.stderr?.on('data', (data) => {
      stderr += data.toString();
    });

    if (options.input) {
      proc.stdin?.write(options.input);
      proc.stdin?.end();
    }

    const timeout = setTimeout(() => {
      proc.kill('SIGTERM');
      reject(new Error(`Command timed out after ${options.timeout ?? 30000}ms`));
    }, options.timeout ?? 30000);

    proc.on('close', (code) => {
      clearTimeout(timeout);
      resolve({ stdout, stderr, exitCode: code ?? 0 });
    });

    proc.on('error', (err) => {
      clearTimeout(timeout);
      reject(err);
    });
  });
}

export async function galaCheck(source: string, filePath?: string): Promise<CheckResult> {
  const gala = await getGalaBinary();
  const args = ['check', '--json'];
  if (filePath) args.push('--input', filePath);

  const { stdout, stderr, exitCode } = await runCommand(gala, args, {
    input: source,
  });

  if (exitCode !== 0 && exitCode !== 1) {
    throw new Error(`gala check failed: ${stderr}`);
  }

  const parsed = JSON.parse(stdout);
  return CheckResultSchema.parse(parsed);
}

export async function galaBuild(
  source: string,
  options: { emitGir?: boolean; girVersion?: number } = {}
): Promise<GirResult> {
  const gala = await getGalaBinary();
  const args = ['build', '--json'];
  if (options.emitGir) {
    args.push('--emit', 'gir=json');
    if (options.girVersion) args.push('--gir-version', String(options.girVersion));
  }

  const { stdout, stderr, exitCode } = await runCommand(gala, args, { input: source });

  if (exitCode !== 0) {
    throw new Error(`gala build failed: ${stderr}`);
  }

  const parsed = JSON.parse(stdout);
  return GirResultSchema.parse(parsed);
}

export async function galaRun(
  source: string,
  options: { shots?: number; seed?: number } = {}
): Promise<RunResult> {
  const gala = await getGalaBinary();
  const args = ['run', '--json'];
  if (options.shots) args.push('--shots', String(options.shots));
  if (options.seed) args.push('--seed', String(options.seed));

  const { stdout, stderr, exitCode } = await runCommand(gala, args, { input: source });

  if (exitCode !== 0) {
    throw new Error(`gala run failed: ${stderr}`);
  }

  const parsed = JSON.parse(stdout);
  return RunResultSchema.parse(parsed);
}

export async function galaTest(source: string, properties: string[] = []): Promise<TestResult> {
  const gala = await getGalaBinary();
  const args = ['test', '--property', '--json'];
  for (const p of properties) {
    args.push('--property', p);
  }

  const { stdout, stderr, exitCode } = await runCommand(gala, args, { input: source });

  if (exitCode !== 0 && exitCode !== 1) {
    throw new Error(`gala test failed: ${stderr}`);
  }

  const parsed = JSON.parse(stdout);
  return TestResultSchema.parse(parsed);
}

export async function galaFix(source: string): Promise<FixResult> {
  const gala = await getGalaBinary();
  const args = ['fix', '--json'];

  const { stdout, stderr, exitCode } = await runCommand(gala, args, { input: source });

  if (exitCode !== 0) {
    throw new Error(`gala fix failed: ${stderr}`);
  }

  const parsed = JSON.parse(stdout);
  return FixResultSchema.parse(parsed);
}

export async function galaExplain(code: string): Promise<ExplainResult> {
  const gala = await getGalaBinary();
  const args = ['explain', code, '--markdown'];

  const { stdout, stderr, exitCode } = await runCommand(gala, args);

  if (exitCode !== 0) {
    throw new Error(`gala explain failed: ${stderr}`);
  }

  const parsed = JSON.parse(stdout);
  return ExplainResultSchema.parse(parsed);
}

export async function galaGir(source: string): Promise<GirResult> {
  return galaBuild(source, { emitGir: true, girVersion: 1 });
}

export async function galaLspQuery(_input: {
  source: string;
  filePath: string;
  position: { line: number; character: number };
  queryType: 'hover' | 'completion' | 'definition' | 'code_lens' | 'gir_at_position';
}): Promise<LspQueryResult> {
  // This would connect to a running gala-lsp instance
  // For now, return a placeholder
  return LspQueryResultSchema.parse({
    hover: undefined,
    completion: undefined,
    definition: undefined,
    code_lens: undefined,
    gir_at_position: undefined,
  });
}
