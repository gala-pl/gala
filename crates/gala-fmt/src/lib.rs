//! Canonical formatter for Gala.

use gala_ast::{Expr, Item, Stmt, Type};
use gala_lexer::Lexer;
use gala_parser::parse;
use gala_span::SourceMap;

/// Formats Gala source code.
pub fn format_source(source: &str) -> String {
    let mut map = SourceMap::new();
    let fid = map.add_file("<fmt>".into(), source.to_string());
    let lexer = Lexer::new(fid, source);
    let tokens = lexer.collect_all();

    match parse(&tokens) {
        Ok(items) => format_items(&items),
        Err(_) => source.to_string(),
    }
}

/// Check if source is already formatted.
pub fn is_formatted(source: &str) -> bool {
    format_source(source) == source
}

fn format_items(items: &[Item]) -> String {
    let mut out = String::new();
    for item in items {
        format_item(item, &mut out, 0);
        out.push('\n');
    }
    out
}

fn pattern_to_string(pat: &gala_ast::Pattern) -> String {
    match pat {
        gala_ast::Pattern::Ident(i) => i.0.clone(),
        gala_ast::Pattern::Wildcard => "_".to_string(),
        gala_ast::Pattern::Tuple(elems) => {
            let inner = elems.iter().map(pattern_to_string).collect::<Vec<_>>().join(", ");
            format!("({inner})")
        }
        gala_ast::Pattern::Struct { path, fields } => {
            let inner = fields
                .iter()
                .map(|(name, fp)| format!("{}: {}", name.0, pattern_to_string(fp)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} {{ {} }}", path_to_string(path), inner)
        }
        gala_ast::Pattern::Literal(lit) => literal_to_string(lit),
    }
}

fn format_item(item: &Item, out: &mut String, indent: usize) {
    let pad = "    ".repeat(indent);
    match item {
        gala_ast::Item::FnDef(f) => {
            out.push_str(&format!("{}fn {}(", pad, f.ident));
            for (i, p) in f.params.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!(
                    "{}: {}",
                    pattern_to_string(&p.pattern),
                    type_to_string(&p.ty)
                ));
            }
            out.push(')');
            if let Some(ty) = &f.ret_ty {
                out.push_str(&format!(" -> {}", type_to_string(ty)));
            }
            if let Some(eff) = &f.effect {
                out.push_str(&format!(" {}", effect_to_string(eff)));
            }
            out.push_str(" {\n");
            format_block(&f.body, out, indent + 1);
            out.push_str(&format!("{}}}", pad));
        }
        gala_ast::Item::StructDef(s) => {
            out.push_str(&format!("{}struct {} {{", pad, s.ident));
            for f in &s.fields {
                out.push_str(&format!(
                    "\n{}    {}: {},",
                    "    ".repeat(indent + 1),
                    f.ident,
                    type_to_string(&f.ty)
                ));
            }
            out.push_str(&format!("\n{}}}", pad));
        }
        _ => {
            out.push_str(&format!("{}// {:?}", pad, item));
        }
    }
}

fn format_block(block: &gala_ast::Block, out: &mut String, indent: usize) {
    for stmt in &block.stmts {
        format_stmt(stmt, out, indent);
    }
    if let Some(tail) = &block.tail {
        format_expr(tail, out, indent);
    }
}

fn format_stmt(stmt: &Stmt, out: &mut String, indent: usize) {
    let pad = "    ".repeat(indent);
    match stmt {
        Stmt::Let(l) => {
            let ty_str = l.ty.as_ref().map(type_to_string).unwrap_or_default();
            let init_str = l
                .init
                .as_ref()
                .map(|e| {
                    let mut s = String::new();
                    format_expr(e, &mut s, 0);
                    s
                })
                .unwrap_or_default();
            out.push_str(&format!(
                "{}let {}: {} = {};\n",
                pad,
                pattern_to_string(&l.pattern),
                ty_str,
                init_str
            ));
        }
        Stmt::Expr(e) => {
            format_expr(e, out, indent);
            out.push_str(";\n");
        }
        Stmt::Item(i) => format_item(i, out, indent),
        Stmt::Return(e) => {
            out.push_str(&format!("{}return", pad));
            if let Some(e) = e {
                out.push(' ');
                format_expr(e, out, indent);
            }
            out.push_str(";\n");
        }
    }
}

fn format_expr(expr: &Expr, out: &mut String, indent: usize) {
    let pad = "    ".repeat(indent);
    match expr {
        Expr::Literal(l) => out.push_str(&literal_to_string(l)),
        Expr::Ident(i) => out.push_str(&i.0),
        Expr::Binary(b) => {
            out.push_str(&format!(
                "({} {} {})",
                expr_to_string(&b.lhs),
                binop_to_string(b.op),
                expr_to_string(&b.rhs)
            ));
        }
        Expr::Call(c) => {
            out.push_str(&format!(
                "{}({})",
                expr_to_string(&c.callee),
                c.args.iter().map(expr_to_string).collect::<Vec<_>>().join(", ")
            ));
        }
        Expr::Block(b) => {
            out.push_str("{\n");
            format_block(b, out, indent + 1);
            out.push_str(&format!("{}}}", pad));
        }
        _ => out.push_str(&format!("{:?}", expr)),
    }
}

