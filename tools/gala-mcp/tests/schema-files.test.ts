import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import { describe, expect, it } from 'vitest';

describe('JSON Schema Files', () => {
  const ajv = new Ajv({ strict: false });
  addFormats(ajv);

  const schemasDir = resolve(__dirname, '../dist');

  const schemaFiles = ['diagnostic.v1.json', 'gir.v1.json', 'property_result.v1.json'];

  for (const file of schemaFiles) {
    it(`loads and validates ${file}`, () => {
      const schemaPath = resolve(schemasDir, file);
      const schema = JSON.parse(readFileSync(schemaPath, 'utf-8'));

      // Validate it's a valid JSON Schema
      expect(schema.$schema).toBeDefined();
      expect(schema.$id).toBeDefined();
      expect(schema.title).toBeDefined();
      expect(schema.type).toBe('object');

      // Compile with AJV to ensure it's valid
      const validate = ajv.compile(schema);
      expect(validate).toBeDefined();
    });
  }
});
