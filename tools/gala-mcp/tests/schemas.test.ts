import { describe, expect, it } from 'vitest';
import {
  CheckResultSchema,
  DiagnosticSchema,
  ExplainResultSchema,
  FixResultSchema,
  GirResultSchema,
  GirSchema,
  PropertyResultSchema,
  RunResultSchema,
  TestResultSchema,
} from '../src/types.js';

describe('Type Schemas', () => {
  describe('DiagnosticSchema', () => {
    it('validates a valid diagnostic', () => {
      const diagnostic = {
        code: 'E0412',
        severity: 'error',
        message: 'qubit used after measurement',
        primary_span: {
          file_id: 0,
          start: 10,
          end: 15,
          file_path: 'test.gala',
        },
        labels: [
          {
            file_id: 0,
            start: 10,
            end: 15,
            message: 'consumed here',
            file_path: 'test.gala',
          },
        ],
        notes: ['a qubit is a linear resource'],
        suggested_fixes: [
          {
            description: 'Use the measurement result instead',
            edits: [{ file_id: 0, start: 10, end: 15, new_text: 'result' }],
          },
        ],
        explain_url: 'https://gala-lang.org/explain/E0412',
      };

      const result = DiagnosticSchema.safeParse(diagnostic);
      expect(result.success).toBe(true);
    });

    it('rejects invalid error code', () => {
      const diagnostic = {
        code: 'INVALID',
        severity: 'error',
        message: 'test',
        primary_span: { file_id: 0, start: 0, end: 1 },
        labels: [],
        notes: [],
        suggested_fixes: [],
      };

      const result = DiagnosticSchema.safeParse(diagnostic);
      expect(result.success).toBe(false);
    });

    it('rejects missing required fields', () => {
      const diagnostic = {
        code: 'E0412',
        severity: 'error',
      };

      const result = DiagnosticSchema.safeParse(diagnostic);
      expect(result.success).toBe(false);
    });
  });

  describe('GirSchema', () => {
    it('validates a valid GIR', () => {
      const gir = {
        version: 1,
        functions: [
          {
            id: 'fn1',
            name: 'bell',
            params: [{ name: 'q', type: 'Qubits<2>', linearity: 'linear' }],
            return_type: 'Qubits<2>',
            effect: 'quantum',
            body: {},
            spans: { file_id: 0, start: 0, end: 100 },
            properties: { is_unitary: true },
          },
        ],
        types: {
          'Qubits<2>': {
            kind: 'qubits',
            name: 'Qubits<2>',
            params: ['2'],
            linearity: 'linear',
          },
        },
        effects: { fn1: 'quantum' },
        metadata: {
          source_hash: 'abc123',
          compiler_version: '0.1.0',
          timestamp: '2024-01-01T00:00:00Z',
        },
      };

      const result = GirSchema.safeParse(gir);
      expect(result.success).toBe(true);
    });

    it('rejects invalid version', () => {
      const gir = {
        version: 2,
        functions: [],
        types: {},
        effects: {},
      };

      const result = GirSchema.safeParse(gir);
      expect(result.success).toBe(false);
    });
  });

  describe('PropertyResultSchema', () => {
    it('validates unitary property result', () => {
      const result = {
        property: 'unitary',
        passed: true,
        details: {
          function_name: 'qft4',
          num_trials: 100,
          tolerance: 1e-10,
          max_error: 5e-11,
          matrix_norm_diff: 1e-12,
          fidelity: 0.99999999999,
        },
        duration_ms: 45,
      };

      const parsed = PropertyResultSchema.safeParse(result);
      expect(parsed.success).toBe(true);
    });

    it('validates grad_matches property result', () => {
      const result = {
        property: 'grad_matches',
        passed: true,
        details: {
          function_name: 'ansatz',
          num_trials: 50,
          tolerance: 1e-6,
          max_error: 1e-7,
        },
        duration_ms: 120,
      };

      const parsed = PropertyResultSchema.safeParse(result);
      expect(parsed.success).toBe(true);
    });

    it('rejects unknown property', () => {
      const result = {
        property: 'unknown',
        passed: true,
        details: {},
        duration_ms: 10,
      };

      const parsed = PropertyResultSchema.safeParse(result);
      expect(parsed.success).toBe(false);
    });
  });

  describe('CheckResultSchema', () => {
    it('validates check result with diagnostics', () => {
      const result = {
        diagnostics: [
          {
            code: 'E0412',
            severity: 'error',
            message: 'qubit used after measurement',
            primary_span: { file_id: 0, start: 10, end: 15 },
            labels: [],
            notes: [],
            suggested_fixes: [],
          },
        ],
        gir: {
          version: 1,
          functions: [],
          types: {},
          effects: {},
        },
      };

      const parsed = CheckResultSchema.safeParse(result);
      expect(parsed.success).toBe(true);
    });
  });

  describe('RunResultSchema', () => {
    it('validates run result', () => {
      const result = {
        results: [
          {
            name: 'measure',
            counts: { '0': 512, '1': 512 },
            probabilities: { '0': 0.5, '1': 0.5 },
          },
        ],
        circuit: 'H(q[0]); CX(q[0], q[1]);',
        shots: 1024,
        seed: 42,
      };

      const parsed = RunResultSchema.safeParse(result);
      expect(parsed.success).toBe(true);
    });
  });

  describe('TestResultSchema', () => {
    it('validates test result with summary', () => {
      const result = {
        passed: true,
        results: [
          {
            property: 'unitary',
            passed: true,
            details: { function_name: 'qft4', num_trials: 100, tolerance: 1e-10, max_error: 1e-11 },
            duration_ms: 45,
          },
        ],
        summary: { total: 1, passed: 1, failed: 0 },
      };

      const parsed = TestResultSchema.safeParse(result);
      expect(parsed.success).toBe(true);
    });
  });

  describe('FixResultSchema', () => {
    it('validates fix result', () => {
      const result = {
        applied: [
          {
            description: 'Replace qubit with result',
            edits: [{ file_id: 0, start: 10, end: 15, new_text: 'result' }],
          },
        ],
        remaining: [],
      };

      const parsed = FixResultSchema.safeParse(result);
      expect(parsed.success).toBe(true);
    });
  });

  describe('ExplainResultSchema', () => {
    it('validates explain result', () => {
      const result = {
        code: 'E0412',
        markdown: '# E0412: Qubit used after measurement\n\n...',
        examples: [
          {
            title: 'Bad',
            bad: 'let q = qalloc(1); measure(q); h(q);',
            good: 'let q = qalloc(1); let r = measure(q);',
          },
        ],
      };

      const parsed = ExplainResultSchema.safeParse(result);
      expect(parsed.success).toBe(true);
    });
  });

  describe('GirResultSchema', () => {
    it('validates gir result', () => {
      const result = {
        gir: {
          version: 1,
          functions: [],
          types: {},
          effects: {},
        },
        version: '1',
      };

      const parsed = GirResultSchema.safeParse(result);
      expect(parsed.success).toBe(true);
    });
  });
});