fn literal_to_string(l: &gala_ast::Literal) -> String {
    match l {
        gala_ast::Literal::Int(i) => i.to_string(),
        gala_ast::Literal::Float(f) => f.to_string(),
        gala_ast::Literal::Bool(b) => b.to_string(),
        gala_ast::Literal::String(s) => format!("\"{}\"", s),
        gala_ast::Literal::Unit => "()".to_string(),
        gala_ast::Literal::Complex { re, im } => format!("{}+{}i", re, im),
    }
}

fn expr_to_string(e: &Expr) -> String {
    let mut out = String::new();
    format_expr(e, &mut out, 0);
    out
}

fn binop_to_string(op: gala_ast::BinOp) -> &'static str {
    match op {
        gala_ast::BinOp::Add => "+",
        gala_ast::BinOp::Sub => "-",
        gala_ast::BinOp::Mul => "*",
        gala_ast::BinOp::Div => "/",
        gala_ast::BinOp::Mod => "%",
        gala_ast::BinOp::Eq => "==",
        gala_ast::BinOp::Ne => "!=",
        gala_ast::BinOp::Lt => "<",
        gala_ast::BinOp::Le => "<=",
        gala_ast::BinOp::Gt => ">",
        gala_ast::BinOp::Ge => ">=",
        gala_ast::BinOp::And => "&&",
        gala_ast::BinOp::Or => "||",
        gala_ast::BinOp::Range => "..",
    }
}

fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(p) => path_to_string(p),
        Type::Qubits(c) => format!("Qubits<{}>", const_expr_to_string(c)),
        Type::Qubit => "Qubit".to_string(),
        Type::Measured(t) => format!("Measured<{}>", type_to_string(t)),
        Type::Tuple(ts) => {
            format!("({})", ts.iter().map(type_to_string).collect::<Vec<_>>().join(", "))
        }
        Type::Array(t, c) => format!("[{}; {}]", type_to_string(t), const_expr_to_string(c)),
        Type::Fn { params, ret, .. } => format!(
            "fn({}) -> {}",
            params.iter().map(type_to_string).collect::<Vec<_>>().join(", "),
            type_to_string(ret)
        ),
        Type::Named(s, ts) => {
            format!("{}<{}>", s, ts.iter().map(type_to_string).collect::<Vec<_>>().join(", "))
        }
    }
}

fn path_to_string(p: &gala_ast::Path) -> String {
    p.segments.iter().map(|s| s.ident.0.clone()).collect::<Vec<_>>().join(".")
}

fn const_expr_to_string(c: &gala_ast::ConstExpr) -> String {
    match c {
        gala_ast::ConstExpr::Int(i) => i.to_string(),
        gala_ast::ConstExpr::Ident(i) => i.0.clone(),
        gala_ast::ConstExpr::Binary { lhs, op, rhs } => format!(
            "({} {} {})",
            const_expr_to_string(lhs),
            binop_to_string(*op),
            const_expr_to_string(rhs)
        ),
    }
}

fn effect_to_string(eff: &gala_ast::Effect) -> &'static str {
    match eff {
        gala_ast::Effect::Pure => "pure",
        gala_ast::Effect::Quantum => "quantum",
        gala_ast::Effect::Prob => "prob",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_fn() {
        let input = "fn main() -> Int { return 42; }";
        let result = format_source(input);
        assert!(result.contains("fn main"));
        assert!(result.contains("return 42"));
    }

    #[test]
    fn test_is_formatted_roundtrip() {
        let input = "fn main() -> Int { return 42; }";
        let formatted = format_source(input);
        // Should be idempotent
        assert_eq!(format_source(&formatted), formatted);
    }

    #[test]
    fn test_format_empty() {
        assert_eq!(format_source(""), "");
    }

    #[test]
    fn test_binop_to_string() {
        assert_eq!(binop_to_string(gala_ast::BinOp::Add), "+");
        assert_eq!(binop_to_string(gala_ast::BinOp::Mul), "*");
        assert_eq!(binop_to_string(gala_ast::BinOp::Eq), "==");
        assert_eq!(binop_to_string(gala_ast::BinOp::Range), "..");
    }

    #[test]
    fn test_effect_to_string() {
        assert_eq!(effect_to_string(&gala_ast::Effect::Pure), "pure");
        assert_eq!(effect_to_string(&gala_ast::Effect::Quantum), "quantum");
        assert_eq!(effect_to_string(&gala_ast::Effect::Prob), "prob");
    }

    #[test]
    fn test_literal_to_string() {
        let int_lit = gala_ast::Literal::Int(42);
        assert_eq!(literal_to_string(&int_lit), "42");

        let bool_lit = gala_ast::Literal::Bool(true);
        assert_eq!(literal_to_string(&bool_lit), "true");

        let unit_lit = gala_ast::Literal::Unit;
        assert_eq!(literal_to_string(&unit_lit), "()");

        let str_lit = gala_ast::Literal::String("hello".to_string());
        assert_eq!(literal_to_string(&str_lit), "\"hello\"");
    }
}
