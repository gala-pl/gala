" After-syntax refinements for Gala
" Highlight error codes like E0530
syntax match galaErrorCode '\<E\d\{4}\>'
highlight default link galaErrorCode Error

" Highlight doc-comment keywords (like @param, @return)
syntax match galaDocTag '@\w\+' contained containedin=galaDocComment
highlight default link galaDocTag Special