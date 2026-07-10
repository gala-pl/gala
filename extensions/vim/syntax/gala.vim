if exists("b:current_syntax")
  finish
endif

syntax case match

" Comments
syntax match galaLineComment "\/\/.*$" contains=@Spell
syntax region galaBlockComment start="/\*" end="\*/" fold contains=galaBlockComment
syntax match galaDocComment "\/\/\/.*$" contains=@Spell

" Strings
syntax region galaString start=+"+ skip=+\\\\\|\\"+ end=+"+ contains=galaEscape
syntax match galaEscape '\\[nrt0\\"]' contained

" Numbers
syntax match galaInteger "\<\d\+\>"
syntax match galaHex "\<0[xX][0-9a-fA-F]\+\>"
syntax match galaBin  "\<0[bB][01]\+\>"
syntax match galaFloat "\<\d\+\.\d*\([eE][+-]\=\d\+\)\=\>"
syntax match galaComplex "\<\d\+[iIjJ]\>"

" Types
syntax keyword galaType Qubit Qubits Measured Bool Int Float Complex
syntax keyword galaType Vec Params String Unit Self

" Effects
syntax keyword galaEffect pure quantum prob

" Quantum keywords
syntax keyword galaQuantum qubit qubits measure reverse adjoint control grad drop

" Gate operations
syntax keyword galaGate h x y z s t rx ry rz cx cz swap

" Built-in functions
syntax keyword galaBuiltin print println assert len sample expectation distribution

" Keyword groups
syntax keyword galaKeyword fn let mut if else match for in while return
syntax keyword galaKeyword import from as type struct enum trait impl const where

" Built-in constants
syntax keyword galaBoolean true false

" Function declarations
syntax match galaFunction "\<\h\w*\>(?=\s*\()" contains=NONE

" Operators
syntax match galaOperator "[+\-*/%]"
syntax match galaOperator "=="
syntax match galaOperator "!="
syntax match galaOperator "<="
syntax match galaOperator ">="
syntax match galaOperator "<\|>"
syntax match galaOperator "&&"
syntax match galaOperator "||"
syntax match galaOperator "!"
syntax match galaOperator "="
syntax match galaOperator "->"
syntax match galaOperator "\.\."
syntax match galaOperator "|"

" Delimiters
syntax match galaDelimiter "[,;:(){}[\]]"

" Highlight links
highlight default link galaLineComment   Comment
highlight default link galaBlockComment  Comment
highlight default link galaDocComment      SpecialComment
highlight default link galaEscape        SpecialChar
highlight default link galaString        String
highlight default link galaInteger       Number
highlight default link galaHex           Number
highlight default link galaBin           Number
highlight default link galaFloat          Float
highlight default link galaComplex       Number
highlight default link galaType          Type
highlight default link galaEffect        Keyword
highlight default link galaQuantum       Special
highlight default link galaGate          Function
highlight default link galaBuiltin       Function
highlight default link galaKeyword       Keyword
highlight default link galaBoolean       Boolean
highlight default link galaFunctionDecl  Function
highlight default link galaOperator     Operator
highlight default link galaDelimiter     Delimiter

let b:current_syntax = "gala"