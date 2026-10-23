// MCP server core: JSON-RPC 2.0 over newline-delimited stdio. No third-party dependencies.

import { existsSync, readFileSync, statSync } from 'node:fs';
import { createInterface } from 'node:readline';
import { isAbsolute } from 'node:path';
import { runGala } from './gala-cli.ts';
import { TOOL_DEFINITIONS, callTool, checkCheckOutput, errorText, loadSchema, validateArgs, type ToolName } from './tools.ts';

export const SERVER_INFO = { name: 'gala-mcp', version: '0.1.0' };
const SUPPORTED_PROTOCOLS = ['2025-06-18', '2025-03-26', '2024-11-05'];

export class RpcError extends Error {
  constructor(
    readonly code: number,
    message: string,
    readonly data?: unknown
  ) {
    super(message);
  }
}

type JsonObject = Record<string, unknown>;

const isObject = (v: unknown): v is JsonObject =>
  typeof v === 'object' && v !== null && !Array.isArray(v);

// ---------- resources ----------

const STATIC_RESOURCES = [
  {
    uri: 'gala://schema/diagnostic.v1',
    name: 'diagnostic.v1 schema',
    description: 'JSON Schema for Gala diagnostics',
    mimeType: 'application/json',
  },
  {
    uri: 'gala://schema/gir.v1',
    name: 'gir.v1 schema',
    description: 'JSON Schema for Gala IR (GIR) v1',
    mimeType: 'application/json',
  },
  {
    uri: 'gala://schema/property_result.v1',
    name: 'property_result.v1 schema',
    description: 'JSON Schema for quantum property test results',
    mimeType: 'application/json',
  },
];

const RESOURCE_TEMPLATES = [
  {
    uriTemplate: 'gala://diagnostics/{file}',
    name: 'Diagnostics for a .gala file',
    description: 'gala.check.v1 output for an absolute path to a .gala file (percent-encoded)',
    mimeType: 'application/json',
  },
  {
    uriTemplate: 'gala://gir/{file}',
    name: 'GIR for a .gala file',
    description: 'Not available: the gala CLI cannot emit gir.v1 JSON yet',
    mimeType: 'application/json',
  },
];

/** Resolves an absolute path to an existing .gala file, or throws. */
export function galaFilePath(raw: string): string {
  const path = decodeURIComponent(raw);
  if (!isAbsolute(path)) throw new RpcError(-32602, `path must be absolute: ${path}`);
  if (!path.endsWith('.gala')) throw new RpcError(-32602, `not a .gala file: ${path}`);
  if (!existsSync(path) || !statSync(path).isFile()) {
    throw new RpcError(-32602, `file not found: ${path}`);
  }
  return path;
}

async function readResource(uri: string): Promise<JsonObject> {
  const schemaNames: Record<string, string> = {
    'gala://schema/diagnostic.v1': 'diagnostic.v1',
    'gala://schema/gir.v1': 'gir.v1',
    'gala://schema/property_result.v1': 'property_result.v1',
  };
  if (schemaNames[uri]) {
    const schema = loadSchema(schemaNames[uri] as string);
    return contents(uri, JSON.stringify(schema, null, 2));
  }
  if (uri.startsWith('gala://diagnostics/')) {
    const path = galaFilePath(uri.slice('gala://diagnostics/'.length));
    const p = await runGala(['check', path, '--json']);
    if (p.spawnError || p.timedOut) throw new RpcError(-32603, 'gala binary failed to run');
    let doc: unknown;
    try {
      doc = JSON.parse(p.stdout);
    } catch {
      throw new RpcError(-32603, `gala check produced no JSON: ${p.stderr.trim()}`);
    }
    const problems = checkCheckOutput(doc);
    if (problems.length > 0) throw new RpcError(-32603, `invalid output: ${problems.join('; ')}`);
    return contents(uri, JSON.stringify(doc, null, 2));
  }
  if (uri.startsWith('gala://gir/')) {
    throw new RpcError(-32002, 'GIR JSON is not available: the gala CLI cannot emit gir.v1 yet');
  }
  throw new RpcError(-32002, `resource not found: ${uri}`);
}

function contents(uri: string, text: string): JsonObject {
  return { contents: [{ uri, mimeType: 'application/json', text }] };
}

// ---------- prompts ----------

