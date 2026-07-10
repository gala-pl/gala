const e = "http://json-schema.org/draft-07/schema#", t = "https://gala-lang.org/schemas/gir.v1.json", r = "Gala IR (GIR) v1", i = "1.0.0", s = "object", n = [
  "version",
  "functions",
  "types",
  "effects"
], p = {
  version: {
    type: "integer",
    const: 1
  },
  functions: {
    type: "array",
    items: {
      type: "object",
      required: [
        "id",
        "name",
        "params",
        "return_type",
        "effect",
        "body",
        "spans"
      ],
      properties: {
        id: {
          type: "string"
        },
        name: {
          type: "string"
        },
        params: {
          type: "array",
          items: {
            type: "object",
            required: [
              "name",
              "type",
              "linearity"
            ],
            properties: {
              name: {
                type: "string"
              },
              type: {
                type: "string"
              },
              linearity: {
                type: "string",
                enum: [
                  "linear",
                  "unrestricted"
                ]
              }
            }
          }
        },
        return_type: {
          type: "string"
        },
        effect: {
          type: "string",
          enum: [
            "pure",
            "quantum",
            "prob"
          ]
        },
        body: {
          type: "object",
          description: "GIR expression node"
        },
        spans: {
          type: "object",
          required: [
            "file_id",
            "start",
            "end"
          ],
          properties: {
            file_id: {
              type: "integer"
            },
            start: {
              type: "integer"
            },
            end: {
              type: "integer"
            }
          }
        },
        properties: {
          type: "object",
          properties: {
            is_unitary: {
              type: "boolean"
            },
            is_reversible: {
              type: "boolean"
            },
            has_adjoint: {
              type: "boolean"
            },
            has_controlled: {
              type: "boolean"
            }
          }
        }
      }
    }
  },
  types: {
    type: "object",
    additionalProperties: {
      type: "object",
      required: [
        "kind"
      ],
      properties: {
        kind: {
          type: "string",
          enum: [
            "primitive",
            "qubit",
            "qubits",
            "measured",
            "function",
            "array",
            "tuple",
            "struct",
            "named"
          ]
        },
        name: {
          type: "string"
        },
        params: {
          type: "array",
          items: {
            type: "string"
          }
        },
        linearity: {
          type: "string",
          enum: [
            "linear",
            "unrestricted"
          ]
        }
      }
    }
  },
  effects: {
    type: "object",
    additionalProperties: {
      type: "string",
      enum: [
        "pure",
        "quantum",
        "prob"
      ]
    }
  },
  metadata: {
    type: "object",
    properties: {
      source_hash: {
        type: "string"
      },
      compiler_version: {
        type: "string"
      },
      timestamp: {
        type: "string",
        format: "date-time"
      }
    }
  }
}, o = {
  $schema: e,
  $id: t,
  title: r,
  version: i,
  type: s,
  required: n,
  properties: p
};
export {
  t as $id,
  e as $schema,
  o as default,
  p as properties,
  n as required,
  r as title,
  s as type,
  i as version
};
