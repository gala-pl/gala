//! Long-form explanations for diagnostic codes (`gala explain <code>`).

use crate::DiagnosticCode;
use serde_json::{json, Value};

/// One registry entry: a short title, the long explanation, and a worked example.
#[derive(Debug, Clone, Copy)]
pub struct CodeInfo {
    /// Numeric part of the code (`412` for `E0412`).
    pub code: u16,
    /// Short title, used as the heading.
    pub title: &'static str,
    /// Why the rule exists, in physical terms where the rule encodes physics.
    pub body: &'static str,
    /// A program that triggers the diagnostic.
    pub bad_example: &'static str,
    /// The corrected program.
    pub good_example: &'static str,
}

/// Every code the toolchain can emit.
pub const REGISTRY: &[CodeInfo] = &[
    CodeInfo {
        code: 101,
        title: "unexpected token",
        body: "The parser expected one kind of token and found another. Check for a missing \
               closing parenthesis, brace, or comma, or a missing `fn` keyword.",
        bad_example: "fn f() -> Int { (1 + 2 }",
        good_example: "fn f() -> Int { (1 + 2) }",
    },
    CodeInfo {
        code: 102,
        title: "unterminated string literal",
        body: "A string literal was opened with `\"` and never closed on the same source.",
        bad_example: "fn f() { print(\"hello) }",
        good_example: "fn f() { print(\"hello\") }",
    },
    CodeInfo {
        code: 103,
        title: "unterminated block comment",
        body: "A `/*` comment was not closed. Block comments nest, so every `/*` needs its `*/`.",
        bad_example: "/* outer /* inner */",
        good_example: "/* outer /* inner */ */",
    },
    CodeInfo {
        code: 104,
        title: "invalid numeric literal",
        body: "The digits do not form a valid integer or float literal.",
        bad_example: "fn f() -> Int { 12abc }",
        good_example: "fn f() -> Int { 12 }",
    },
    CodeInfo {
        code: 105,
        title: "invalid character",
        body: "The character is not part of Gala's lexical grammar.",
        bad_example: "fn f() -> Int { 1 @ 2 }",
        good_example: "fn f() -> Int { 1 + 2 }",
    },
    CodeInfo {
        code: 106,
        title: "expected expression",
        body: "An operator or keyword was found where an expression must start.",
        bad_example: "fn f() -> Int { let x = ; 1 }",
        good_example: "fn f() -> Int { let x = 1; x }",
    },
    CodeInfo {
        code: 107,
        title: "expected statement",
        body: "A statement was expected inside a block.",
        bad_example: "fn f() { = 1 }",
        good_example: "fn f() { let x = 1 }",
    },
    CodeInfo {
        code: 108,
        title: "expected type",
        body: "A type annotation was required here, for example after `->` or `:`.",
        bad_example: "fn f() -> { 1 }",
        good_example: "fn f() -> Int { 1 }",
    },
    CodeInfo {
        code: 109,
        title: "expected identifier",
        body: "A name was required, for example for a function, parameter, or binding.",
        bad_example: "fn (x: Int) -> Int { x }",
        good_example: "fn id(x: Int) -> Int { x }",
    },
    CodeInfo {
        code: 110,
        title: "unexpected end of file",
        body: "The file ended before a construct was closed.",
        bad_example: "fn f() -> Int { 1",
        good_example: "fn f() -> Int { 1 }",
    },
    CodeInfo {
        code: 201,
        title: "type mismatch",
        body: "Two types that must agree do not. Gala never converts silently between `Int`, \
               `Float`, `Bool`, or qubit types.",
        bad_example: "fn f() -> Int { true }",
        good_example: "fn f() -> Bool { true }",
    },
    CodeInfo {
        code: 202,
        title: "unresolved name",
        body: "The name is not a local binding, a function in this package, an imported item, or \
               a builtin. Check spelling, or add an `import`.",
        bad_example: "fn f() -> Int { missing + 1 }",
        good_example: "fn f() -> Int { let missing = 1; missing + 1 }",
    },
    CodeInfo {
        code: 203,
        title: "assignment to an immutable binding",
        body: "Only bindings declared with `let mut` can be assigned after initialization.",
        bad_example: "fn f() -> Int { let x = 1\n x = 2\n x }",
        good_example: "fn f() -> Int { let mut x = 1\n x = 2\n x }",
    },
    CodeInfo {
        code: 207,
        title: "feature not supported yet",
        body: "The construct is part of the Gala language design but this compiler does not \
               implement it yet. The message names the construct.",
        bad_example: "fn f() -> Int { match 1 { _ => 0 } }",
        good_example: "fn f() -> Int { if 1 == 1 { 0 } else { 1 } }",
    },
    CodeInfo {
        code: 204,
        title: "wrong number of arguments",
        body: "A call supplied a different number of arguments than the callee declares.",
        bad_example: "fn g(a: Int) -> Int { a }\nfn f() -> Int { g(1, 2) }",
        good_example: "fn g(a: Int) -> Int { a }\nfn f() -> Int { g(1) }",
    },
    CodeInfo {
        code: 205,
        title: "const generic mismatch",
        body: "A const generic argument does not match the expected value.",
        bad_example: "fn f() -> Qubits<2> quantum { qubits<3>() }",
        good_example: "fn f() -> Qubits<3> quantum { qubits<3>() }",
    },
    CodeInfo {
        code: 206,
        title: "register size mismatch",
        body: "A `Qubits<N>` register was used where a register of a different size is required.",
        bad_example: "fn f(q: Qubits<2>) -> Qubits<3> quantum { q }",
        good_example: "fn f(q: Qubits<3>) -> Qubits<3> quantum { q }",
    },
    CodeInfo {
        code: 301,
        title: "declared effect is too weak",
        body: "The body performs an operation that its declared effect does not allow. Effects \
               form the lattice `pure ⊑ quantum ⊑ prob`: a function may call only functions whose \
               effect is at or below its own.",
        bad_example: "fn f() -> Int pure { let q = qubit(); 1 }",
        good_example: "fn f() -> Int quantum { let q = qubit(); drop(q); 1 }",
    },
    CodeInfo {
        code: 302,
        title: "pure function calls a quantum function",
        body: "A `pure` function cannot allocate, transform, or measure qubits. Qubit operations \
               are unitary and reversible, so they belong to a `quantum` function.",
        bad_example: "fn f(q: Qubit) -> Qubit pure { h(q) }",
        good_example: "fn f(q: Qubit) -> Qubit quantum { h(q) }",
    },
    CodeInfo {
        code: 303,
        title: "quantum function calls a probabilistic function",
        body: "Calling a `prob` function from a `quantum` context crosses the measurement \
               boundary. A quantum function is guaranteed measurement-free and therefore \
               reversible. Declare the caller `prob` to acknowledge the boundary.",
        bad_example: "fn m(q: Qubit) -> Measured<Bool> prob { measure(q) }\n\
                      fn f(q: Qubit) -> Measured<Bool> quantum { m(q) }",
        good_example: "fn m(q: Qubit) -> Measured<Bool> prob { measure(q) }\n\
                       fn f(q: Qubit) -> Measured<Bool> prob { m(q) }",
    },
    CodeInfo {
        code: 304,
        title: "measurement in a non-probabilistic function",
        body: "`measure` destroys a qubit and returns a classical result. The effect is `prob`, so \
               only a function declared `prob` may measure.",
        bad_example: "fn f() -> Measured<Bool> quantum { let q = qubit(); measure(q) }",
        good_example: "fn f() -> Measured<Bool> prob { let q = qubit(); measure(q) }",
    },
    CodeInfo {
        code: 401,
        title: "linear value duplicated (no-cloning)",
        body: "A qubit cannot be copied. The no-cloning theorem says an arbitrary unknown quantum \
               state cannot be duplicated, so Gala's type system makes qubits linear: each one is \
               used exactly once.",
        bad_example: "fn f(q: Qubit) -> (Qubit, Qubit) quantum { clone(q) }",
        good_example: "fn f() -> Qubit quantum { qubit() }",
    },
    CodeInfo {
        code: 402,
        title: "linear value dropped implicitly",
        body: "A qubit went out of scope without being consumed. Discarding an entangled qubit \
               corrupts the computation, so every qubit must be measured, returned, or released \
               with `drop(q)`, which synthesizes the uncomputation.",
        bad_example: "fn f() quantum { let q = qubit() }",
        good_example: "fn f() quantum { let q = qubit(); drop(q) }",
    },
    CodeInfo {
        code: 403,
        title: "the same qubit is passed twice",
        body: "One call received the same qubit in two argument positions. Two references to one \
               qubit alias it, which the linear discipline forbids.",
        bad_example: "fn f() quantum { let a = qubit(); cx(a, a) }",
        good_example: "fn f() quantum { let a = qubit(); let b = qubit(); cx(a, b) }",
    },
    CodeInfo {
        code: 405,
        title: "linear value consumed on only some branches",
        body: "A qubit is consumed in one branch of an `if` but not the other. After the join it \
               is either live or gone depending on the path, so the checker cannot tell which.",
        bad_example: "fn f(c: Bool) quantum {\n  let q = qubit()\n  if c { drop(q) }\n}",
        good_example: "fn f(c: Bool) quantum {\n  let q = qubit()\n  if c { drop(q) } else { drop(q) }\n}",
    },
    CodeInfo {
        code: 412,
        title: "qubit used after it was consumed",
        body: "A qubit is a linear resource. Measuring it, passing it to a consuming operation, or \
               dropping it ends its life. Any later use refers to a qubit that no longer exists. \
               If you need the value afterward, use the classical result instead.",
        bad_example: "fn f() -> Measured<Bool> prob {\n  let a = qubit()\n  let r = measure(a)\n  h(a)\n  r\n}",
        good_example: "fn f() -> Measured<Bool> prob {\n  let a = qubit()\n  h(a)\n  measure(a)\n}",
    },
    CodeInfo {
        code: 530,
        title: "cannot uncompute automatically",
        body: "An ancilla is released with `drop`, which must return it to |0⟩ before release. \
               Gala synthesizes the adjoint of the circuit that produced the ancilla. It refuses \
               when that circuit depends on a measured value (irreversible), because the adjoint \
               would need the measurement outcome, which no longer exists.",
        bad_example: "fn f(d: Qubit) -> Qubit prob {\n  let a = qubit()\n  cx(d, a)\n  measure(d)\n  drop(a)\n  d\n}",
        good_example: "fn f(d: Qubit) -> Qubit quantum {\n  let a = qubit()\n  cx(d, a)\n  cx(d, a)\n  drop(a)\n  d\n}",
    },
    CodeInfo {
        code: 601,
        title: "backend capability mismatch",
        body: "The program needs a capability the selected backend does not declare, such as \
               mid-circuit measurement on a base-profile target or more qubits than the device has.",
        bad_example: "fn main() { run(f, on: backend.hardware(\"small\")) }",
        good_example: "fn main() { run(f, on: backend.simulator()) }",
    },
    CodeInfo {
        code: 602,
        title: "QIR emission failed",
        body: "The GIR could not be lowered to QIR. The message names the unsupported operation.",
        bad_example: "gala build --emit qir unsupported.gala",
        good_example: "gala build --emit qir supported.gala",
    },
    CodeInfo {
        code: 603,
        title: "hardware unavailable",
        body: "The requested hardware backend could not be reached or has no credentials.",
        bad_example: "gala run --backend ionq.aria program.gala  (without credentials)",
        good_example: "gala run program.gala  (uses the built-in simulator)",
    },
    CodeInfo {
        code: 701,
        title: "runtime error",
        body: "The program failed while executing on the simulator, for example a register index \
               was out of range.",
        bad_example: "fn main() -> Int { let q = qubits<2>(); h(q[5]); 0 }",
        good_example: "fn main() -> Int { let q = qubits<2>(); h(q[1]); 0 }",
    },
    CodeInfo {
        code: 702,
        title: "simulator capacity exceeded",
        body: "The state vector would need more qubits than the simulator will allocate. The \
               state vector doubles with every qubit.",
        bad_example: "fn main() -> Int { let q = qubits<40>(); 0 }",
        good_example: "fn main() -> Int { let q = qubits<4>(); 0 }",
    },
];

