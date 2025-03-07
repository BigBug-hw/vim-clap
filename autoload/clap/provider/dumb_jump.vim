" Author: liuchengxu <xuliuchengxlc@gmail.com>
" Description: Jump to definition/reference based on the regexp.

scriptencoding utf-8

let s:save_cpo = &cpoptions
set cpoptions&vim

let s:dumb_jump = {}

function! s:dumb_jump.sink(selected) abort
  let pattern = '^\[\(\a\+\)\]\zs\(.*\):\(\d\+\):\(\d\+\):'
  let matched = matchlist(a:selected, pattern)
  let [fpath, linenr, column] = [matched[2], str2nr(matched[3]), str2nr(matched[4])]
  call clap#sink#open_file(fpath, linenr, column)
endfunction

function! s:into_qf_item(line) abort
  let pattern = '^\[\(\a\+\)\]\zs\(.*\):\(\d\+\):\(\d\+\):\(.*\)'
  let matched = matchlist(a:line, pattern)
  let [fpath, linenr, column, text] = [matched[2], str2nr(matched[3]), str2nr(matched[4]), matched[5]]
  return {'filename': fpath, 'lnum': linenr, 'col': column, 'text': text}
endfunction

function! s:dumb_jump_sink_star(lines) abort
  call clap#sink#open_quickfix(map(a:lines, 's:into_qf_item(v:val)'))
endfunction

function! s:dumb_jump.on_typed() abort
  let query = g:clap.input.get()
  if empty(query)
    call clap#highlight#clear()
    return
  endif
  call clap#client#notify('on_typed')
endfunction

function! s:dumb_jump.init() abort
  call clap#client#notify_on_init()
endfunction

function! s:dumb_jump.on_move_async() abort
  call clap#client#notify('on_move')
endfunction

let s:dumb_jump['sink*'] = function('s:dumb_jump_sink_star')
let s:dumb_jump.syntax = 'clap_dumb_jump'
let s:dumb_jump.enable_rooter = v:true
let g:clap#provider#dumb_jump# = s:dumb_jump

let &cpoptions = s:save_cpo
unlet s:save_cpo
