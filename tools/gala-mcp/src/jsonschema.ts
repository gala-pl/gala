// Minimal JSON Schema validator covering the subset used by the Gala schema files:
// type, required, properties, additionalProperties, items, enum, const, pattern,
// format (uri, date-time). Unknown keywords are ignored.

export type Schema = Record<string, unknown>;

export interface SchemaIssue {
  path: string;
  message: string;
}

const isObject = (v: unknown): v is Record<string, unknown> =>
  typeof v === 'object' && v !== null && !Array.isArray(v);

const hasOwn = (o: object, k: string): boolean => Object.prototype.hasOwnProperty.call(o, k);

function matchesType(value: unknown, type: string): boolean {
  switch (type) {
    case 'string':
      return typeof value === 'string';
    case 'boolean':
      return typeof value === 'boolean';
    case 'number':
      return typeof value === 'number' && Number.isFinite(value);
    case 'integer':
      return typeof value === 'number' && Number.isInteger(value);
    case 'object':
      return isObject(value);
    case 'array':
      return Array.isArray(value);
    case 'null':
      return value === null;
    default:
      return false;
  }
}

function describeType(value: unknown): string {
  if (value === null) return 'null';
  if (Array.isArray(value)) return 'array';
  if (typeof value === 'number') return Number.isInteger(value) ? 'integer' : 'number';
  return typeof value;
}

function isUri(value: string): boolean {
  try {
    return new URL(value).protocol.length > 0;
  } catch {
    return false;
  }
}

export function validate(value: unknown, schema: Schema, path = '$'): SchemaIssue[] {
  const issues: SchemaIssue[] = [];
  const push = (message: string, at: string = path) => issues.push({ path: at, message });

  if (schema.type !== undefined) {
    const types = Array.isArray(schema.type) ? (schema.type as string[]) : [schema.type as string];
    if (!types.some((t) => matchesType(value, t))) {
      push(`expected ${types.join(' | ')}, got ${describeType(value)}`);
      return issues;
    }
  }

  if (hasOwn(schema, 'const') && JSON.stringify(value) !== JSON.stringify(schema.const)) {
    push(`expected constant ${JSON.stringify(schema.const)}`);
  }

  if (Array.isArray(schema.enum) && !schema.enum.some((e) => JSON.stringify(e) === JSON.stringify(value))) {
    push(`value must be one of ${JSON.stringify(schema.enum)}`);
  }

  if (typeof value === 'string') {
    if (typeof schema.pattern === 'string' && !new RegExp(schema.pattern, 'u').test(value)) {
      push(`string does not match pattern ${schema.pattern}`);
    }
    if (schema.format === 'uri' && !isUri(value)) push('not a valid URI');
    if (schema.format === 'date-time' && Number.isNaN(Date.parse(value))) {
      push('not a valid date-time');
    }
  }

  if (isObject(value)) {
    const required = Array.isArray(schema.required) ? (schema.required as string[]) : [];
    for (const key of required) {
      if (!hasOwn(value, key)) push(`missing required property "${key}"`);
    }
    const props = isObject(schema.properties) ? schema.properties : {};
    for (const [key, sub] of Object.entries(props)) {
      if (hasOwn(value, key) && isObject(sub)) {
        issues.push(...validate(value[key], sub, `${path}.${key}`));
      }
    }
    if (schema.additionalProperties === false) {
      for (const key of Object.keys(value)) {
        if (!hasOwn(props, key)) push(`unexpected property "${key}"`);
      }
    } else if (isObject(schema.additionalProperties)) {
      for (const key of Object.keys(value)) {
        if (!hasOwn(props, key)) {
          issues.push(...validate(value[key], schema.additionalProperties, `${path}.${key}`));
        }
      }
    }
  }

  if (Array.isArray(value) && isObject(schema.items)) {
    value.forEach((item, i) => {
      issues.push(...validate(item, schema.items as Schema, `${path}[${i}]`));
    });
  }

  return issues;
}

export function formatIssues(issues: SchemaIssue[]): string {
  return issues.map((i) => `${i.path}: ${i.message}`).join('; ');
}
