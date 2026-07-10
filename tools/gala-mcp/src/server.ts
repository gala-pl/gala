import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  GetPromptRequestSchema,
  ListPromptsRequestSchema,
  ListResourcesRequestSchema,
  ListToolsRequestSchema,
  ReadResourceRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import { z } from 'zod';

import {
  galaBuild,
  galaCheck,
  galaExplain,
  galaFix,
  galaGir,
  galaLspQuery,
  galaRun,
  galaTest,
} from './gala-cli.js';

const server = new Server(
  { name: 'gala-mcp', version: '0.1.0' },
  { capabilities: { tools: {}, resources: {}, prompts: {} } }
);

const ToolInputSchemas = {
  gala_check: z.object({
    source: z.string(),
    filePath: z.string().optional(),
  }),
  gala_build: z.object({
    source: z.string(),
    emitGir: z.boolean().optional(),
    girVersion: z.number().int().positive().optional(),
  }),
  gala_run: z.object({
    source: z.string(),
    shots: z.number().int().positive().optional(),
    seed: z.number().int().nonnegative().optional(),
  }),
  gala_test: z.object({
    source: z.string(),
    properties: z
      .array(z.enum(['unitary', 'reversible', 'uncomputes', 'grad_matches', 'effect_honesty']))
      .optional(),
  }),
  gala_fix: z.object({
    source: z.string(),
  }),
  gala_explain: z.object({
    code: z.string().regex(/^E[0-9]{4}$/),
  }),
  gala_gir: z.object({
    source: z.string(),
  }),
  gala_lsp_query: z.object({
    source: z.string(),
    filePath: z.string(),
    position: z.object({
      line: z.number().int().nonnegative(),
      character: z.number().int().nonnegative(),
    }),
    queryType: z.enum(['hover', 'completion', 'definition', 'code_lens', 'gir_at_position']),
  }),
} as const;

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: 'gala_check',
      description:
        'Type-check Gala source code and return structured diagnostics with error codes, spans, and suggested fixes',
      inputSchema: {
        type: 'object',
        properties: {
          source: { type: 'string', description: 'Gala source code to check' },
          file_path: { type: 'string', description: 'Optional file path for error reporting' },
        },
        required: ['source'],
      },
    },
    {
      name: 'gala_build',
      description: 'Build Gala source and optionally emit GIR (Gala Intermediate Representation)',
      inputSchema: {
        type: 'object',
        properties: {
          source: { type: 'string', description: 'Gala source code' },
          emit_gir: { type: 'boolean', description: 'Whether to emit GIR JSON' },
          gir_version: { type: 'number', description: 'GIR schema version (default: 1)' },
        },
        required: ['source'],
      },
    },
    {
      name: 'gala_run',
      description: 'Execute Gala program on the built-in quantum simulator',
      inputSchema: {
        type: 'object',
        properties: {
          source: { type: 'string', description: 'Gala source code' },
          shots: { type: 'number', description: 'Number of measurement shots (default: 1024)' },
          seed: { type: 'number', description: 'Random seed for reproducibility' },
        },
        required: ['source'],
      },
    },
    {
      name: 'gala_test',
      description:
        'Run quantum property tests (unitarity, reversibility, uncomputation, gradient correctness)',
      inputSchema: {
        type: 'object',
        properties: {
          source: {
            type: 'string',
            description: 'Gala source code with #[property(...)] attributes',
          },
          properties: {
            type: 'array',
            items: {
              type: 'string',
              enum: ['unitary', 'reversible', 'uncomputes', 'grad_matches', 'effect_honesty'],
            },
            description: 'Specific properties to test (default: all)',
          },
        },
        required: ['source'],
      },
    },
    {
      name: 'gala_fix',
      description: 'Apply suggested fixes for diagnostics in Gala source code',
      inputSchema: {
        type: 'object',
        properties: {
          source: { type: 'string', description: 'Gala source code to fix' },
        },
        required: ['source'],
      },
    },
    {
      name: 'gala_explain',
      description: 'Get detailed explanation of a Gala diagnostic error code',
      inputSchema: {
        type: 'object',
        properties: {
          code: { type: 'string', pattern: '^E[0-9]{4}$', description: 'Error code (e.g., E0412)' },
        },
        required: ['code'],
      },
    },
    {
      name: 'gala_gir',
      description: 'Get Gala IR (GIR) for semantic analysis of quantum/classical code',
      inputSchema: {
        type: 'object',
        properties: {
          source: { type: 'string', description: 'Gala source code' },
        },
        required: ['source'],
      },
    },
    {
      name: 'gala_lsp_query',
      description:
        'Query the Gala Language Server for semantic information (hover, completion, GIR at position)',
      inputSchema: {
        type: 'object',
        properties: {
          source: { type: 'string', description: 'Gala source code' },
          file_path: { type: 'string', description: 'File path for LSP context' },
          position: {
            type: 'object',
            properties: {
              line: { type: 'number', minimum: 0 },
              character: { type: 'number', minimum: 0 },
            },
            required: ['line', 'character'],
          },
          query_type: {
            type: 'string',
            enum: ['hover', 'completion', 'definition', 'code_lens', 'gir_at_position'],
          },
        },
        required: ['source', 'file_path', 'position', 'query_type'],
      },
    },
  ],
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    let result: unknown;

    switch (name) {
      case 'gala_check': {
        const input = ToolInputSchemas.gala_check.parse(args);
        result = await galaCheck(input.source, input.filePath);
        break;
      }
      case 'gala_build': {
        const input = ToolInputSchemas.gala_build.parse(args);
        result = await galaBuild(input.source, {
          emitGir: input.emitGir,
          girVersion: input.girVersion,
        });
        break;
      }
      case 'gala_run': {
        const input = ToolInputSchemas.gala_run.parse(args);
        result = await galaRun(input.source, { shots: input.shots, seed: input.seed });
        break;
      }
      case 'gala_test': {
        const input = ToolInputSchemas.gala_test.parse(args);
        result = await galaTest(input.source, input.properties);
        break;
      }
      case 'gala_fix': {
        const input = ToolInputSchemas.gala_fix.parse(args);
        result = await galaFix(input.source);
        break;
      }
      case 'gala_explain': {
        const input = ToolInputSchemas.gala_explain.parse(args);
        result = await galaExplain(input.code);
        break;
      }
      case 'gala_gir': {
        const input = ToolInputSchemas.gala_gir.parse(args);
        result = await galaGir(input.source);
        break;
      }
      case 'gala_lsp_query': {
        const input = ToolInputSchemas.gala_lsp_query.parse(args);
        result = await galaLspQuery(input);
        break;
      }
      default:
        throw new Error(`Unknown tool: ${name}`);
    }

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(result, null, 2),
        },
      ],
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify({ error: message }, null, 2),
        },
      ],
      isError: true,
    };
  }
});

