import { afterAll, beforeAll, describe, expect, test } from 'bun:test';
import { type ChildProcessWithoutNullStreams, spawn } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { checkCheckOutput, parseRunStdout, checkRunOutput } from '../src/tools.ts';
import { validate } from '../src/jsonschema.ts';

const REPO = resolve(import.meta.dir, '../../..');
const SERVER = resolve(import.meta.dir, '../src/index.ts');
const CHECK_FIXTURE = resolve(REPO, 'tests/conformance/linearity/use_after_consume.gala');
const RUN_FIXTURE = resolve(REPO, 'tests/conformance/run/bell_run.gala');

/** Minimal JSON-RPC client over the server's stdio. */
class McpClient {
  private proc: ChildProcessWithoutNullStreams;
  private buffer = '';
  private nextId = 1;
  private pending = new Map<number, (msg: any) => void>();

  constructor() {
    this.proc = spawn(process.execPath, [SERVER], { stdio: ['pipe', 'pipe', 'pipe'] });
    this.proc.stdout.setEncoding('utf8');
    this.proc.stdout.on('data', (chunk: string) => {
      this.buffer += chunk;
      let idx = this.buffer.indexOf('\n');
      while (idx >= 0) {
        const line = this.buffer.slice(0, idx);
        this.buffer = this.buffer.slice(idx + 1);
        if (line.trim()) {
          const msg = JSON.parse(line);
          const waiter = this.pending.get(msg.id);
          if (waiter) {
            this.pending.delete(msg.id);
            waiter(msg);
          }
        }
        idx = this.buffer.indexOf('\n');
      }
    });
  }

  request(method: string, params: Record<string, unknown> = {}): Promise<any> {
    const id = this.nextId++;
    return new Promise((done, fail) => {
      const timer = setTimeout(() => fail(new Error(`timeout waiting for ${method}`)), 60_000);
      this.pending.set(id, (msg) => {
        clearTimeout(timer);
        done(msg);
      });
      this.proc.stdin.write(`${JSON.stringify({ jsonrpc: '2.0', id, method, params })}\n`);
    });
  }

  notify(method: string, params: Record<string, unknown> = {}): void {
    this.proc.stdin.write(`${JSON.stringify({ jsonrpc: '2.0', method, params })}\n`);
  }

  close(): void {
    this.proc.kill();
  }
}

/** Parses the JSON text of a tool result. */
function toolJson(result: any): any {
  return JSON.parse(result.content[0].text);
}

