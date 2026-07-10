import { z } from 'zod';

export const DiagnosticSchema = z.object({
  code: z.string().regex(/^E[0-9]{4}$/),
  severity: z.enum(['error', 'warning', 'note', 'help']),
  message: z.string(),
  primary_span: z.object({
    file_id: z.number().int().nonnegative(),
    start: z.number().int().nonnegative(),
    end: z.number().int().nonnegative(),
  }),
  labels: z.array(
    z.object({
      file_id: z.number().int().nonnegative(),
      start: z.number().int().nonnegative(),
      end: z.number().int().nonnegative(),
      message: z.string(),
    })
  ),
  notes: z.array(z.string()),
  suggested_fixes: z.array(
    z.object({
      description: z.string(),
      edits: z.array(
        z.object({
          file_id: z.number().int().nonnegative(),
          start: z.number().int().nonnegative(),
          end: z.number().int().nonnegative(),
          new_text: z.string(),
        })
      ),
    })
  ),
  explain_url: z.string().url().optional(),
});

export type Diagnostic = z.infer<typeof DiagnosticSchema>;

export const GirSchema = z.object({
  version: z.literal(1),
  functions: z.array(
    z.object({
      id: z.string(),
      name: z.string(),
      params: z.array(
        z.object({
          name: z.string(),
          type: z.string(),
          linearity: z.enum(['linear', 'unrestricted']),
        })
      ),
      return_type: z.string(),
      effect: z.enum(['pure', 'quantum', 'prob']),
      body: z.unknown(),
      spans: z.object({
        file_id: z.number().int().nonnegative(),
        start: z.number().int().nonnegative(),
        end: z.number().int().nonnegative(),
      }),
      properties: z
        .object({
          is_unitary: z.boolean().optional(),
          is_reversible: z.boolean().optional(),
          has_adjoint: z.boolean().optional(),
          has_controlled: z.boolean().optional(),
        })
        .optional(),
    })
  ),
  types: z.record(
    z.object({
      kind: z.enum([
        'primitive',
        'qubit',
        'qubits',
        'measured',
        'function',
        'array',
        'tuple',
        'struct',
        'named',
      ]),
      name: z.string().optional(),
      params: z.array(z.string()).optional(),
      linearity: z.enum(['linear', 'unrestricted']).optional(),
    })
  ),
  effects: z.record(z.enum(['pure', 'quantum', 'prob'])),
  metadata: z
    .object({
      source_hash: z.string().optional(),
      compiler_version: z.string().optional(),
      timestamp: z.string().datetime().optional(),
    })
    .optional(),
});

export type Gir = z.infer<typeof GirSchema>;

export const PropertyResultSchema = z.object({
  property: z.enum(['unitary', 'reversible', 'uncomputes', 'grad_matches', 'effect_honesty']),
  passed: z.boolean(),
  details: z.object({
    function_name: z.string().optional(),
    num_trials: z.number().int().nonnegative().optional(),
    tolerance: z.number().optional(),
    max_error: z.number().optional(),
    counterexample: z.unknown().optional(),
    matrix_norm_diff: z.number().optional(),
    fidelity: z.number().optional(),
  }),
});

export type PropertyResult = z.infer<typeof PropertyResultSchema>;

export const CheckResultSchema = z.object({
  diagnostics: z.array(DiagnosticSchema),
  gir: GirSchema.optional(),
});

export type CheckResult = z.infer<typeof CheckResultSchema>;

export const RunResultSchema = z.object({
  results: z.array(
    z.object({
      name: z.string(),
      counts: z.record(z.number().int().nonnegative()),
      probabilities: z.record(z.number()),
    })
  ),
  circuit: z.string().optional(),
  shots: z.number().int().nonnegative(),
  seed: z.number().int().nonnegative().optional(),
});

export type RunResult = z.infer<typeof RunResultSchema>;

export const TestResultSchema = z.object({
  passed: z.boolean(),
  results: z.array(PropertyResultSchema),
  summary: z.object({
    total: z.number().int().nonnegative(),
    passed: z.number().int().nonnegative(),
    failed: z.number().int().nonnegative(),
  }),
});

export type TestResult = z.infer<typeof TestResultSchema>;

export const FixResultSchema = z.object({
  applied: z.array(
    z.object({
      description: z.string(),
      edits: z.array(
        z.object({
          file_id: z.number().int().nonnegative(),
          start: z.number().int().nonnegative(),
          end: z.number().int().nonnegative(),
          new_text: z.string(),
        })
      ),
    })
  ),
  remaining: z.array(DiagnosticSchema),
});

export type FixResult = z.infer<typeof FixResultSchema>;

export const ExplainResultSchema = z.object({
  code: z.string().regex(/^E[0-9]{4}$/),
  markdown: z.string(),
  examples: z
    .array(
      z.object({
        title: z.string(),
        bad: z.string(),
        good: z.string(),
      })
    )
    .optional(),
});

export type ExplainResult = z.infer<typeof ExplainResultSchema>;

export const GirResultSchema = z.object({
  gir: GirSchema,
  version: z.literal('1'),
});

export type GirResult = z.infer<typeof GirResultSchema>;

export const LspQueryInputSchema = z.object({
  source: z.string(),
  filePath: z.string(),
  position: z.object({
    line: z.number().int().nonnegative(),
    character: z.number().int().nonnegative(),
  }),
  queryType: z.enum(['hover', 'completion', 'definition', 'code_lens', 'gir_at_position']),
});

export type LspQueryInput = z.infer<typeof LspQueryInputSchema>;

export const LspQueryResultSchema = z.object({
  hover: z
    .object({
      type: z.string().optional(),
      effect: z.string().optional(),
      linearity: z.string().optional(),
      documentation: z.string().optional(),
      gir_node: z.unknown().optional(),
    })
    .optional(),
  completion: z
    .array(
      z.object({
        label: z.string(),
        kind: z.number().optional(),
        detail: z.string().optional(),
        insert_text: z.string().optional(),
      })
    )
    .optional(),
  definition: z
    .array(
      z.object({
        file_path: z.string(),
        range: z.object({
          start: z.object({ line: z.number(), character: z.number() }),
          end: z.object({ line: z.number(), character: z.number() }),
        }),
      })
    )
    .optional(),
  code_lens: z
    .array(
      z.object({
        range: z.object({
          start: z.object({ line: z.number(), character: z.number() }),
          end: z.object({ line: z.number(), character: z.number() }),
        }),
        command: z
          .object({
            title: z.string(),
            command: z.string(),
            arguments: z.array(z.unknown()).optional(),
          })
          .optional(),
      })
    )
    .optional(),
  gir_at_position: z.unknown().optional(),
});

export type LspQueryResult = z.infer<typeof LspQueryResultSchema>;
