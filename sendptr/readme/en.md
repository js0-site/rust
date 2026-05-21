# sendptr : Convenient cross-thread raw pointer usage

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API](#api)
- [History](#history)

## Features

- **Thread Safety Wrapper**: Implements `Send` and `Sync` for raw pointers, allowing them to be transferred between threads.
- **Zero Overhead**: Thin wrapper around `*const T` with no runtime performance cost.
- **Easy Access**: Implements `Deref` for convenient access to the underlying data (unsafe).
- **Simple API**: Minimalistic design with just the essential methods.

## Usage

Here is a demonstration of how to use `sendptr` to pass a raw pointer to another thread.

```rust
use sendptr::SendPtr;
use std::thread;

fn main() {
    let data = Box::new(42);
    let ptr = Box::into_raw(data);

    // Wrap the raw pointer to make it Send
    let send_ptr = SendPtr::new(ptr as *const i32);

    let handle = thread::spawn(move || {
        // Access the pointer in another thread
        let val = unsafe { *send_ptr };
        println!("Value from thread: {}", val);

        // Clean up memory (if needed)
        unsafe {
            let _ = Box::from_raw(send_ptr.get() as *mut i32);
        }
    });

    handle.join().unwrap();
}
```

## Design

The core design philosophy is to provide a lightweight escape hatch for Rust's strict concurrency model regarding raw pointers.

- **Wrapper Struct**: `SendPtr<T>` wraps a `*const T`.
- **Marker Traits**: Explicitly implements `unsafe impl Send` and `unsafe impl Sync`, asserting to the compiler that the pointer is safe to move across thread boundaries.
- **Dereference**: Implements `Deref` to allow `*ptr` syntax, though accessing the data remains `unsafe` due to the nature of raw pointers.

## Tech Stack

- **Rust**: The project is written entirely in Rust.
- **Standard Library**: Relies only on `std` for core pointer and trait functionality.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration
├── readme          # Documentation
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
├── src             # Source code
│   └── lib.rs      # Main library file
└── tests           # Tests and examples
    └── main.rs     # Integration tests
```

## API

The library exports the following main components from `lib.rs`:

### `struct SendPtr<T>`

A wrapper struct for a raw pointer `*const T` that implements `Send` and `Sync`.

#### `fn new(ptr: *const T) -> Self`

Creates a new `SendPtr` instance wrapping the given raw pointer.

#### `fn get(&self) -> *const T`

Returns the underlying raw pointer.

#### `impl Deref for SendPtr<T>`

Allows dereferencing the `SendPtr` to access the underlying `T`. Note that dereferencing a raw pointer is always `unsafe`.

## History

Rust's ownership and concurrency model, often summarized as "Fearless Concurrency," relies heavily on the `Send` and `Sync` traits. These traits act as gatekeepers, ensuring that only data safe to be shared or moved between threads can do so. Raw pointers, by their nature, are the "wild west" of memory management—they offer no guarantees of validity or thread safety, and thus, the Rust compiler conservatively marks them as `!Send` and `!Sync`.

However, in low-level systems programming, interacting with FFI, or building custom synchronization primitives, developers often know that a specific pointer is indeed safe to move, even if the compiler cannot prove it. `sendptr` was created to serve as a trusted courier in these scenarios. It acts as a manual override, a "sheriff" that vouches for the raw pointer, allowing it to bypass the compiler's blockade and travel freely between threads. This follows a pattern seen in other low-level crates, providing a focused, minimal tool for a specific advanced use case.
