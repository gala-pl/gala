// MCP tool definitions and handlers. Each handler shells out to the gala CLI and
// validates its structured output before returning it.

import { readFileSync } from 'node:fs';
import { basename } from 'node:path';
import { type GalaProcess, runGala, withSourceFile } from './gala-cli.ts';
import { type Schema, validate } from './jsonschema.ts';

export interface TextContent {
  type: 'text';
  text: string;
}

export interface ToolResult {
  content: TextContent[];
  isError?: boolean;
}

export const SCHEMA_URL = new URL('./schemas/', import.meta.url);

export const SCHEMA_FILES: Record<string, string> = {
  'diagnostic.v1': 'diagnostic.v1.json',
  'gir.v1': 'gir.v1.json',
  'property_result.v1': 'property_result.v1.json',
};

export function loadSchema(name: string): Schema {
  const file = SCHEMA_FILES[name];
  if (!file) throw new Error(`unknown schema ${name}`);
  return JSON.parse(readFileSync(new URL(file, SCHEMA_URL), 'utf8')) as Schema;
}

const DIAGNOSTIC_SCHEMA = (): Schema => loadSchema('diagnostic.v1');

const RULE_CODES = ['unitary', 'reversible', 'uncomputes', 'grad_matches', 'effect_honesty'];

export const TOOL_DEFINITIONS = [
  {
    name: 'gala_check',
    description:
      'Type-check Gala source and return gala.check.v1 JSON with diagnostics (code, spans, labels, suggested fixes).',
    inputSchema: {
      type: 'object',
      properties: {
        source: { type: 'string', description: 'Gala source code' },
        file_path: { type: 'string', description: 'Optional file name used in diagnostics' },
      },
      required: ['source'],
      additionalProperties: false,
    },
  },
  {
    name: 'gala_build',
    description:
      'Compile Gala source and return the text IR, or gir.v1 JSON when emit_gir is true.',
    inputSchema: {
      type: 'object',
      properties: {
        source: { type: 'string', description: 'Gala source code' },
        emit_gir: { type: 'boolean', description: 'Return gir.v1 JSON instead of text IR' },
        gir_version: { type: 'integer', description: 'GIR schema version (only 1)' },
      },
      required: ['source'],
      additionalProperties: false,
    },
  },
  {
    name: 'gala_run',
    description:
      'Run a Gala program on the built-in simulator. Returns print() output and, for measured results, a histogram.',
    inputSchema: {
      type: 'object',
      properties: {
        source: { type: 'string', description: 'Gala source code' },
        shots: { type: 'integer', minimum: 1, description: 'Number of shots' },
        seed: { type: 'integer', minimum: 0, description: 'Random seed' },
      },
      required: ['source'],
      additionalProperties: false,
    },
  },
  {
    name: 'gala_test',
    description: 'Run the #[test] functions in a Gala source file.',
    inputSchema: {
      type: 'object',
      properties: {
        source: { type: 'string', description: 'Gala source code' },
        properties: {
          type: 'array',
          items: { type: 'string', enum: RULE_CODES },
          description: 'Property kinds to run, e.g. grad_matches; empty runs all declared',
        },
      },
      required: ['source'],
      additionalProperties: false,
    },
  },
  {
    name: 'gala_fix',
    description:
      'Suggest validated fixes for Gala source (gala fix --json). Fixes are only offered when they reduce the error count.',
    inputSchema: {
      type: 'object',
      properties: { source: { type: 'string', description: 'Gala source code' } },
      required: ['source'],
      additionalProperties: false,
    },
  },
  {
    name: 'gala_explain',
    description: 'Explain a Gala diagnostic code such as E0412.',
    inputSchema: {
      type: 'object',
      properties: {
        code: { type: 'string', pattern: '^E[0-9]{4}$', description: 'Error code, e.g. E0412' },
      },
      required: ['code'],
      additionalProperties: false,
    },
  },
  {
    name: 'gala_gir',
    description:
      'Return gir.v1 JSON for a Gala source (gala build --emit gir=json).',
    inputSchema: {
      type: 'object',
      properties: { source: { type: 'string', description: 'Gala source code' } },
      required: ['source'],
      additionalProperties: false,
    },
  },
  {
    name: 'gala_lsp_query',
    description:
      'Query the Gala language server at a position. Not available: this server does not start or connect to gala-lsp.',
    inputSchema: {
      type: 'object',
      properties: {
        source: { type: 'string', description: 'Gala source code' },
        file_path: { type: 'string', description: 'File name for the document' },
        position: {
          type: 'object',
          properties: {
            line: { type: 'integer', minimum: 0 },
            character: { type: 'integer', minimum: 0 },
          },
          required: ['line', 'character'],
        },
        queryType: {
          type: 'string',
          enum: ['hover', 'completion', 'definition', 'code_lens', 'gir_at_position'],
        },
      },
      required: ['source', 'file_path', 'position', 'queryType'],
      additionalProperties: false,
    },
  },
] as const;

