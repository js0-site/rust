# CIter: Zero-Copy Circular Iterator for Rust

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Usage Examples](#usage-examples)
- [API Reference](#api-reference)
- [Design Philosophy](#design-philosophy)
- [Technical Stack](#technical-stack)
- [Project Structure](#project-structure)
- [Historical Context](#historical-context)

## Overview

CIter is a lightweight, zero-copy circular iterator library for Rust that enables efficient traversal of slices from any starting position. The iterator wraps around the slice boundaries, providing seamless circular access to data with index information without memory allocation or copying.

## Features

- **Zero-Copy Design**: References original slice data without allocation
- **Flexible Starting Position**: Begin iteration from any index within the slice
- **Automatic Wraparound**: Seamlessly continues from the beginning after reaching the end
- **Position Tracking**: Built-in position tracking for current iterator state
- **Index Information**: Returns both index and value for each element
- **Random Start Support**: Optional random starting position with `rand` feature
- **Memory Safe**: Leverages Rust's ownership system for safe memory access
- **Generic Implementation**: Works with any slice type `&[T]`

## Usage Examples

### Basic Circular Iteration

```rust
use citer::CIter;

let data = [1, 2, 3, 4, 5];
let iter = CIter::new(&data, 2); // Start from index 2
let result: Vec<_> = iter.collect();
// Result: [(2, &3), (3, &4), (4, &5), (0, &1), (1, &2)]
```

### Position Tracking

```rust
let data = [10, 20, 30];
let iter = CIter::new(&data, 2);
println!("Current position: {}", iter.pos()); // Output: 1
```

### Random Starting Position

```rust
// Requires "rand" feature
let data = [1, 2, 3, 4, 5];
let iter = CIter::rand(&data);
let result: Vec<_> = iter.collect();
// Result: Random permutation starting from random position with indices
```

### Manual Iteration

```rust
let data = [1, 2, 3];
let mut iter = CIter::new(&data, 1);

assert_eq!(iter.next(), Some((1, &2)));
assert_eq!(iter.next(), Some((2, &3)));
assert_eq!(iter.next(), Some((0, &1)));
assert_eq!(iter.next(), None); // Iterator exhausted
```

### Extracting Values Only

```rust
let data = [1, 2, 3, 4, 5];
let iter = CIter::new(&data, 2);
let values: Vec<&i32> = iter.map(|(_, &value)| value).collect();
// Result: [3, 4, 5, 1, 2]
```

## API Reference

### `CIter<'a, T>`

Main circular iterator struct with lifetime parameter `'a` and generic type `T`.

#### Fields

- `idx: usize` - Current index position
- `li: &'a [T]` - Reference to the slice data
- `ed: usize` - Number of elements already visited

#### Methods

- `new(li: &'a [T], pos: usize) -> Self`
  - Creates new circular iterator starting from specified position
  - Position is used as-is without bounds checking (handled by modulo in iteration)

- `pos(&self) -> usize`
  - Returns current logical position (0-based)
  - Returns `idx - 1` for non-zero indices, `0` for zero index

- `rand(li: &'a [T]) -> Self` (requires `rand` feature)
  - Creates iterator with random starting position
  - Uses thread-local random number generator

#### Iterator Implementation

Implements standard `Iterator` trait:

- `type Item = (usize, &'a T)` - Returns tuple of (index, reference to value)
- `fn next(&mut self) -> Option<Self::Item>`

## Design Philosophy

The library follows a minimalist approach focusing on performance and safety:

```mermaid
graph TD
  A[Slice Input] --> B[CIter::new]
  B --> C[Position Calculation]
  C --> D[Iterator Creation]
  D --> E[next Call]
  E --> F{Elements Visited < Length}
  F -->|Yes| G[Calculate Index with Modulo]
  G --> H[Return (index, value) Tuple]
  F -->|No| I[Return None]
  H --> J[Increment Counters]
  J --> E
  I --> K[Iterator Exhausted]
```

### Core Design Principles

1. **Zero-Copy Architecture**: References original data without duplication
2. **Bounded Iteration**: Guarantees termination after visiting all elements once
3. **Modulo Arithmetic**: Handles position wraparound efficiently
4. **Index Preservation**: Maintains original index information throughout iteration
5. **Lifetime Safety**: Ensures iterator cannot outlive referenced data

### Module Interaction Flow

- `CIter::new()` initializes iterator state
- `Iterator::next()` implements core circular logic with modulo arithmetic
- Position tracking maintains current state without additional allocations
- Optional `rand` feature provides randomized starting positions
- Index information is preserved in the returned tuples

## Technical Stack

- **Language**: Rust 2024 Edition
- **Core Dependencies**:
  - `aok` (0.1.18) - Result handling utilities
- **Optional Dependencies**:
  - `rand` (0.9.2) - Random starting position support
- **Development Dependencies**:
  - `aok` (0.1.18) - Test result handling
  - `log` (0.4.29) - Logging infrastructure
  - `loginit` (0.1.18) - Log initialization
  - `static_init` (1.0.4) - Static initialization
  - `log` (0.1.43) - Structured logging
  - `log_init` (0.1.34) - Local log initialization utilities

## Project Structure

```
citer/
├── src/
│   └── lib.rs          # Core CIter implementation
├── tests/
│   └── main.rs         # Integration tests
├── readme/
│   ├── en.md          # English documentation
│   └── zh.md          # Chinese documentation
├── Cargo.toml         # Project configuration
└── test.sh           # Test execution script
```

### Key Components

- **`CIter` struct**: Main iterator implementation with position and element tracking
- **Iterator trait**: Standard Rust iterator interface returning (index, value) tuples
- **Feature gates**: Optional functionality behind compile-time flags
- **Comprehensive tests**: Unit and integration test coverage

## Historical Context

Circular iterators have deep roots in computer science, log back to early work on circular buffers in the 1960s. The concept gained prominence with Donald Knuth's "The Art of Computer Programming," where circular data structures were explored as fundamental algorithmic building blocks.

In systems programming, circular iterators became essential for implementing ring buffers, round-robin schedulers, and audio processing pipelines. The zero-copy approach pioneered in languages like C found new expression in Rust's ownership system, enabling memory-safe circular iteration without runtime overhead.

CIter's unique approach of preserving index information alongside values reflects modern needs in data processing, where maintaining context about element positions is crucial for algorithms like circular convolution, periodic data analysis, and rotating window operations. The modulo arithmetic approach used in CIter reflects decades of optimization in circular addressing, commonly found in digital signal processing and embedded systems where efficient wraparound behavior is critical for real-time performance.