/// Look up the explanation for a code.
pub fn lookup(code: DiagnosticCode) -> Option<&'static CodeInfo> {
    REGISTRY.iter().find(|info| info.code == code.0)
}

/// Render the explanation as plain text.
pub fn render_text(code: DiagnosticCode) -> Option<String> {
    let info = lookup(code)?;
    Some(format!(
        "{}: {}\n\n{}\n\nExample that triggers it:\n    {}\n\nFixed:\n    {}\n",
        code, info.title, info.body, info.bad_example, info.good_example
    ))
}

/// Render the explanation as Markdown.
pub fn render_markdown(code: DiagnosticCode) -> Option<String> {
    let info = lookup(code)?;
    Some(format!(
        "# {code}: {}\n\n{}\n\n## Example that triggers it\n\n```gala\n{}\n```\n\n## Fixed\n\n```gala\n{}\n```\n",
        info.title, info.body, info.bad_example, info.good_example
    ))
}

/// Render the explanation as JSON (`{code, title, explanation, examples}`).
pub fn render_json(code: DiagnosticCode) -> Option<Value> {
    let info = lookup(code)?;
    Some(json!({
        "code": code.to_string(),
        "title": info.title,
        "explanation": info.body,
        "examples": {
            "bad": info.bad_example,
            "good": info.good_example,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explain_known_code_mentions_title() {
        let text = render_text(DiagnosticCode(412)).expect("E0412 is registered");
        assert!(text.starts_with("E0412: qubit used after it was consumed"));
        assert!(text.contains("linear resource"));
    }

    #[test]
    fn explain_unknown_code_is_none() {
        assert!(render_text(DiagnosticCode(9999)).is_none());
        assert!(render_markdown(DiagnosticCode(9999)).is_none());
        assert!(render_json(DiagnosticCode(9999)).is_none());
    }

    #[test]
    fn markdown_and_json_render_all_registered_codes() {
        for info in REGISTRY {
            let code = DiagnosticCode(info.code);
            let md = render_markdown(code).expect("markdown");
            assert!(md.starts_with(&format!("# {code}")));
            let js = render_json(code).expect("json");
            assert_eq!(js["code"], code.to_string());
            assert!(js["examples"]["bad"].is_string());
            assert!(js["examples"]["good"].is_string());
        }
    }

    #[test]
    fn registry_codes_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for info in REGISTRY {
            assert!(seen.insert(info.code), "duplicate registry code {}", info.code);
        }
    }
}