describe('gala-mcp stdio server', () => {
  let client: McpClient;

  beforeAll(() => {
    client = new McpClient();
  });

  afterAll(() => {
    client.close();
  });

  test('initialize returns server info and capabilities', async () => {
    const res = await client.request('initialize', {
      protocolVersion: '2025-06-18',
      capabilities: {},
      clientInfo: { name: 'test', version: '0' },
    });
    expect(res.jsonrpc).toBe('2.0');
    expect(res.result.serverInfo.name).toBe('gala-mcp');
    expect(res.result.protocolVersion).toBe('2025-06-18');
    expect(res.result.capabilities.tools).toBeDefined();
    expect(res.result.capabilities.resources).toBeDefined();
    expect(res.result.capabilities.prompts).toBeDefined();
    client.notify('notifications/initialized');
  });

  test('tools/list advertises every gala tool', async () => {
    const res = await client.request('tools/list');
    const names = res.result.tools.map((t: { name: string }) => t.name).sort();
    expect(names).toEqual(
      [
        'gala_build',
        'gala_check',
        'gala_explain',
        'gala_fix',
        'gala_gir',
        'gala_lsp_query',
        'gala_run',
        'gala_test',
      ].sort()
    );
  });

  test('gala_check reports E0412 for use_after_consume.gala', async () => {
    const source = await Bun.file(CHECK_FIXTURE).text();
    const res = await client.request('tools/call', {
      name: 'gala_check',
      arguments: { source, file_path: 'use_after_consume.gala' },
    });
    expect(res.result.isError).toBeFalsy();
    const doc = toolJson(res.result);
    expect(checkCheckOutput(doc)).toEqual([]);
    expect(doc.ok).toBe(false);
    const codes = doc.diagnostics.map((d: { code: string }) => d.code);
    expect(codes).toContain('E0412');
    const e0412 = doc.diagnostics.find((d: { code: string }) => d.code === 'E0412');
    expect(e0412.primary_span.file_path).toBe('use_after_consume.gala');
  });

  test('gala_run prints true for bell_run.gala', async () => {
    const source = await Bun.file(RUN_FIXTURE).text();
    const res = await client.request('tools/call', {
      name: 'gala_run',
      arguments: { source, shots: 4, seed: 1 },
    });
    expect(res.result.isError).toBeFalsy();
    const doc = toolJson(res.result);
    expect(checkRunOutput(doc)).toEqual([]);
    expect(doc.output[0]).toBe('true');
    expect(doc.result).toBe('0');
  });

  test('gala_fix, gala_gir and gala_lsp_query return error results, not fake success', async () => {
    for (const [name, args] of [
      ['gala_fix', { source: 'fn main() -> Int { 0 }' }],
      ['gala_gir', { source: 'fn main() -> Int { 0 }' }],
      [
        'gala_lsp_query',
        {
          source: 'fn main() -> Int { 0 }',
          file_path: 'a.gala',
          position: { line: 0, character: 0 },
          queryType: 'hover',
        },
      ],
    ] as const) {
      const res = await client.request('tools/call', { name, arguments: args });
      expect(res.result.isError).toBe(true);
    }
  });

  test('invalid arguments produce a tool error result', async () => {
    const res = await client.request('tools/call', {
      name: 'gala_explain',
      arguments: { code: 'E12' },
    });
    expect(res.result.isError).toBe(true);
  });

  test('resources/read serves the diagnostic schema and diagnostics for a file', async () => {
    const schema = await client.request('resources/read', { uri: 'gala://schema/diagnostic.v1' });
    expect(JSON.parse(schema.result.contents[0].text).title).toBe('Gala Diagnostic');

    const uri = `gala://diagnostics/${encodeURIComponent(CHECK_FIXTURE)}`;
    const diags = await client.request('resources/read', { uri });
    expect(JSON.parse(diags.result.contents[0].text).schema).toBe('gala.check.v1');
  });

  test('prompts/get includes the Gala file content', async () => {
    const res = await client.request('prompts/get', {
      name: 'gala/debug-error',
      arguments: { file: CHECK_FIXTURE },
    });
    const text = res.result.messages[0].content.text;
    expect(text).toContain('E0412');
    expect(text).toContain('measure(a)');
  });

  test('unknown method returns method-not-found', async () => {
    const res = await client.request('nope/nope');
    expect(res.error.code).toBe(-32601);
  });
});

describe('helpers', () => {
  test('jsonschema rejects a diagnostic with a bad code and a wrong type', () => {
    const schema = JSON.parse(
      readFileSync(resolve(import.meta.dir, '../src/schemas/diagnostic.v1.json'), 'utf8')
    );
    const bad = {
      code: 'X1',
      severity: 'error',
      message: 3,
      primary_span: { file_id: 0, start: 0, end: 1 },
      labels: [],
      notes: [],
      suggested_fixes: [],
    };
    const issues = validate(bad, schema);
    expect(issues.some((i) => i.path === '$.code')).toBe(true);
    expect(issues.some((i) => i.path === '$.message')).toBe(true);
  });

  test('parseRunStdout splits prints from the histogram', () => {
    const parsed = parseRunStdout('true\nresults over 4 shots:\n  0: 4\n');
    expect(parsed.output).toEqual(['true']);
    expect(parsed.histogram).toEqual({ '0': 4 });
    expect(parsed.shots).toBe(4);
    const scalar = parseRunStdout('true\nresult: 0\n');
    expect(scalar.output).toEqual(['true']);
    expect(scalar.result).toBe('0');
  });
});
