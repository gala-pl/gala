const e = "http://json-schema.org/draft-07/schema#", t = "https://gala-lang.org/schemas/diagnostic.v1.json", r = "Gala Diagnostic", i = "1.0.0", s = "object", n = [
  "code",
  "severity",
  "message",
  "primary_span",
  "labels",
  "notes",
  "suggested_fixes"
], p = {
  code: {
    type: "string",
    pattern: "^E\\d{4}$"
  },
  severity: {
    type: "string",
    enum: [
      "error",
      "warning",
      "note",
      "help"
    ]
  },
  message: {
    type: "string"
  },
  primary_span: {
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
      },
      file_path: {
        type: "string"
      }
    }
  },
  labels: {
    type: "array",
    items: {
      type: "object",
      required: [
        "file_id",
        "start",
        "end",
        "message"
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
        },
        message: {
          type: "string"
        },
        file_path: {
          type: "string"
        }
      }
    }
  },
  notes: {
    type: "array",
    items: {
      type: "string"
    }
  },
  suggested_fixes: {
    type: "array",
    items: {
      type: "object",
      required: [
        "description",
        "edits"
      ],
      properties: {
        description: {
          type: "string"
        },
        edits: {
          type: "array",
          items: {
            type: "object",
            required: [
              "file_id",
              "start",
              "end",
              "new_text"
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
              },
              new_text: {
                type: "string"
              }
            }
          }
        }
      }
    }
  },
  explain_url: {
    type: "string",
    format: "uri"
  }
}, a = {
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
  a as default,
  p as properties,
  n as required,
  r as title,
  s as type,
  i as version
};
