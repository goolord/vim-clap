" Author: liuchengxu <xuliuchengxlc@gmail.com>
" Description: Vim client for the daemon job.

let s:save_cpo = &cpoptions
set cpoptions&vim

let s:req_id = get(s:, 'req_id', 0)
let s:session_id = get(s:, 'session_id', 0)
let s:handlers = get(s:, 'handlers', {})

function! s:process_filter_result(msg) abort
  if g:clap.display.win_is_valid()
    if !has_key(a:msg, 'query') || a:msg.query ==# g:clap.input.get()
      call clap#state#process_filter_result(a:msg)
      call clap#preview#async_open_with_delay()
    endif
  endif
endfunction

function! s:set_total_size(msg) abort
  let g:clap.display.initial_size = a:msg.total
endfunction

function! clap#client#handle(msg) abort
  let decoded = json_decode(a:msg)

  if has_key(decoded, 'method')
    call call(decoded.method, [decoded])
    return
  endif

  if has_key(decoded, 'force_execute') && has_key(s:handlers, decoded.id)
    let Handler = remove(s:handlers, decoded.id)
    call Handler(get(decoded, 'result', v:null), get(decoded, 'error', v:null))
    return
  endif

  " Only process the latest request, drop the outdated responses.
  if s:req_id != decoded.id
    return
  endif

  if has_key(s:handlers, decoded.id)
    let Handler = remove(s:handlers, decoded.id)
    call Handler(get(decoded, 'result', v:null), get(decoded, 'error', v:null))
    return
  endif
endfunction

function! clap#client#notify_on_init(method, ...) abort
  let s:session_id += 1
  let params = {
        \   'cwd': clap#rooter#working_dir(),
        \   'provider_id': g:clap.provider.id,
        \   'source_fpath': expand('#'.g:clap.start.bufnr.':p'),
        \   'display_winwidth': winwidth(g:clap.display.winid),
        \ }
  if has_key(g:clap.preview, 'winid')
        \ && clap#api#floating_win_is_valid(g:clap.preview.winid)
    let params['preview_winheight'] = winheight(g:clap.preview.winid)
  endif
  if g:clap.provider.id ==# 'help_tags'
    let params['runtimepath'] = &runtimepath
  endif
  if a:0 > 0
    call extend(params, a:1)
  endif
  call clap#client#notify(a:method, params)
endfunction

function! clap#client#call_on_init(method, callback, ...) abort
  call call(function('clap#client#notify_on_init'), [a:method] + a:000)
  let s:handlers[s:req_id] = a:callback
endfunction

" One optional argument: Dict, extra params
function! clap#client#call_on_move(method, callback, ...) abort
  let curline = g:clap.display.getcurline()
  if empty(curline)
    return
  endif
  let params = {'curline': curline}
  if a:0 > 0
    call extend(params, a:1)
  endif
  call clap#client#call(a:method, a:callback, params)
endfunction

function! clap#client#call_with_lnum(method, callback, ...) abort
  let params = {'lnum': g:__clap_display_curlnum}
  if g:clap.provider.id ==# 'grep'
    let params['enable_icon'] = g:clap_provider_grep_enable_icon ? v:true : v:false
  endif
  if a:0 > 0
    call extend(params, a:1)
  endif
  call clap#client#call(a:method, a:callback, params)
endfunction

function! clap#client#call_preview_file(extra) abort
  call clap#client#call('preview/file', function('clap#impl#on_move#handler'), clap#preview#maple_opts(a:extra))
endfunction

function! clap#client#notify(method, params) abort
  let s:req_id += 1
  call clap#job#daemon#send_message(json_encode({
        \ 'id': s:req_id,
        \ 'session_id': s:session_id,
        \ 'method': a:method,
        \ 'params': a:params,
        \ }))
endfunction

let s:notify_delay = 200
let s:notify_timer = -1
function! clap#client#notify_with_delay(method, params) abort
  if s:notify_timer != -1
    call timer_stop(s:notify_timer)
  endif
  let s:notify_timer = timer_start(s:notify_delay, { -> clap#client#notify(a:method, a:params)})
endfunction

function! clap#client#notify_recent_file() abort
  if &buftype ==# 'nofile'
    return
  endif
  call clap#client#call('note_recent_files', v:null, {'file': expand(expand('<afile>:p'))})
endfunction

function! clap#client#call(method, callback, params) abort
  call clap#client#notify(a:method, a:params)
  if a:callback isnot v:null
    let s:handlers[s:req_id] = a:callback
  endif
endfunction

let &cpoptions = s:save_cpo
unlet s:save_cpo
