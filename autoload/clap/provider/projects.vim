" Author: bigbug
" Description: projects provider

let s:save_cpo = &cpoptions
set cpoptions&vim

let s:projects = {}

function! s:projects.init() abort
  call clap#client#notify_on_init()
endfunction

function! clap#provider#projects#handle_on_initialize(result) abort
  let result = a:result
  call g:clap.display.set_lines(result.entries)
  call clap#sign#reset_to_first_line()
  call clap#indicator#update_processed(result.total)
  call clap#sign#reset_to_first_line()
  call g:clap#display_win.shrink_if_undersize()
endfunction

function! s:format_buffer(b) abort
  let buffer_name = bufname(a:b)
  let fullpath = empty(buffer_name) ? '[No Name]' : fnamemodify(buffer_name, ':p:~:.')
  let filename = empty(fullpath) ? '[No Name]' : fnamemodify(fullpath, ':t')
  let flag = a:b == bufnr('')  ? '%' : (a:b == bufnr('#') ? '#' : ' ')
  let modified = getbufvar(a:b, '&modified') ? ' [+]' : ''
  let readonly = getbufvar(a:b, '&modifiable') ? '' : ' [RO]'

  let filename = s:padding(filename, 25)
  let bp = s:padding('['.a:b.']', 5)
  let fsize = s:padding(clap#util#getfsize(fullpath), 6)
  let icon = g:clap_enable_icon ? s:padding(clap#icon#for(fullpath), 3) : ''
  let extra = join(filter([modified, readonly], '!empty(v:val)'), '')
  let line = s:padding(get(s:line_info, a:b, ''), 10)

  return trim(printf('%s %s %s %s %s %s %s %s', bp, filename, fsize, icon, line, fullpath, flag, extra))
endfunction

function! s:projects.source() abort
  let l:buffers = execute('buffers')
  let s:line_info = {}
  for line in split(l:buffers, "\n")
    let bufnr = str2nr(trim(matchstr(line, '^\s*\d\+')))
    let lnum = matchstr(line, '\s\+\zsline.*$')
    let s:line_info[bufnr] = lnum
  endfor
  let bufs = map(clap#util#buflisted_sorted(v:true), 's:format_buffer(str2nr(v:val))')
  if empty(bufs)
    return []
  else
    return bufs[1:] + [bufs[0]]
  endif
endfunction

function! s:projects.on_typed() abort
  call clap#client#notify('on_typed')
endfunction

function! s:extract(row) abort
  let lnum = matchstr(a:row, '^.*:\zs\(\d\+\)')
  let path = matchstr(a:row, '\[.*@\zs\(\f*\)\ze\]')
  return [lnum, path]
endfunction

function! s:projects.sink(selected) abort
  let [lnum, path] = s:extract(a:selected)
  call clap#sink#open_file(path, lnum, 1)
endfunction

function! s:projects.on_move() abort
  let [lnum, path] = s:extract(g:clap.display.getcurline())
  call clap#preview#file_at(path, lnum)
endfunction

function! s:projects.on_exit() abort
  if exists('g:__clap_match_scope_enum')
    unlet g:__clap_match_scope_enum
  endif
endfunction

let s:projects.enable_rooter = v:true
let s:projects.icon = 'File'
let s:projects.on_move_async = function('clap#impl#on_move#async')
let s:projects.syntax = 'clap_projects'
let s:projects.support_open_action = v:false

let g:clap#provider#projects# = s:projects

let &cpoptions = s:save_cpo
unlet s:save_cpo
