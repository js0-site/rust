# clap_args : Clap 轻量封装，自动处理版本与帮助

`clap` 的简化封装，通过自动处理版本和帮助参数，简化命令行参数解析流程。

## 目录

- [特性](#特性)
- [使用](#使用)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [API 参考](#api-参考)
- [历史](#历史)

## 特性

- **自动版本管理**: 直接读取 `Cargo.toml` 中的版本号。
- **详细信息**: 通过 `--vv` 提供平台和目标信息。
- **内置帮助**: 自动处理 `-h` 和 `--help` 参数。
- **宏支持**: `parse!` 宏简化初始化过程。

## 使用

添加到 `Cargo.toml`:

```toml
[dependencies]
clap_args = { version = "*", features = ["macro"] }
```

### 示例

```rust
use clap_args::{arg, ArgAction};

fn main() -> aok::Result<()> {
  if let Some(matches) = clap_args::parse!(|cmd| {
    cmd
      .arg(arg!(-b --bind [BIND] "http 代理绑定地址").default_value("0.0.0.0:15080"))
      .arg(arg!(-p --port <PORT> "监听端口"))
      .arg(arg!(-d --debug "启用调试模式").action(ArgAction::SetTrue))
  }) {
    let bind: &String = matches.get_one("bind").unwrap();
    println!("绑定: {bind}");

    if let Some(port) = matches.get_one::<String>("port") {
      println!("端口: {port}");
    }

    if matches.get_flag("debug") {
      println!("启用调试模式");
    }
  }
  Ok(())
}
```

## 设计思路

核心设计目标是减少通用 CLI 任务的样板代码。

**调用流程**:

1.  **宏调用**: 用户调用 `parse!` 并传入闭包。
2.  **初始化**: `parse` 函数初始化 `clap::Command`，配置默认参数（`-v`, `--vv`, `-h`）。
3.  **用户配置**: 执行用户闭包，配置额外参数。
4.  **执行**: `clap` 解析参数。
5.  **处理**:
    - 若存在版本或帮助参数，立即处理并返回 `None`。
    - 否则，返回 `Some(ArgMatches)` 供用户处理。

## 技术堆栈

- **Rust**: 核心语言。
- **Clap**: 底层参数解析器。
- **const_str**: 编译时字符串处理。
- **current_platform**: 获取平台信息。

## 目录结构

```
.
├── Cargo.toml      # 项目配置
├── examples/       # 使用示例
├── readme/         # 文档
├── src/            # 源代码
│   └── lib.rs      # 主库文件
└── test.sh         # 测试脚本
```

## API 参考

### `parse!` 宏

使用当前包名和版本初始化解析器。

```rust
clap_args::parse!(|cmd| { ... })
```

### `parse` 函数

宏调用的底层函数。

```rust
pub fn parse(
  project: impl Into<String>,
  ver: impl Borrow<[u64; 3]>,
  cmd_build: impl FnOnce(Command) -> Command,
) -> Option<ArgMatches>
```

### 导出

- `arg!`: 从 `clap` 重新导出，用于定义参数。
- `ArgAction`: 从 `clap` 重新导出，用于定义参数动作。

## 历史

命令行参数的概念可以追溯到 20 世纪 70 年代 Unix 的早期。`argv`（参数向量）约定允许程序在运行时动态接收输入，这是相对于硬编码参数的重大飞跃。几十年来，解析库从简单的循环检查演变为像 Rust 中的 `clap` 这样复杂的框架，提供类型安全、自动生成帮助和子命令支持，反映了现代 CLI 工具日益增长的复杂性和能力。
