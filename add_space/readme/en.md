# add_space: Perfecting the mixed-script reading experience

## Table of Contents

- [Significance](#significance)
- [Installation](#installation)
- [Usage](#usage)
  - [Command Line](#command-line)
  - [LazyVim Configuration](#lazyvim-configuration)
  - [Examples](#examples)
- [API Reference](#api-reference)
  - [`State` enum](#state-enum)
  - [`state(c: char) -> State`](#statec-char---state)
  - [`add_space(txt: impl AsRef<str>) -> String`](#add_spacetxt-impl-asrefstr---string)
- [Design Philosophy](#design-philosophy)
- [Technology Stack](#technology-stack)
- [File Structure](#file-structure)
- [Historical Anecdote](#historical-anecdote)

## Significance

In mixed Chinese and English text, adding spaces between Chinese and English words significantly enhances readability. This tool automates the process, saving time and improving the reading experience.

## Installation

```bash
cargo install add_space
```

## Usage

### Command Line

Process a file and print the result to standard output:

```bash
add_space <file_path>
```

Process a file and write the changes back to the file:

```bash
add_space <file_path> --write
```

Use with standard input/output streams:

```bash
echo "Hello世界" | add_space
```

### LazyVim Configuration

If you use [lazyvim](https://github.com/LazyVim/LazyVim), you can edit `~/.config/nvim/lua/config/autocmds.lua` and add the following configuration to automatically add spaces on file save:

```lua
vim.api.nvim_create_autocmd("BufWritePost", {
  group = vim.api.nvim_create_augroup("add_space_on_save", { clear = true }),
  pattern = { "*.txt", "*.md", "*.mdt" },
  callback = function()
    local file_path = vim.fn.expand("%:p")
    local command = "add_space -w " .. vim.fn.shellescape(file_path)
    vim.fn.system(command)
    vim.cmd("edit")
  end,
})
```

### Examples

| Original Text | Processed Text |
| --- | --- |
| `OAuth 2.0鉴权用户只能查询到通过OAuth 2.0鉴权创建的会议` | `OAuth 2.0 鉴权用户只能查询到通过 OAuth 2.0 鉴权创建的会议` |
| `当你凝视着bug，bug也凝视着你` | `当你凝视着 bug，bug 也凝视着你` |
| `中文English中文` | `中文 English 中文` |
| `使用了Python的print()函数打印"你好,世界"` | `使用了 Python 的 print() 函数打印"你好,世界"` |

## API Reference

The core logic is exposed in the `lib.rs` library.

### `State` enum

Represents the classification of a character.

- `Space`: Whitespace characters.
- `Char`: CJK characters (Han, Hiragana, Katakana, etc.).
- `Letter`: English letters, numbers, and certain symbols.
- `Punctuation`: Punctuation marks.

### `state(c: char) -> State`

This function takes a character and returns its corresponding `State`. It uses the `unicode-script` crate to identify the script of the character.

### `add_space(txt: impl AsRef<str>) -> String`

This is the main function that performs the spacing logic. It iterates through the input text, determines the state of each character using the `state` function, and inserts a space when a `Char` type is followed by a `Letter` type or vice versa.

## Design Philosophy

The program's entry point is in `main.rs`, which handles command-line argument parsing and file I/O using the `clap` crate. The core logic resides in `lib.rs`.

The `add_space` function iterates through the text, using a state machine to determine whether a space is needed. It calls the `state` function to classify each character into one of four types: `Char` (Chinese, Japanese, etc.), `Letter` (English, numbers), `Space`, or `Punctuation`. A space is inserted when a `Char` type is followed by a `Letter` type or vice versa, ensuring proper spacing.

## Technology Stack

- **Rust**: The programming language used for this project.
- **clap**: A library for parsing command-line arguments.
- **unicode-script**: A library for determining the script of a Unicode character.

## File Structure

```
.
├── Cargo.toml      # Project configuration file
├── src
│   ├── lib.rs      # Core logic for adding spaces
│   └── main.rs     # Command-line interface
└── tests
    └── main.rs     # Test cases
```

## Historical Anecdote

The practice of adding spaces between Chinese and English text, often called "pangu spacing," is a convention that emerged with the rise of digital typography. While traditional Chinese text has no spaces, the inclusion of English words and letters necessitated a new approach to maintain readability. Early digital systems and search engines struggled to parse mixed-script text without clear separators. Although modern technology has largely overcome these limitations, the convention persists for aesthetic reasons and to improve the reading experience. This has led to the development of numerous tools and scripts, like this one, dedicated to automating the process.
