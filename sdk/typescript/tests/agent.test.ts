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

  test('grad is unsupported and returns no number', async () => {
    const result = await agent.grad('f', 'theta');
    expect(result.supported).toBe(false);
    expect(result.reason.length).toBeGreaterThan(0);
  });

  test('verifyProperties is unsupported, never passed', async () => {
    const result = await agent.verifyProperties(read('tests/conformance/run/bell_run.gala'), [
      'unitary',
    ]);
    expect(result.status).toBe('unsupported');
    expect(result.requested).toEqual(['unitary']);
  });

  test('a missing binary is reported as a failed run, not a crash', async () => {
    const broken = new GalaAgent({ galaBin: '/nonexistent/gala' });
    const result = await broken.run('fn main() -> Int { 0 }');
    expect(result.ok).toBe(false);
    expect(result.error).toContain('could not run');
  });
});
