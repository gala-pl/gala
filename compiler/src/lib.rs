pub mod ast;
pub mod codegen;
pub mod diagnostics;
pub mod ir;
pub mod lexer;
pub mod parser;

use diagnostics::Diagnostics;

pub fn compile(source: &str) -> Result<String, Diagnostics> {
    let mut diags = Diagnostics::new();

    let lexer = lexer::Lexer::new(source);
    let mut parser = parser::Parser::new(lexer);
    let stmts = match parser.parse() {
        Ok(stmts) => stmts,
        Err(msg) => {
            diags.push(diagnostics::Diagnostic::error(msg));
            return Err(diags);
        }
    };

    let output = codegen::generate(&stmts);
    Ok(output)
}
