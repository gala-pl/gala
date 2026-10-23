// Queries the Gala language server (gala-lsp) over stdio, one request per process.
//
// Protocol: JSON-RPC 2.0 with Content-Length framing. The sequence is initialize, initialized,
// didOpen (the source), the query, shutdown, exit.

import { spawn } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const PACKAGE_DIR = dirname(fileURLToPath(import.meta.url));

export function galaLspBinPath(): string {
  const fromEnv = process.env.GALA_LSP;
  if (fromEnv && fromEnv.length > 0) return fromEnv;
  return resolve(PACKAGE_DIR, '../../../target/debug/gala-lsp');
}

/** The LSP method behind each query type; `null` means the server does not provide it. */
export const QUERY_METHODS: Record<string, { method: string; custom: boolean } | null> = {
  hover: { method: 'textDocument/hover', custom: false },
  completion: { method: 'textDocument/completion', custom: false },
  definition: { method: 'textDocument/definition', custom: false },
  gir_at_position: { method: 'gala/girAtPosition', custom: true },
  code_lens: null,
};

export interface LspQueryResult {
  ok: boolean;
  result?: unknown;
  error?: string;
}

/** Sends one query and returns the server's result. Errors are values, never thrown. */
export async function queryLsp(opts: {
  source: string;
  filePath: string;
  position: { line: number; character: number };
  queryType: string;
  timeoutMs?: number;
}): Promise<LspQueryResult> {
  const spec = QUERY_METHODS[opts.queryType];
  if (spec === undefined) return { ok: false, error: `unknown queryType ${opts.queryType}` };
  if (spec === null) {
    return { ok: false, error: `queryType ${opts.queryType} is not provided by gala-lsp` };
  }
  const uri = `file:///${opts.filePath.replace(/^\/+/, '')}`;
  const child = spawn(galaLspBinPath(), [], { stdio: ['pipe', 'pipe', 'pipe'] });
  let buffer = Buffer.alloc(0);
  const waiting = new Map<number, (msg: any) => void>();

  child.stdout.on('data', (chunk: Buffer) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const headerEnd = buffer.indexOf('\r\n\r\n');
      if (headerEnd < 0) return;
      const header = buffer.subarray(0, headerEnd).toString('ascii');
      const m = /Content-Length:\s*(\d+)/i.exec(header);
      if (!m) return;
      const length = Number(m[1]);
      const bodyStart = headerEnd + 4;
      if (buffer.length < bodyStart + length) return;
      const msg = JSON.parse(buffer.subarray(bodyStart, bodyStart + length).toString('utf8'));
      buffer = buffer.subarray(bodyStart + length);
      if (typeof msg.id === 'number' && waiting.has(msg.id)) {
        waiting.get(msg.id)!(msg);
        waiting.delete(msg.id);
      }
    }
  });

  const send = (msg: object) => {
    const body = JSON.stringify({ jsonrpc: '2.0', ...msg });
    child.stdin.write(`Content-Length: ${Buffer.byteLength(body, 'utf8')}\r\n\r\n${body}`);
  };
  const request = (id: number, method: string, params: unknown) =>
    new Promise<any>((done) => {
      waiting.set(id, done);
      send({ id, method, params });
    });

  const timer = setTimeout(() => child.kill('SIGKILL'), opts.timeoutMs ?? 20_000);
  try {
    const init = await request(1, 'initialize', { processId: null, rootUri: null, capabilities: {} });
    if (init.error) return { ok: false, error: `initialize failed: ${init.error.message}` };
    send({ method: 'initialized', params: {} });
    send({
      method: 'textDocument/didOpen',
      params: { textDocument: { uri, languageId: 'gala', version: 1, text: opts.source } },
    });
    const params =
      spec.custom || opts.queryType === 'hover' || opts.queryType === 'definition'
        ? { textDocument: { uri }, position: opts.position }
        : { textDocument: { uri }, position: opts.position };
    const response = await request(2, spec.method, params);
    await request(3, 'shutdown', null);
    send({ method: 'exit' });
    if (response.error) return { ok: false, error: response.error.message };
    return { ok: true, result: response.result ?? null };
  } catch (err) {
    return { ok: false, error: `language server query failed: ${(err as Error).message}` };
  } finally {
    clearTimeout(timer);
    child.kill();
  }
}
