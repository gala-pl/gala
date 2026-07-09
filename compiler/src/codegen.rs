use crate::ast::*;

pub fn generate(stmts: &[Stmt]) -> String {
    let mut output = String::new();
    for stmt in stmts {
        emit_stmt(stmt, &mut output, 0);
    }
    output
}

fn emit_stmt(stmt: &Stmt, out: &mut String, indent: usize) {
    match stmt {
        Stmt::Expr(expr) => {
            let pad = "    ".repeat(indent);
            match expr {
                Expr::FnDef(_) => {
                    emit_expr(expr, out, indent);
                    out.push('\n');
                }
                _ => {
                    out.push_str(&pad);
                    emit_expr(expr, out, indent);
                    if !matches!(expr, Expr::If(_, _, _) | Expr::Block(_)) {
                        out.push(';');
                    }
                    out.push('\n');
                }
            }
        }
        Stmt::Let(name, expr) => {
            let pad = "    ".repeat(indent);
            out.push_str(&format!("{pad}let {name} = "));
            emit_expr(expr, out, indent);
            out.push_str(";\n");
        }
        Stmt::Return(expr) => {
            let pad = "    ".repeat(indent);
            match expr {
                Some(e) => {
                    out.push_str(&format!("{pad}return "));
                    emit_expr(e, out, indent);
                    out.push_str(";\n");
                }
                None => out.push_str(&format!("{pad}return;\n")),
            }
        }
    }
}

fn emit_expr(expr: &Expr, out: &mut String, indent: usize) {
    match expr {
        Expr::Int(n) => out.push_str(&n.to_string()),
        Expr::Float(n) => out.push_str(&n.to_string()),
        Expr::String(s) => out.push_str(&format!("\"{s}\"")),
        Expr::Bool(b) => out.push_str(&b.to_string()),
        Expr::Ident(s) => out.push_str(s),
        Expr::BinOp(l, op, r) => {
            out.push('(');
            emit_expr(l, out, indent);
            out.push_str(match op {
                BinOp::Add => " + ",
                BinOp::Sub => " - ",
                BinOp::Mul => " * ",
                BinOp::Div => " / ",
                BinOp::Mod => " % ",
                BinOp::Eq => " == ",
                BinOp::Ne => " != ",
                BinOp::Lt => " < ",
                BinOp::Le => " <= ",
                BinOp::Gt => " > ",
                BinOp::Ge => " >= ",
                BinOp::And => " && ",
                BinOp::Or => " || ",
            });
            emit_expr(r, out, indent);
            out.push(')');
        }
        Expr::UnOp(op, e) => {
            out.push_str(match op {
                UnOp::Neg => "-",
                UnOp::Not => "!",
            });
            emit_expr(e, out, indent);
        }
        Expr::Call(f, args) => {
            emit_expr(f, out, indent);
            out.push('(');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                emit_expr(arg, out, indent);
            }
            out.push(')');
        }
        Expr::If(cond, then, else_) => {
            out.push_str("if ");
            emit_expr(cond, out, indent);
            out.push_str(" {\n");
            if let Expr::Block(stmts) = then.as_ref() {
                for s in stmts {
                    emit_stmt(s, out, indent + 1);
                }
            }
            let pad = "    ".repeat(indent);
            out.push_str(&format!("{pad}}}"));
            if let Some(else_) = else_ {
                out.push_str(" else ");
                match else_.as_ref() {
                    Expr::If(_, _, _) => emit_expr(else_, out, indent),
                    Expr::Block(stmts) => {
                        out.push_str("{\n");
                        for s in stmts {
                            emit_stmt(s, out, indent + 1);
                        }
                        out.push_str(&format!("{pad}}}"));
                    }
                    e => emit_expr(e, out, indent),
                }
            }
        }
        Expr::Block(stmts) => {
            out.push_str("{\n");
            for s in stmts {
                emit_stmt(s, out, indent + 1);
            }
            let pad = "    ".repeat(indent);
            out.push_str(&format!("{pad}}}"));
        }
        Expr::Let(name, expr) => {
            out.push_str(&format!("let {name} = "));
            emit_expr(expr, out, indent);
        }
        Expr::FnDef(fdef) => {
            let pad = "    ".repeat(indent);
            out.push_str(&format!("{}fn {}({})", pad, fdef.name,
                fdef.params.iter()
                    .map(|(n, t)| format!("{n}: {}", type_str(t)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            if fdef.ret != Type::Unit {
                out.push_str(&format!(" -> {}", type_str(&fdef.ret)));
            }
            out.push(' ');
            emit_expr(&fdef.body, out, indent);
        }
    }
}

fn type_str(t: &Type) -> &str {
    match t {
        Type::Int => "Int",
        Type::Float => "Float",
        Type::Bool => "Bool",
        Type::String => "String",
        Type::Unit => "Unit",
        Type::Fn(_, _) => "Fn",
        Type::Named(s) => s,
    }
}
