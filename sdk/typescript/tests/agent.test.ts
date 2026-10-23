import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { GalaAgent } from '../src/index.ts';

const REPO = resolve(import.meta.dir, '../../..');
const read = (rel: string) => readFileSync(resolve(REPO, rel), 'utf8');

const agent = new GalaAgent();

describe('GalaAgent', () => {
  test('check reports E0412 for use_after_consume.gala', async () => {
    const result = await agent.check(read('tests/conformance/linearity/use_after_consume.gala'));
    expect(result.schema).toBe('gala.check.v1');
    expect(result.ok).toBe(false);
    expect(result.diagnostics.map((d) => d.code)).toContain('E0412');
  });

  test('run prints true for bell_run.gala', async () => {
    const result = await agent.run(read('tests/conformance/run/bell_run.gala'), { seed: 1 });
    expect(result.ok).toBe(true);
    expect(result.output[0]).toBe('true');
    expect(result.result).toBe('0');
  });

  test('grad returns the parameter-shift gradient from the CLI', async () => {
    const source = [
      'fn classify(x: Float, theta: Float) -> Measured<Bool> prob {',
      '    let q = qubit()',
      '    ry(q, x)',
      '    ry(q, theta)',
      '    measure(q)',
      '}',
      '',
      'fn main() -> Int { 0 }',
      '',
    ].join('\n');
    const result = await agent.grad(source, { fn: 'classify', wrt: 1, args: [0.4, 0.9], shots: 40000 });
    expect(result.supported).toBe(true);
    // Analytic value: -sin(0.4 + 0.9), about -0.9636.
    expect(Math.abs((result.value ?? 0) - -Math.sin(1.3))).toBeLessThan(0.05);
    expect(result.occurrences).toBe(1);
  });

  test('verifyProperties runs runnable properties and reports the rest as unsupported', async () => {
    const source = [
      '#[property(effect_honesty, unitary)]',
      'fn prep() -> Qubit quantum {',
      '    qubit()',
      '}',
      '',
      'fn main() -> Int { 0 }',
      '',
    ].join('\n');
    const result = await agent.verifyProperties(source, ['effect_honesty', 'unitary']);
    expect(result.results.map((r) => r.property)).toEqual(['effect_honesty']);
    expect(result.results[0]?.passed).toBe(true);
    expect(result.skipped.map((s) => s.property)).toEqual(['unitary']);
    expect(result.status).toBe('unsupported');
  });

  test('verifyProperties reports a source that does not compile as error', async () => {
    const result = await agent.verifyProperties('fn main() -> Int { missing }', ['effect_honesty']);
    expect(result.status).toBe('error');
    expect(result.diagnostics.length).toBeGreaterThan(0);
  });

  test('a missing binary is reported as a failed run, not a crash', async () => {
    const broken = new GalaAgent({ galaBin: '/nonexistent/gala' });
    const result = await broken.run('fn main() -> Int { 0 }');
    expect(result.ok).toBe(false);
    expect(result.error).toContain('could not run');
  });
});