const _ResourceSchemas = {
  'gala://diagnostics/{file}': z.object({
    file: z.string(),
  }),
  'gala://gir/{file}': z.object({
    file: z.string(),
  }),
  'gala://schema/diagnostic.v1': z.object({}),
  'gala://schema/gir.v1': z.object({}),
};

server.setRequestHandler(ListResourcesRequestSchema, async () => ({
  resources: [
    {
      uri: 'gala://schema/diagnostic.v1',
      name: 'Diagnostic JSON Schema v1',
      mimeType: 'application/json',
      description: 'JSON schema for Gala structured diagnostics',
    },
    {
      uri: 'gala://schema/gir.v1',
      name: 'GIR JSON Schema v1',
      mimeType: 'application/json',
      description: 'JSON schema for Gala Intermediate Representation',
    },
    {
      uri: 'gala://schema/property_result.v1',
      name: 'Property Test Result JSON Schema v1',
      mimeType: 'application/json',
      description: 'JSON schema for quantum property test results',
    },
  ],
}));

server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const { uri } = request.params;

  if (uri === 'gala://schema/diagnostic.v1') {
    const schema = await import('./schemas/diagnostic.v1.json', { with: { type: 'json' } });
    return {
      contents: [
        { uri, mimeType: 'application/json', text: JSON.stringify(schema.default, null, 2) },
      ],
    };
  }
  if (uri === 'gala://schema/gir.v1') {
    const schema = await import('./schemas/gir.v1.json', { with: { type: 'json' } });
    return {
      contents: [
        { uri, mimeType: 'application/json', text: JSON.stringify(schema.default, null, 2) },
      ],
    };
  }
  if (uri === 'gala://schema/property_result.v1') {
    const schema = await import('./schemas/property_result.v1.json', { with: { type: 'json' } });
    return {
      contents: [
        { uri, mimeType: 'application/json', text: JSON.stringify(schema.default, null, 2) },
      ],
    };
  }

  throw new Error(`Resource not found: ${uri}`);
});