const PROMPTS = [
  {
    name: 'gala/debug-error',
    description: 'Explain a Gala file’s diagnostics in quantum-physics terms and suggest fixes',
    arguments: [
      { name: 'file', description: 'Absolute path to a .gala file', required: true },
    ],
  },
  {
    name: 'gala/write-quantum-fn',
    description: 'Write a Gala quantum function implementing a specific algorithm',
    arguments: [
      { name: 'task', description: "e.g. 'Bell pair preparation'", required: true },
      { name: 'constraints', description: "e.g. 'no mid-circuit measurement'", required: false },
    ],
  },
  {
    name: 'gala/optimize-circuit',
    description: 'Optimize a Gala quantum function (gate count and depth)',
    arguments: [
      { name: 'file', description: 'Absolute path to a .gala file', required: true },
      { name: 'objectives', description: "e.g. 'minimize 2q gates'", required: false },
    ],
  },
];

const BELL_EXAMPLE = `fn main() -> Measured<Bool> prob {
    let a = qubit()
    let b = qubit()
    h(a)
    let (a, b) = cx(a, b)
    drop(b)
    measure(a)
}`;

function userMessage(text: string): JsonObject {
  return { role: 'user', content: { type: 'text', text } };
}

async function getPrompt(name: string, args: Record<string, string | undefined>): Promise<JsonObject> {
  const requireArg = (key: string): string => {
    const v = args[key];
    if (!v) throw new RpcError(-32602, `missing required argument: ${key}`);
    return v;
  };

  if (name === 'gala/debug-error') {
    const path = galaFilePath(requireArg('file'));
    const source = readFileSync(path, 'utf8');
    const p = await runGala(['check', path, '--json']);
    let diagnostics: string;
    try {
      diagnostics = JSON.stringify(JSON.parse(p.stdout), null, 2);
    } catch {
      diagnostics = `(gala check produced no JSON) ${p.stderr.trim()}`;
    }
    return {
      description: 'Debug Gala diagnostics',
      messages: [
        userMessage(
          `Analyze these Gala diagnostics. Explain the quantum-mechanical reason each error ` +
            `occurs (linearity, no-cloning, reversibility, uncomputation) and suggest concrete fixes.\n\n` +
            `Diagnostics (gala.check.v1):\n\`\`\`json\n${diagnostics}\n\`\`\`\n\n` +
            `Source (${path}):\n\`\`\`gala\n${source}\n\`\`\``
        ),
      ],
    };
  }

  if (name === 'gala/write-quantum-fn') {
    const task = requireArg('task');
    const constraints = args.constraints ?? 'none';
    return {
      description: 'Write a Gala quantum function',
      messages: [
        userMessage(
          `Write a Gala quantum function that implements: ${task}\n\n` +
            `Constraints: ${constraints}\n\n` +
            `Requirements:\n- Use Gala syntax with an effect annotation (pure, quantum or prob).\n` +
            `- Respect linearity: every qubit is consumed exactly once (measure, drop, or returned).\n` +
            `- Verify the result with gala_check before returning it.\n\n` +
            `Reference example that type-checks:\n\`\`\`gala\n${BELL_EXAMPLE}\n\`\`\``
        ),
      ],
    };
  }

  if (name === 'gala/optimize-circuit') {
    const path = galaFilePath(requireArg('file'));
    const source = readFileSync(path, 'utf8');
    const objectives = args.objectives ?? 'minimize depth and two-qubit gate count';
    const build = await runGala(['build', path]);
    const ir = build.exitCode === 0 ? build.stdout : `(build failed) ${build.stderr.trim()}`;
    return {
      description: 'Optimize a Gala quantum function',
      messages: [
        userMessage(
          `Optimize this Gala function. Objectives: ${objectives}. Keep it semantically equivalent ` +
            `and keep it type-checking with gala_check.\n\n` +
            `Source (${path}):\n\`\`\`gala\n${source}\n\`\`\`\n\n` +
            `Current text IR:\n\`\`\`\n${ir}\n\`\`\``
        ),
      ],
    };
  }

  throw new RpcError(-32602, `unknown prompt: ${name}`);
}

// ---------- JSON-RPC dispatch ----------

