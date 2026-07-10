" Gala filetype plugin
if exists("b:did_ftplugin")
  finish
endif

setlocal commentstring=//\ %s
setlocal formatoptions-=t formatoptions+=croql
setlocal path+=src/**,tests/**
setlocal suffixesadd=.gala

if executable('gala')
  setlocal makeprg=gala\ check\ %
  setlocal errorformat=%-G%f:%l:%c:%trror:%m
endif

let b:did_ftplugin = 1