server.setRequestHandler(ListPromptsRequestSchema, async () => ({
  prompts: [
    {
      name: 'gala/debug-error',
      description: 'Given a Gala diagnostic, explain the quantum physics and suggest fixes',
      arguments: [
        {
          name: 'diagnostic',
          description: 'JSON diagnostic object from gala_check',
          required: true,
        },
      ],
    },
    {
      name: 'gala/write-quantum-fn',
      description: 'Write a Gala quantum function implementing a specific algorithm',
      arguments: [
        {
          name: 'task',
          description:
            "Description of the quantum algorithm (e.g., 'Bell pair preparation', 'QFT on 4 qubits')",
          required: true,
        },
        {
          name: 'constraints',
          description:
            "Optional constraints (e.g., 'use only H, CX, Rz gates', 'no mid-circuit measurement')",
          required: false,
        },
      ],
    },
    {
      name: 'gala/optimize-circuit',
      description: 'Optimize a Gala quantum function for a specific backend',
      arguments: [
        {
          name: 'source',
          description: 'Gala source code of the quantum function',
          required: true,
        },
        {
          name: 'backend',
          description: "Target backend (e.g., 'ibm_brisbane', 'ionq_aria', 'simulator')",
          required: true,
        },
        {
          name: 'objectives',
          description:
            "Optimization objectives (e.g., 'minimize depth', 'minimize 2q gates', 'maximize fidelity')",
          required: false,
        },
      ],
    },
  ],
}));

server.setRequestHandler(GetPromptRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  switch (name) {
    case 'gala/debug-error': {
      const diagnostic = args?.diagnostic;
      return {
        description: 'Debug a Gala diagnostic error',
        messages: [
          {
            role: 'user',
            content: {
              type: 'text',
              text: `Analyze this Gala diagnostic and explain the quantum physics behind the error, then suggest fixes:\n\n${JSON.stringify(diagnostic, null, 2)}`,
            },
          },
        ],
      };
    }
    case 'gala/write-quantum-fn': {
      const task = args?.task;
      const constraints = args?.constraints ?? '';
      return {
        description: 'Write a Gala quantum function',
        messages: [
          {
            role: 'user',
            content: {
              type: 'text',
              text: `Write a Gala quantum function that implements: ${task}\n\nConstraints: ${constraints}\n\nRequirements:\n- Use proper Gala syntax with effect annotations (quantum/prob/pure)\n- Include linear type annotations for qubits\n- Add #[property(unitary)] for verification\n- Use standard library gates from gala.gates`,
            },
          },
        ],
      };
    }
    case 'gala/optimize-circuit': {
      const source = args?.source;
      const backend = args?.backend;
      const objectives = args?.objectives ?? 'minimize depth and 2-qubit gate count';
      return {
        description: 'Optimize a Gala quantum circuit for a backend',
        messages: [
          {
            role: 'user',
            content: {
              type: 'text',
              text: `Optimize this Gala quantum function for ${backend}:\n\n${source}\n\nObjectives: ${objectives}\n\nProvide optimized Gala code with #[property(unitary)] and explain the optimizations applied.`,
            },
          },
        ],
      };
    }
    default:
      throw new Error(`Unknown prompt: ${name}`);
  }
});

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch((_err) => {
  process.exit(1);
});
