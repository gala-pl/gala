const e = "http://json-schema.org/draft-07/schema#", t = "https://gala-lang.org/schemas/property_result.v1.json", r = "Gala Property Test Result", s = "1.0.0", o = "object", p = [
  "property",
  "passed",
  "details"
], n = {
  property: {
    type: "string",
    enum: [
      "unitary",
      "reversible",
      "uncomputes",
      "grad_matches",
      "effect_honesty"
    ]
  },
  passed: {
    type: "boolean"
  },
  details: {
    type: "object",
    properties: {
      function_name: {
        type: "string"
      },
      num_trials: {
        type: "integer"
      },
      tolerance: {
        type: "number"
      },
      max_error: {
        type: "number"
      },
      counterexample: {
        type: "object"
      },
      matrix_norm_diff: {
        type: "number"
      },
      fidelity: {
        type: "number"
      }
    }
  },
  duration_ms: {
    type: "integer"
  }
}, a = {
  $schema: e,
  $id: t,
  title: r,
  version: s,
  type: o,
  required: p,
  properties: n
};
export {
  t as $id,
  e as $schema,
  a as default,
  n as properties,
  p as required,
  r as title,
  o as type,
  s as version
};
