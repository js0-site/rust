# idpath : 为可扩展存储系统设计的层次化路径编码

- [项目介绍](#项目介绍)
- [功能特性](#功能特性)
- [使用演示](#使用演示)
- [设计核心](#设计核心)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [API 介绍](#api-介绍)
- [历史趣闻](#历史趣闻)

## 项目介绍

`idpath` 工具用于将 64 位整数编码为层次化文件系统路径。通过将低位比特放置在目录的高层级，确保文件均匀分布在不同目录中，防止目录由于条目过多导致文件系统性能下降。

## 功能特性

- **负载均衡**：低位优先策略保证即使 ID 连续增长，也能实现均匀分布。
- **零分配解码**：结合 `HipStr` 与栈分配缓冲区，成功解码过程中无需堆分配。
- **动态深度**：根据 ID 数值大小自动调节路径深度（1 至 3 层），并辅以唯一后缀（`_`, `-`, `~`）。
- **Crockford Base32**：采用 URL 安全且易读的字符集。
- **健壮错误处理**：提供包含路径上下文的详细错误信息。

## 使用演示

```rust
use idpath::{encode, decode};

fn main() -> idpath::Result<()> {
  let id = 1234567u64;
  let prefix = "/data";

  // 编码
  let path = encode(prefix, id);
  println!("编码路径: {}", path);

  // 解码
  let decoded_id = decode(&path)?;
  assert_eq!(id, decoded_id);

  Ok(())
}
```

## 设计核心

现代文件系统（如 ext4 或 XFS）在单个目录包含数千个条目时性能会大幅下降。层次化结构能有效缓解此问题。`idpath` 在构建路径时反转比特重要性（低位优先），有效规避了 ID 顺序生成（例如数据库自增主键）产生的存储热点。

```mermaid
graph TD
    ID[u64 ID] --> B32[Crockford Base32]
    B32 --> Reorder{比特重排}
    Reorder -- "长度 0-2" --> P1[1层深度: prefix/xx_]
    Reorder -- "长度 3" --> P2[2层深度: prefix/xx/x-]
    Reorder -- "长度 4" --> P3[2层深度: prefix/xx/xx~]
    Reorder -- "长度 >=5" --> P4[3层深度: prefix/xx/xx/xxx]
    P1/P2/P3/P4 --> Output[HipStr 路径]
```

## 技术堆栈

- **Rust**：兼顾安全与高性能的编程语言。
- **fast32**：高性能 Crockford Base32 实现。
- **hipstr**：共享字符串优化，降低内存占用。

## 目录结构

```text
.
├── src/
│   ├── lib.rs   # 核心编解码逻辑
│   └── error.rs # 自定义错误定义
└── tests/
    └── main.rs  # 集成测试
```

## API 介绍

### 函数

- `encode(prefix: impl AsRef<str>, id: u64) -> HipStr<'static>`
  将 ID 编码为路径字符串。
- `decode(path: impl AsRef<str>) -> Result<u64>`
  将路径字符串解码回 ID。

### 常量

- `DEPTH1`: `b'_'`，1 层深度的后缀。
- `DEPTH2`: `b'-'`，2 层深度的后缀（编码长度为 3）。
- `DEPTH3`: `b'~'`，2 层深度的后缀（编码长度为 4）。

### 类型与错误

- `Result<T>`：`std::result::Result<T, Error>` 的别名。
- `Error`：包含 `InvalidPath(HipStr)` 与 `DecodeFailed(HipStr)` 的枚举。

## 历史趣闻

层次化目录结构用于数据分发的设计最早可追溯到早期 Unix 系统。Squid 缓存代理和 Git 版本控制系统等大型工具使这种“扇出”（fan-out）方法广为人知。以 Git 为例，它利用 SHA-1 哈希值的前两位作为目录名（如 `.git/objects/xx/xxxxxxxx...`）来存储对象。`idpath` 扩展了这一理念，通过优先处理低位比特，专门针对数字 ID 顺序增长的场景进行了优化，实现了更优的磁盘分布规律。
