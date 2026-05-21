# sendptr : 方便跨线程用裸指针

## 目录

- [功能特性](#功能特性)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [API 接口](#api-接口)
- [历史背景](#历史背景)

## 功能特性

- **线程安全包装**：为裸指针实现 `Send` 和 `Sync`，使其能在线程间传递。
- **零开销**：对 `*const T` 的极简封装，无运行时性能损耗。
- **便捷访问**：实现 `Deref` trait，方便访问底层数据（需 unsafe）。
- **接口简单**：仅提供核心必要的方法，保持轻量。

## 使用演示

以下代码演示了如何使用 `sendptr` 将裸指针传递给另一个线程：

```rust
use sendptr::SendPtr;
use std::thread;

fn main() {
    let data = Box::new(42);
    let ptr = Box::into_raw(data);

    // 包装裸指针以使其具备 Send 特性
    let send_ptr = SendPtr::new(ptr as *const i32);

    let handle = thread::spawn(move || {
        // 在另一个线程中访问指针
        let val = unsafe { *send_ptr };
        println!("Value from thread: {}", val);

        // 清理内存（如果需要）
        unsafe {
            let _ = Box::from_raw(send_ptr.get() as *mut i32);
        }
    });

    handle.join().unwrap();
}
```

## 设计思路

本库的核心设计理念是为 Rust 严格的并发模型提供针对裸指针的“逃生舱”。

- **包装结构**：`SendPtr<T>` 内部封装了 `*const T`。
- **标记 Trait**：显式实现 `unsafe impl Send` 和 `unsafe impl Sync`，向编译器断言该指针跨线程移动是安全的。
- **解引用**：实现 `Deref` 以支持 `*ptr` 语法，但由于裸指针的特性，访问数据依然需要 `unsafe` 块。

## 技术堆栈

- **Rust**：项目完全使用 Rust 编写。
- **标准库**：仅依赖 `std` 提供的核心指针和 trait 功能。

## 目录结构

```
.
├── Cargo.toml      # 项目配置文件
├── readme          # 文档目录
│   ├── en.md       # 英文说明文档
│   └── zh.md       # 中文说明文档
├── src             # 源代码目录
│   └── lib.rs      # 核心库文件
└── tests           # 测试与示例
    └── main.rs     # 集成测试
```

## API 接口

`lib.rs` 导出了以下主要组件：

### `struct SendPtr<T>`

裸指针 `*const T` 的包装结构体，实现了 `Send` 和 `Sync`。

#### `fn new(ptr: *const T) -> Self`

创建 SendPtr 实例，包装给定的裸指针。

#### `fn get(&self) -> *const T`

获取底层的裸指针。

#### `impl Deref for SendPtr<T>`

允许对 `SendPtr` 进行解引用以访问底层的 `T`。请注意，解引用裸指针始终是 `unsafe` 操作。

## 历史背景

Rust 的所有权和并发模型（常被称为“无畏并发”）在很大程度上依赖于 `Send` 和 `Sync` trait。这两个 trait 就像守门员，确保只有安全的数据才能在线程间共享或移动。裸指针本质上是内存管理的“西部荒野”——它们不提供有效性或线程安全的保证，因此 Rust 编译器保守地将它们标记为 `!Send` 和 `!Sync`。

然而，在底层系统编程、与 FFI 交互或构建自定义同步原语时，开发者往往清楚某个特定指针实际上是可以安全移动的，即使编译器无法证明这一点。`sendptr` 的诞生就是为了在这些场景中充当值得信赖的信使。它作为手动覆盖机制，就像“警长”，为裸指针担保，允许它绕过编译器的封锁，在线程间自由穿梭。这遵循了底层 crate 中常见的设计模式，为特定的高级用例提供专注且极简的工具。
