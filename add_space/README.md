[English](#en) | [中文](#zh)

---

<a id="en"></a>

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

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [x.com/Js0Site](https://x.com/Js0Site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# add_space: 完美混合脚本阅读体验

## 目录

- [项目意义](#项目意义)
- [安装](#安装)
- [使用演示](#使用演示)
  - [命令行](#命令行)
  - [LazyVim 配置](#lazyvim-配置)
  - [示例](#示例)
- [API 参考](#api-参考)
  - [`State` 枚举](#state-枚举)
  - [`state(c: char) -> State`](#statec-char---state)
  - [`add_space(txt: impl AsRef<str>) -> String`](#add_spacetxt-impl-asrefstr---string)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [文件结构](#文件结构)
- [历史小故事](#历史小故事)

## 项目意义

在混合中英文的文本中，中英文之间添加空格能显著提升阅读体验。此工具可自动化该过程，节省时间。

## 安装

```bash
cargo install add_space
```

## 使用演示

### 命令行

处理文件并将结果打印到标准输出：

```bash
add_space <file_path>
```

处理文件并将更改写回文件：

```bash
add_space <file_path> --write
```

与标准输入/输出流一起使用：

```bash
echo "Hello世界" | add_space
```

### LazyVim 配置

如果你使用 [lazyvim](https://github.com/LazyVim/LazyVim) 的话，可以编辑 `~/.config/nvim/lua/config/autocmds.lua`

加入如下的配置，让文件在保存的时候自动添加空格:

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

### 示例

| 原始文本 | 处理后文本 |
| --- | --- |
| `OAuth 2.0鉴权用户只能查询到通过OAuth 2.0鉴权创建的会议` | `OAuth 2.0 鉴权用户只能查询到通过 OAuth 2.0 鉴权创建的会议` |
| `当你凝视着bug，bug也凝视着你` | `当你凝视着 bug，bug 也凝视着你` |
| `中文English中文` | `中文 English 中文` |
| `使用了Python的print()函数打印"你好,世界"` | `使用了 Python 的 print() 函数打印"你好,世界"` |

## API 参考

核心逻辑在 `lib.rs` 库中提供。

### `State` 枚举

代表字符的分类。

- `Space`: 空白字符。
- `Char`: 中日韩等 CJK 字符。
- `Letter`: 英文字母、数字和某些符号。
- `Punctuation`: 标点符号。

### `state(c: char) -> State`

此函数接收一个字符并返回其对应的 `State`。它使用 `unicode-script` 包来识别字符的脚本。

### `add_space(txt: impl AsRef<str>) -> String`

这是执行间距逻辑的主要函数。它遍历输入文本，使用 `state` 函数确定每个字符的状态，并在 `Char` 类型后跟 `Letter` 类型或反之时插入空格。

## 设计思路

程序入口位于 `main.rs`，负责处理命令行参数解析和文件 I/O。核心逻辑位于 `lib.rs`。

`add_space` 函数遍历文本，通过状态机确定是否需要添加空格。它调用 `state` 函数将每个字符分为四种类型之一：`Char`（中文、日文等）、`Letter`（英文、数字）、`Space` 或 `Punctuation`。当 `Char` 类型后跟 `Letter` 类型或反之时，会插入空格，以确保适当的间距。

## 技术堆栈

- **Rust**: 项目使用的编程语言。
- **clap**: 解析命令行参数的库。
- **unicode-script**: 确定 Unicode 字符脚本的库。

## 文件结构

```
.
├── Cargo.toml      # 项目配置文件
├── src
│   ├── lib.rs      # 添加空格的核心逻辑
│   └── main.rs     # 命令行界面
└── tests
    └── main.rs     # 测试用例
```

## 历史小故事

在中英文之间添加空格的做法，常被称为“盘古之白”，是随着数字排版的兴起而出现的惯例。传统中文文本没有空格，但英文单词和字母的加入需要新方法来保持可读性。早期数字系统和搜索引擎在没有明确分隔符的情况下难以解析混合脚本的文本。尽管现代技术已在很大程度上克服这些限制，但出于美学原因和改善阅读体验，这种惯例仍然存在。这催生了许多自动化此过程的工具和脚本，如此项目。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [x.com/Js0Site](https://x.com/Js0Site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
