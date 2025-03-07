<p align="center">
    <img width="300px" src="https://user-images.githubusercontent.com/8850248/67629807-ee76a500-f8b6-11e9-8777-264a897dd9d4.png">
</p>

[![CI](https://github.com/liuchengxu/vim-clap/workflows/ci/badge.svg)](https://github.com/liuchengxu/vim-clap/actions?workflow=ci)
[![Gitter][g1]][g2]
[![Donate](https://img.shields.io/badge/Donate-PayPal-green.svg)](https://paypal.me/liuchengxu)

[g1]: https://badges.gitter.im/liuchengxu/vim-clap.svg
[g2]: https://gitter.im/liuchengxu/vim-clap?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge

Vim-clap is a modern generic performant finder using the `floating_win` of neovim or `popup` of vim, powered by an external backend written in Rust.

<p align="center">
  <img width="600px" src="https://user-images.githubusercontent.com/8850248/73323347-24467380-4282-11ea-8dac-5ef5a1ee63bb.gif">
</p>

[>>>> More screenshots](https://github.com/liuchengxu/vim-clap/issues/1)

## Table of Contents

<!-- TOC GFM -->

* [Features](#features)
* [Caveats](#caveats)
* [Requirement](#requirement)
* [Installation](#installation)
  * [vim-plug](#vim-plug)
* [Usage](#usage)
  * [Commands](#commands)
    * [Providers](#providers)
  * [Global variables](#global-variables)
  * [Keybindings](#keybindings)
    * [Insert mode](#insert-mode)
    * [NeoVim only](#neovim-only)
      * [Normal mode](#normal-mode)
      * [Cmdline mode](#cmdline-mode)
  * [Execute some code during the process](#execute-some-code-during-the-process)
  * [Change highlights](#change-highlights)
  * [Search syntax](#search-syntax)
    * [Fzf search syntax](#fzf-search-syntax)
    * [Extended search syntax](#extended-search-syntax)
  * [Config file](#config-file)
* [How to define your own provider](#how-to-define-your-own-provider)
* [Disable auto-completion plugin in clap input window](#disable-auto-completion-plugin-in-clap-input-window)
* [Contribution](#contribution)
* [Credit](#credit)
* [License](#license)

<!-- /TOC -->

## Features

- [x] ~~Pure vimscript~~.
  - Pin to some early version of vim-clap if you prefer the pure vimscript plugin.
- [x] ~~Work out of the box, without any extra dependency~~.
  - The Rust binary is required to have a decent user experience.
- [x] Blazingly fast thanks to the powerful Rust backend.
- [x] Extensible, easy to add new providers.
- [x] Support [the search syntax borrowed from fzf](https://github.com/junegunn/fzf#search-syntax) and more.
- [x] Flexible UI layout.
- [ ] Support searching by multiple providers simultaneously.

## Caveats

- Vim-clap is in a very early stage, breaking changes and bugs are expected.

- The Windows support is not fully tested. The providers without using any system related command should work smoothly, that is to say, most sync providers are just able to work. Please [create an issue](https://github.com/liuchengxu/vim-clap/issues/new?assignees=&labels=&template=bug_report.md&title=) if you run into any error in Windows. And any help would be appreciated.

- Although a lot of effort has been made to unify the behavior of vim-clap between vim and neovim, and most part works in the same way, it just can't be exactly the same, for `floating_win` and `popup` are actually two different things anyway.

## Requirement

- Vim: `:echo has('patch-8.1.2114')`.
- NeoVim: `:echo has('nvim-0.4.2')`.

## Installation

### [vim-plug](https://github.com/junegunn/vim-plug)

```vim
" Build the Rust binary if `cargo` exists on your system.
Plug 'liuchengxu/vim-clap', { 'do': ':Clap install-binary' }

" The bang version will try to download the prebuilt binary if `cargo` does not exist.
Plug 'liuchengxu/vim-clap', { 'do': ':Clap install-binary!' }

" `:Clap install-binary[!]` will always try to compile the binary locally.
" If you do care about the disk used for the compilation, use the way of force download,
" which will directly download the prebuilt binary even if `cargo` is available.
Plug 'liuchengxu/vim-clap', { 'do': { -> clap#installer#force_download() } }

" `:Clap install-binary[!]` will run using the terminal feature which is inherently async.
" If you don't want that and hope to run the hook synchorously:
Plug 'liuchengxu/vim-clap', { 'do': has('win32') ? 'cargo build --release' : 'make' }
```

It's recommended to install the Rust binary automatically via the `do` hook. If that does not work for you, please refer to [INSTALL.md](INSTALL.md) for installing it manually.

## Usage

Vim-clap is utterly easy to use, just type, press Ctrl-J/K to locate the wanted entry, and press Enter to apply and exit. The default settings should work well for most people in most cases, but it's absolutely hackable too.

### Commands

The paradigm is `Clap [provider_id_or_alias] {provider_args}`, where the `provider_id_or_alias` is obviously either the name or alias of provider. Technically the `provider_id` can be anything that can be used a key of a Dict, but I recommend you using an _identifier_ like name as the provider id, and use the alias rule if you prefer a special name.

<details>
  <summary>cache is no longer necessary since v0.37.</summary>

You can use `+no-cache` option to disable/refresh the cache, e.g., `:Clap files +no-cache ~` for searching files under the home directory without cache, the shortcut for `+no-cache` option:

- `:Clap!! [provider_id_or_alias] {provider_args}`, e.g, `:Clap!! files ~`.
- `:Clap [provider_id_or_alias][!] {provider_args}`, e.g, `:Clap files! ~`. (Recommended)

Note the `*` in the spinner, it tells you are using the cache, use `g:clap_forerunner_status_sign` to configure it.

<img width="561" alt="cache spinner" src="https://user-images.githubusercontent.com/8850248/78767291-fafe3e00-79bc-11ea-91a8-e17518e7a1b2.png">
</details>

#### Providers

| Command                                | List                                                   | Requirement                                                             |
| :------------------------------------- | :----------------------------------------------------- | :---------------------------------------------------------------------- |
| `Clap blines`                          | Lines in the current buffer                            | _none_                                                                  |
| `Clap buffers`                         | Open buffers                                           | _none_                                                                  |
| `Clap colors`                          | Colorschemes                                           | _none_                                                                  |
| `Clap command`                         | Command                                                | _none_                                                                  |
| `Clap hist:` or `Clap command_history` | Command history                                        | _none_                                                                  |
| `Clap hist/` or `Clap search_history`  | Search history                                         | _none_                                                                  |
| `Clap filetypes`                       | File types                                             | _none_                                                                  |
| `Clap help_tags`                       | Help tags                                              | _none_                                                                  |
| `Clap jumps`                           | Jumps                                                  | _none_                                                                  |
| `Clap lines`                           | Lines in the loaded buffers                            | _none_                                                                  |
| `Clap marks`                           | Marks                                                  | _none_                                                                  |
| `Clap maps`                            | Maps                                                   | _none_                                                                  |
| `Clap quickfix`                        | Entries of the quickfix list                           | _none_                                                                  |
| `Clap loclist`                         | Entries of the location list                           | _none_                                                                  |
| `Clap registers`                       | Registers                                              | _none_                                                                  |
| `Clap yanks`                           | Yank stack of the current vim session                  | _none_                                                                  |
| `Clap history`                         | Open buffers and `v:oldfiles`                          | _none_                                                                  |
| `Clap windows`                         | Windows                                                | _none_                                                                  |
| `Clap providers`                       | List the vim-clap providers                            | _none_                                                                  |
| `Clap bcommits`                        | Git commits for the current buffer                     | **[git][git]**                                                          |
| `Clap commits`                         | Git commits                                            | **[git][git]**                                                          |
| `Clap gfiles` or `Clap git_files`      | Files managed by git                                   | **[git][git]**                                                          |
| `Clap git_diff_files`                  | Files managed by git and having uncommitted changes    | **[git][git]**                                                          |
| `Clap live_grep`**<sup>+</sup>**       | Grep using word-regexp matcher                         | **[rg][rg]**                                                            |
| `Clap dumb_jump`                       | Definitions/References using regexp with grep fallback | **[rg][rg]** with `--pcre2`                                             |
| `Clap files`                           | Files                                                  | **[maple][maple]**                                                      |
| `Clap filer`                           | Ivy-like file explorer                                 | **[maple][maple]**                                                      |
| `Clap grep`**<sup>+</sup>**            | Grep using fuzzy matcher                               | **[maple][maple]**                                                      |
| `Clap tags`                            | Tags in the current buffer                             | **[maple][maple]**/**[vista.vim][vista.vim]**                           |
| `Clap proj_tags`                       | Tags in the current project                            | **[maple][maple]** and **[universal-ctags][universal-ctags]** (`+json`) |
| `Clap recent_files`                    | Persistent ordered history of recent files             | **[maple][maple]**                                                      |

[fd]: https://github.com/sharkdp/fd
[rg]: https://github.com/BurntSushi/ripgrep
[git]: https://github.com/git/git
[vista.vim]: https://github.com/liuchengxu/vista.vim
[maple]: https://github.com/liuchengxu/vim-clap/blob/master/INSTALL.md#maple-binary
[universal-ctags]: https://github.com/universal-ctags/ctags

- The command with a superscript `!` means that it is not yet implemented or not tested.
- The command with a superscript `+` means that it supports multi-selection via <kbd>Tab</kbd>.
- `:Clap grep`
  - Use `:Clap grep ++query=<cword>` to grep the word under cursor.
  - Use `:Clap grep ++query=@visual` to grep the visual selection.
  - `cwd` will be searched by default, specify the extra paths in the end to search multiple directories.
    - `:Clap grep ~/.vim/plugged/ale` with `cwd` is `~/.vim/plugged/vim-clap` will both search vim-clap and ale.

[Send a pull request](https://github.com/liuchengxu/vim-clap/pulls) if you want to get your provider listed here.

### Global variables

- `g:clap_layout`: Dict, `{ 'width': '67%', 'height': '33%', 'row': '33%', 'col': '17%' }` by default. This variable controls the size and position of vim-clap window. By default, the vim-clap window is placed relative to the currently active window. To make it relative to the whole editor modify this variable as shown below:

  ```vim
  let g:clap_layout = { 'relative': 'editor' }
  ```

- `g:clap_open_action`: Dict, `{ 'ctrl-t': 'tab split', 'ctrl-x': 'split', 'ctrl-v': 'vsplit' }`, extra key bindings for opening the selected file in a different way. NOTE: do not define a key binding which is conflicted with the other default bindings of vim-clap, and only `ctrl-*` is supported for now.

- `g:clap_provider_alias`: Dict, if you don't want to invoke some clap provider by its id(name), as it's too long or somehow, you can add an alias for that provider.

  ```vim
  " The provider name is `command_history`, with the following alias config,
  " now you can call it via both `:Clap command_history` and `:Clap hist:`.
  let g:clap_provider_alias = {'hist:': 'command_history'}
  ```

- `g:clap_selected_sign`: Dict, `{ 'text': ' >', 'texthl': "ClapSelectedSign", "linehl": "ClapSelected"}`.

- `g:clap_current_selection_sign`: Dict, `{ 'text': '>>', 'texthl': "ClapCurrentSelectionSign", "linehl": "ClapCurrentSelection"}`.

- `g:clap_no_matches_msg`: String, `'NO MATCHES FOUND'`, message to show when there is no matches found.

- `g:clap_popup_input_delay`: Number, `200ms` by default, delay for actually responsing to the input, vim only.

- `g:clap_disable_run_rooter`: Bool, `v:false`, vim-clap by default will try to run from the project root by changing `cwd` temporarily. Set it to `v:true` to run from the origin `cwd`. The project root here means the git base directory. Create an issue if you want to see more support about the project root.

The option naming convention for provider is `g:clap_provider_{provider_id}_{opt}`.

- `g:clap_provider_grep_blink`: [2, 100] by default, blink 2 times with 100ms timeout when jumping the result. Set it to [0, 0] to disable the blink.

- `g:clap_provider_grep_opts`: An empty string by default, allows you to enable flags such as `'--hidden -g "!.git/"'`.

See `:help clap-options` for more information.

### Keybindings

#### Insert mode

- [x] Use <kbd>Ctrl-j</kbd>/<kbd>Down</kbd> or <kbd>Ctrl-k</kbd>/<kbd>Up</kbd> to navigate the result list up and down linewise.
- [x] Use <kbd>PageDown</kbd>/<kbd>PageUp</kbd> to scroll the result list down and up.
- [x] Use <kbd>Ctrl-a</kbd>/<kbd>Home</kbd> to go to the start of the input.
- [x] Use <kbd>Ctrl-e</kbd>/<kbd>End</kbd> to go to the end of the input.
- [x] Use <kbd>Ctrl-c</kbd>, <kbd>Ctrl-g</kbd>, <kbd>Ctrl-[</kbd> or <kbd>Esc</kbd>(vim) to exit.
- [x] Use <kbd>Ctrl-h</kbd>/<kbd>BS</kbd> to delete previous character.
- [x] Use <kbd>Ctrl-d</kbd> to delete next character.
- [x] Use <kbd>Ctrl-b</kbd> to move cursor left one character.
- [x] Use <kbd>Ctrl-f</kbd> to move cursor right one character.
- [x] Use <kbd>Ctrl-n</kbd> for next input in the history.
- [x] Use <kbd>Ctrl-p</kbd> for previous input in the history.
- [x] Use <kbd>Enter</kbd> to select the entry and exit.
  - Use <kbd>Enter</kbd> to expand the directory or edit the file for `:Clap filer`.
- [x] By default <kbd>Alt-u</kbd> does nothing.
  - Use <kbd>Alt-u</kbd> to go up one directory in `:Clap filer`.
- [x] Use <kbd>Tab</kbd> to select multiple entries and open them using the quickfix window.(Need the provider has `sink*` support)
  - Use <kbd>Tab</kbd> to expand the directory for `:Clap filer`.
- [x] Use <kbd>Ctrl-t</kbd> or <kbd>Ctrl-x</kbd>, <kbd>Ctrl-v</kbd> to open the selected entry in a new tab or a new split.
- [x] Use <kbd>Ctrl-u</kbd> to clear inputs.
- [x] Use <kbd>Ctrl-l</kbd> to launch the whole provider list panel for invoking another provider at any time.
- [x] Use <kbd>Shift-Tab</kbd> to invoke the action dialog(vim only).
- [x] Use <kbd>Shift-up</kbd> and <kbd>Shift-down</kbd> to scroll the preview.

#### NeoVim only

##### Normal mode

- [x] Use <kbd>j</kbd>/<kbd>Down</kbd> or <kbd>k</kbd>/<kbd>Up</kbd> to navigate the result list up and down linewise.
- [x] By default <kbd>Alt-u</kbd> does nothing.
  - Use <kbd>Alt-u</kbd> to go up one directory in `:Clap filer`.
- [x] Use <kbd>Ctrl-c</kbd>, <kbd>Ctrl-g</kbd> or <kbd>Esc</kbd> to exit.
- [x] Use <kbd>Ctrl-d</kbd>/<kbd>Ctrl-u</kbd>/<kbd>PageDown</kbd>/<kbd>PageUp</kbd> to scroll the result list down and up.
- [x] Use <kbd>Ctrl-l</kbd> to launch the whole provider list panel for invoking another provider at any time.
- [x] Use <kbd>Ctrl-n</kbd> for next input in the history.
- [x] Use <kbd>Ctrl-p</kbd> for previous input in the history.
- [x] Use <kbd>Shift-up</kbd> and <kbd>Shift-down</kbd> to scroll the preview.
- [x] Use <kbd>gg</kbd> and <kbd>G</kbd> to scroll to the first and last item.
- [x] Use <kbd>Enter</kbd> to select the entry and exit.
- [x] Use <kbd>Shift-Tab</kbd> to invoke the action dialog.
- [x] Actions defined by `g:clap_open_action`.

##### Cmdline mode

- [x] Use `:q` to exit.

See `:help clap-keybindings` for more information.

### Execute some code during the process

```vim
augroup YourGroup
  autocmd!
  autocmd User ClapOnEnter   call YourFunction()
  autocmd User ClapOnExit    call YourFunction()
augroup END
```

### Change highlights

By default vim-clap will use the colors extracted from your colorscheme, which is not guaranteed to suitable for all the colorschemes. Then you can try the built-in `material_design_dark` theme then:

```vim
let g:clap_theme = 'material_design_dark'
```

![clap-highlights](https://user-images.githubusercontent.com/8850248/74818883-6cfdc380-533a-11ea-81fb-d09d90498c96.png)

You could also set `g:clap_theme` to be a `Dict` to specify the palette:

```vim
" Change the CamelCase of related highlight group name to under_score_case.
let g:clap_theme = { 'search_text': {'guifg': 'red', 'ctermfg': 'red'} }
```

`ClapDisplay` and `ClapPreview` are the most basic highlight groups for the display and preview window, which can be overrided if the provider has its own syntax highlight, then checkout the related [syntax](syntax) file for more granular highlights directly.

If you want to write your own clap theme, take [autoload/clap/themes/material_design_dark.vim](autoload/clap/themes/material_design_dark.vim) as a reference.

See `:help clap-highlights` for more information.

### Search syntax

#### Fzf search syntax

vim-clap adopts the almost all fzf search syntax, please refer to [the search syntax section of fzf's README](https://github.com/junegunn/fzf#search-syntax) for more details. Note that the OR operator defined by a single bar character is not yet implemented, but you can achieve that by using multiple exact matches.

#### Extended search syntax

Apart from the basic fzf search syntax, more search syntax are supported:

| Token  | Match type | Description                                                  |
| ------ | ---------- | ------------------------------------------------------------ |
| `"cli` | word-match | Items that match word `cli` (`clippy` does not match `"cli`) |

### Config file

User config file is loaded from:

- Linux: `~/.config/vimclap/config.toml`
- macOS: `~/Library/Application\ Support/org.vim.Vim-Clap/config.toml`
- Windows: `C:\Users\Alice\AppData\Roaming\Vim\Vim Clap\config\config.toml`

```toml
[matcher]
# There are four sort keys for results: score, begin, end, length, you can specify how the records are sorted using `tiebreak`.
tiebreak = "score,-begin,-end,-length"
```

## How to define your own provider

```vim
" `:Clap quick_open` to open some dotfiles quickly.
" `description` is actually optional, but if you want to show this provider
" when you call `:Clap`, the `description` is neccessary.
let g:clap_provider_quick_open = {
      \ 'source': ['~/.vimrc', '~/.spacevim', '~/.bashrc', '~/.tmux.conf'],
      \ 'sink': 'e',
      \ 'description': 'Quick open some dotfiles',
      \ }
```

Find more examples at [wiki/Examples](https://github.com/liuchengxu/vim-clap/wiki/Examples).

For complete guide about writing a clap provider please see [PROVIDER.md](PROVIDER.md).

## Disable auto-completion plugin in clap input window

Some of the auto-completion engines need to turn off to prevent bizarre behaviors(#580)

For nvim-completion, add autocmd to your init.vim:

```vim
autocmd FileType clap_input let g:completion_enable_auto_pop = 0
```

For nvim-compe:

```vim
autocmd FileType clap_input call compe#setup({ 'enabled': v:false }, 0)
```

## Contribution

Vim-clap is still in beta. Any kinds of contributions are highly welcome.

If you would like to see support for more providers or share your own provider, please [create an issue](https://github.com/liuchengxu/vim-clap/issues) or [create a pull request](https://github.com/liuchengxu/vim-clap/pulls).

If you'd liked to discuss the project more directly, check out [![][g1]][g2].

## Credit

- Vim-clap is initially enlightened by [snails](https://github.com/manateelazycat/snails).
- Some providers' idea and code are borrowed from [fzf.vim](https://github.com/junegunn/fzf.vim).
- The built-in fzy python implementation is based on [sweep.py](https://github.com/aslpavel/sweep.py).

## [License](LICENSE)

MIT
