" Gala indent: expression-oriented, brace-delimited, cindent-based
if exists("b:did_indent")
  finish
endif

setlocal autoindent
setlocal smartindent
setlocal cinwords=fn,if,else,for,while,match,impl,trait
setlocal cino+={1s,e0,^-1s,:0

setlocal shiftwidth=4
setlocal softtabstop=4
setlocal tabstop=4
setlocal expandtab

let b:did_indent = 1