export type ToolName = (typeof TOOL_DEFINITIONS)[number]['name'];

// ---------- result helpers ----------

export function okText(value: unknown): ToolResult {
  const text = typeof value === 'string' ? value : JSON.stringify(value, null, 2);
  return { content: [{ type: 'text', text }] };
}

export function errorText(message: string): ToolResult {
  return { isError: true, content: [{ type: 'text', text: message }] };
}

function parseJson(text: string): unknown | undefined {
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

function procFailure(what: string, p: GalaProcess): ToolResult {
  if (p.spawnError) {
    return errorText(`${what}: could not run gala binary: ${p.spawnError}`);
  }
  if (p.timedOut) return errorText(`${what}: timed out`);
  const detail = (p.stderr || p.stdout).trim();
  return errorText(`${what}: gala exited with code ${p.exitCode}${detail ? `\n${detail}` : ''}`);
}

// ---------- gala_check ----------

/** Validates a gala.check.v1 document; returns a list of problems (empty when valid). */
export function checkCheckOutput(doc: unknown): string[] {
  if (typeof doc !== 'object' || doc === null || Array.isArray(doc)) {
    return ['top-level value is not an object'];
  }
  const d = doc as Record<string, unknown>;
  const problems: string[] = [];
  if (d.schema !== 'gala.check.v1') problems.push('schema is not gala.check.v1');
  for (const key of ['ok', 'errors', 'warnings', 'diagnostics']) {
    if (!(key in d)) problems.push(`missing ${key}`);
  }
  if (!Array.isArray(d.diagnostics)) {
    problems.push('diagnostics is not an array');
  } else {
    d.diagnostics.forEach((diag, i) => {
      for (const issue of validate(diag, DIAGNOSTIC_SCHEMA(), `diagnostics[${i}]`)) {
        problems.push(`${issue.path}: ${issue.message}`);
      }
    });
  }
  return problems;
}

async function galaCheck(args: { source: string; file_path?: string }): Promise<ToolResult> {
  const reportName = args.file_path ? basename(args.file_path) : 'input.gala';
  return withSourceFile(args.source, reportName, async (path) => {
    const p = await runGala(['check', path, '--json']);
    if (p.spawnError || p.timedOut) return procFailure('gala_check', p);
    const doc = parseJson(p.stdout);
    if (doc === undefined) return procFailure('gala_check (no JSON output)', p);
    const problems = checkCheckOutput(doc);
    if (problems.length > 0) {
      return errorText(`gala_check output failed schema validation: ${problems.join('; ')}`);
    }
    // Report the caller's file name instead of the temp path.
    const out = doc as { diagnostics: Array<Record<string, unknown>> };
    for (const diag of out.diagnostics) {
      rewritePaths(diag, path, reportName);
    }
    return okText(out);
  });
}

function rewritePaths(node: unknown, from: string, to: string): void {
  if (Array.isArray(node)) {
    for (const item of node) rewritePaths(item, from, to);
  } else if (typeof node === 'object' && node !== null) {
    for (const [k, v] of Object.entries(node)) {
      if (k === 'file_path' && v === from) (node as Record<string, unknown>)[k] = to;
      else rewritePaths(v, from, to);
    }
  }
}

// ---------- gala_build ----------

async function galaBuild(args: {
  source: string;
  emit_gir?: boolean;
  gir_version?: number;
}): Promise<ToolResult> {
  if (args.gir_version !== undefined && args.gir_version !== 1) {
    return errorText(`gala_build: unsupported gir_version ${args.gir_version} (only 1 exists)`);
  }
  if (args.emit_gir) return galaGir(args.source);
  return withSourceFile(args.source, 'input.gala', async (path) => {
    const p = await runGala(['build', path]);
    if (p.spawnError || p.timedOut || p.exitCode !== 0) return procFailure('gala_build', p);
    return okText({ ok: true, format: 'text-ir', ir: p.stdout });
  });
}

// ---------- gala_run ----------

export interface RunOutput {
  ok: boolean;
  exit_code: number;
  shots: number | null;
  result: string | null;
  output: string[];
  histogram: Record<string, number>;
}

/** Splits `gala run` stdout into print lines and the histogram block. */
export function parseRunStdout(stdout: string): Omit<RunOutput, 'ok' | 'exit_code'> {
  const trimmed = stdout.endsWith('\n') ? stdout.slice(0, -1) : stdout;
  const lines = trimmed.length === 0 ? [] : trimmed.split('\n');
  const output: string[] = [];
  const histogram: Record<string, number> = {};
  let shots: number | null = null;
  let result: string | null = null;
  for (const line of lines) {
    const resultLine = /^result: (.*)$/.exec(line);
    if (resultLine) {
      result = resultLine[1] as string;
      continue;
    }
    const header = /^results over (\d+) shots:$/.exec(line.trim());
    if (header) {
      shots = Number(header[1]);
      continue;
    }
    const bucket = shots !== null ? /^\s+(.+): (\d+)$/.exec(line) : null;
    if (bucket) {
      histogram[bucket[1] as string] = Number(bucket[2]);
      continue;
    }
    output.push(line);
  }
  return { shots, result, output, histogram };
}

export function checkRunOutput(doc: unknown): string[] {
  const problems: string[] = [];
  if (typeof doc !== 'object' || doc === null) return ['not an object'];
  const d = doc as Record<string, unknown>;
  if (typeof d.ok !== 'boolean') problems.push('ok must be boolean');
  if (!Number.isInteger(d.exit_code)) problems.push('exit_code must be integer');
  if (!(d.shots === null || Number.isInteger(d.shots))) problems.push('shots must be integer or null');
  if (!(d.result === null || typeof d.result === 'string')) problems.push('result must be string or null');
  if (!Array.isArray(d.output) || !d.output.every((s) => typeof s === 'string')) {
    problems.push('output must be string[]');
  }
  if (typeof d.histogram !== 'object' || d.histogram === null) problems.push('histogram must be object');
  return problems;
}

async function galaRun(args: { source: string; shots?: number; seed?: number }): Promise<ToolResult> {
  const extra: string[] = [];
  if (args.shots !== undefined) extra.push('--shots', String(args.shots));
  if (args.seed !== undefined) extra.push('--seed', String(args.seed));
  return withSourceFile(args.source, 'input.gala', async (path) => {
    const p = await runGala(['run', path, ...extra]);
    if (p.spawnError || p.timedOut || p.exitCode !== 0) return procFailure('gala_run', p);
    const doc: RunOutput = { ok: true, exit_code: 0, ...parseRunStdout(p.stdout) };
    const problems = checkRunOutput(doc);
    if (problems.length > 0) return errorText(`gala_run output invalid: ${problems.join('; ')}`);
    return okText(doc);
  });
}

// ---------- gala_test ----------

async function galaTest(args: { source: string; properties?: string[] }): Promise<ToolResult> {
  if (args.properties && args.properties.length > 0) {
    return withSourceFile(args.source, 'input.gala', async (path) => {
      const p = await runGala(['test', path, '--property', '--json']);
      if (p.spawnError || p.timedOut) return procFailure('gala_test', p);
      let doc: { results: Array<{ property: string; passed: boolean }>; skipped: unknown[] };
      try {
        doc = JSON.parse(p.stdout);
      } catch {
        return procFailure('gala_test', p);
      }
      const wanted = new Set(args.properties);
      const results = doc.results.filter((r) => wanted.has(r.property));
      const failed = results.filter((r) => !r.passed).length;
      return okText({ ok: failed === 0, exit_code: p.exitCode, failed, results, skipped: doc.skipped });
    });
  }
  return withSourceFile(args.source, 'input.gala', async (path) => {
    const p = await runGala(['test', path]);
    if (p.spawnError || p.timedOut) return procFailure('gala_test', p);
    const summary = /(\d+) tests?, (\d+) failed/.exec(p.stdout);
    if (p.exitCode !== 0 || !summary) return procFailure('gala_test', p);
    const failed = Number(summary[2]);
    return okText({
      ok: failed === 0,
      exit_code: p.exitCode,
      tests: Number(summary[1]),
      failed,
      output: p.stdout.replace(/\n$/, '').split('\n').filter((l) => l.length > 0),
    });
  });
}

// ---------- gala_fix / gala_gir / gala_lsp_query (unavailable) ----------

async function galaFix(source: string): Promise<ToolResult> {
  return withSourceFile(source, 'input.gala', async (path) => {
    const p = await runGala(['fix', path, '--json']);
    if (p.spawnError || p.timedOut || p.exitCode !== 0) return procFailure('gala_fix', p);
    let doc: unknown;
    try {
      doc = JSON.parse(p.stdout);
    } catch {
      return procFailure('gala_fix', p);
    }
    return okText(doc);
  });
}

async function galaGir(source: string): Promise<ToolResult> {
  return withSourceFile(source, 'input.gala', async (path) => {
    const p = await runGala(['build', path, '--emit', 'gir=json']);
    if (p.spawnError || p.timedOut || p.exitCode !== 0) return procFailure('gala_gir', p);
    let doc: { version?: unknown };
    try {
      doc = JSON.parse(p.stdout);
    } catch {
      return procFailure('gala_gir', p);
    }
    if (doc.version !== 1) return errorText('gala_gir: CLI output is not gir.v1');
    return okText(doc);
  });
}

function galaLspQuery(args: { queryType: string }): ToolResult {
  return errorText(
    `gala_lsp_query is not available: the Gala language server is not running or connected ` +
      `to this MCP server (requested queryType: ${args.queryType}).`
  );
}

// ---------- gala_explain ----------

async function galaExplain(args: { code: string }): Promise<ToolResult> {
  const p = await runGala(['explain', args.code, '--json']);
  if (p.spawnError || p.timedOut || p.exitCode !== 0) return procFailure('gala_explain', p);
  const doc = parseJson(p.stdout);
  if (typeof doc !== 'object' || doc === null) return errorText('gala_explain: invalid JSON output');
  const d = doc as Record<string, unknown>;
  if (typeof d.code !== 'string' || typeof d.title !== 'string' || typeof d.explanation !== 'string') {
    return errorText('gala_explain: output missing code/title/explanation');
  }
  return okText(doc);
}

// ---------- dispatch ----------

export function validateArgs(name: ToolName, args: unknown): string[] {
  const def = TOOL_DEFINITIONS.find((t) => t.name === name);
  if (!def) return [`unknown tool ${name}`];
  return validate(args, def.inputSchema as unknown as Schema, 'arguments').map(
    (i) => `${i.path}: ${i.message}`
  );
}

export async function callTool(name: ToolName, args: Record<string, unknown>): Promise<ToolResult> {
  switch (name) {
    case 'gala_check':
      return galaCheck(args as { source: string; file_path?: string });
    case 'gala_build':
      return galaBuild(args as { source: string; emit_gir?: boolean; gir_version?: number });
    case 'gala_run':
      return galaRun(args as { source: string; shots?: number; seed?: number });
    case 'gala_test':
      return galaTest(args as { source: string; properties?: string[] });
    case 'gala_fix':
      return galaFix((args as { source: string }).source);
    case 'gala_explain':
      return galaExplain(args as { code: string });
    case 'gala_gir':
      return galaGir((args as { source: string }).source);
    case 'gala_lsp_query':
      return galaLspQuery(args as { queryType: string });
  }
}
