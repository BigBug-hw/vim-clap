" Author: liuchengxu <xuliuchengxlc@gmail.com>
" Description: Persistent projects, ordered by the Mozilla's Frecency algorithm.

let s:save_cpo = &cpoptions
set cpoptions&vim

let s:projects = {}

function! s:projects.on_typed() abort
  call clap#client#call('projects/on_typed', function('clap#state#handle_response_on_typed'), {
        \ 'provider_id': g:clap.provider.id,
        \ 'query': g:clap.input.get(),
        \ 'enable_icon': g:clap_enable_icon ? v:true : v:false,
        \ 'lnum': g:__clap_display_curlnum
        \ })
endfunction

function! s:projects.on_move_async() abort
  call clap#client#call_with_lnum('projects/on_move', function('clap#impl#on_move#handler'))
endfunction

function! s:projects.init() abort
  call clap#client#call_on_init(
        \ 'projects/on_init', function('clap#state#handle_response_on_typed'), clap#client#init_params(v:null))
endfunction

let s:projects.sink = function('clap#provider#files#sink_impl')
let s:projects.support_open_action = v:true
let s:projects.syntax = 'clap_files'

let g:clap#provider#projects# = s:projects

let &cpoptions = s:save_cpo
unlet s:save_cpo