async function dispatch(method: string, params: JsonObject): Promise<unknown> {
  switch (method) {
    case 'initialize': {
      const requested = typeof params.protocolVersion === 'string' ? params.protocolVersion : '';
      return {
        protocolVersion: SUPPORTED_PROTOCOLS.includes(requested) ? requested : SUPPORTED_PROTOCOLS[0],
        capabilities: {
          tools: { listChanged: false },
          resources: { subscribe: false, listChanged: false },
          prompts: { listChanged: false },
        },
        serverInfo: SERVER_INFO,
        instructions:
          'Gala compiler tools: gala_check, gala_build, gala_run, gala_test, gala_explain. ' +
          'gala_fix, gala_gir and gala_lsp_query are not available yet.',
      };
    }
    case 'ping':
      return {};
    case 'tools/list':
      return { tools: TOOL_DEFINITIONS.map((t) => ({ ...t })) };
    case 'tools/call': {
      const name = params.name;
      if (typeof name !== 'string' || !TOOL_DEFINITIONS.some((t) => t.name === name)) {
        throw new RpcError(-32602, `unknown tool: ${String(name)}`);
      }
      const args = isObject(params.arguments) ? params.arguments : {};
      const problems = validateArgs(name as ToolName, args);
      if (problems.length > 0) {
        return errorText(`invalid arguments for ${name}: ${problems.join('; ')}`);
      }
      try {
        return await callTool(name as ToolName, args);
      } catch (err) {
        return errorText(`${name} failed: ${err instanceof Error ? err.message : String(err)}`);
      }
    }
    case 'resources/list':
      return { resources: STATIC_RESOURCES };
    case 'resources/templates/list':
      return { resourceTemplates: RESOURCE_TEMPLATES };
    case 'resources/read': {
      if (typeof params.uri !== 'string') throw new RpcError(-32602, 'uri must be a string');
      return readResource(params.uri);
    }
    case 'prompts/list':
      return { prompts: PROMPTS };
    case 'prompts/get': {
      if (typeof params.name !== 'string') throw new RpcError(-32602, 'name must be a string');
      const rawArgs = isObject(params.arguments) ? params.arguments : {};
      const args: Record<string, string | undefined> = {};
      for (const [k, v] of Object.entries(rawArgs)) args[k] = typeof v === 'string' ? v : undefined;
      return getPrompt(params.name, args);
    }
    default:
      throw new RpcError(-32601, `method not found: ${method}`);
  }
}

function errorResponse(id: unknown, error: RpcError | Error): JsonObject {
  if (error instanceof RpcError) {
    return { jsonrpc: '2.0', id, error: { code: error.code, message: error.message, data: error.data } };
  }
  return { jsonrpc: '2.0', id, error: { code: -32603, message: error.message } };
}

async function handleSingle(msg: unknown): Promise<JsonObject | undefined> {
  if (!isObject(msg)) return errorResponse(null, new RpcError(-32600, 'Invalid Request'));
  const hasId = 'id' in msg;
  const id = msg.id ?? null;

  // Responses from the client (no method) are ignored; this server sends no requests.
  if (typeof msg.method !== 'string') {
    if ('result' in msg || 'error' in msg) return undefined;
    return hasId ? errorResponse(id, new RpcError(-32600, 'Invalid Request')) : undefined;
  }
  if (msg.jsonrpc !== '2.0') {
    return hasId ? errorResponse(id, new RpcError(-32600, 'Invalid Request')) : undefined;
  }

  try {
    const result = await dispatch(msg.method, isObject(msg.params) ? msg.params : {});
    if (!hasId) return undefined;
    return { jsonrpc: '2.0', id, result };
  } catch (err) {
    if (!hasId) return undefined;
    return errorResponse(id, err instanceof Error ? err : new Error(String(err)));
  }
}

/** Handles one JSON-RPC message or batch. Returns undefined when no response is due. */
export async function handleMessage(msg: unknown): Promise<unknown | undefined> {
  if (Array.isArray(msg)) {
    if (msg.length === 0) return errorResponse(null, new RpcError(-32600, 'Invalid Request'));
    const responses = (await Promise.all(msg.map(handleSingle))).filter((r) => r !== undefined);
    return responses.length > 0 ? responses : undefined;
  }
  return handleSingle(msg);
}

/** Serves MCP over newline-delimited JSON on stdin/stdout. */
export function serveStdio(input: NodeJS.ReadableStream = process.stdin, output: NodeJS.WritableStream = process.stdout): void {
  const write = (value: unknown) => {
    output.write(`${JSON.stringify(value)}\n`);
  };
  const rl = createInterface({ input, crlfDelay: Number.POSITIVE_INFINITY });
  rl.on('line', (line) => {
    if (line.trim().length === 0) return;
    let msg: unknown;
    try {
      msg = JSON.parse(line);
    } catch {
      write(errorResponse(null, new RpcError(-32700, 'Parse error')));
      return;
    }
    handleMessage(msg).then((response) => {
      if (response !== undefined) write(response);
    });
  });
}
