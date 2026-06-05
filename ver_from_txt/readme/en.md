# ver_from_txt : Parse version updates from DNS TXT records

<!-- toc -->

## Introduction

`ver_from_txt` is a Rust library designed to parse version update information published in DNS TXT records. It supports decoding base64 encoded version numbers and parsing download URLs, including automatic expansion of GitHub release URLs. This enables applications to efficiently check for updates via DNS protocol.

## Usage

```rust
use aok::{OK, Void};
use log::info;
use ver_from_txt::ver_from_txt;

#[static_init::constructor(0)]
extern "C" fn _loginit() {
  log_init::init();
}

#[test]
fn test() -> Void {
  let txt = "AAEp;Gup51/v;up[0,2~3].u-01.eu.org;yutk.eu.org";

  let r = ver_from_txt("i18", &[0, 0, 1], txt)?;
  info!("{:?}", r);
  OK
}
```

Output:

```
Some(VerUrlLi {
  ver: Ver(0.1.41),
  url_li: [
    "https://github.com/up51/v/releases/download/i18-0.1.41",
    "https://up0.u-01.eu.org/i18/0.1.41",
    "https://up2.u-01.eu.org/i18/0.1.41",
    "https://up3.u-01.eu.org/i18/0.1.41",
    "https://yutk.eu.org/i18/0.1.41"
  ]
})
```

## Design

The library processes the TXT record in the following steps:

1.  **Split & Decode**: The input string is split by `;`. The first part is treated as a Base64 encoded version number.
2.  **Version Comparison**: It decodes the version using `vb` (variable byte) encoding and compares it with the provided current version. If the parsed version is not greater, it returns `None`.
3.  **URL Parsing**: Given the length constraints of DNS TXT records (typically 255 bytes per character string, and keeping total packet size small is preferable), the library uses a compact representation:
    - **GitHub**: Prefixed with `G`, expanded to `https://github.com/...`.
    - **Bracket Expansion**: Supports `[prefix]range` syntax to generate multiple URLs (e.g., for different mirrors).
    - **Standard URL**: Direct URL segments.

## Tech Stack

- **Language**: Rust
- **Dependencies**:
  - `thiserror`: For ergonomic error handling.
  - `base64`: For decoding version strings.
  - `sver`: Semantic versioning support.
  - `vb`: Variable byte decoding.

## Directory Structure

```
.
├── Cargo.toml
├── readme
│   ├── en.md
│   └── zh.md
├── src
│   ├── error.rs    // Error definitions
│   ├── lib.rs      // Core logic
│   └── name_li.rs  // Helper for name list expansion
├── test.sh         // Test runner
└── tests
    └── main.rs     // Integration tests
```

## API Exports

### Structs

- `VerUrlLi`: Contains the new `Ver` and a `Vec<String>` of download URLs.
- `Error`: Enum representing possible errors (Base64 decode, Vb decode, Invalid Text).

### Functions

- `ver_from_txt`: The main entry point.
  ```rust
  pub fn ver_from_txt(project: &str, pre_ver: &[u64; 3], txt: &str) -> Result<Option<VerUrlLi>>
  ```

## History

In the early days of DNS (RFC 1035), TXT records were intended for simple human-readable notes. However, their flexibility soon turned them into the "Swiss Army Knife" of DNS. Anecdotes tell of system administrators using them to store server latitude/longitude "missile coordinates" (University of Edinburgh) or even slicing movies into distributed download links. Today, they are the backbone of modern email security (SPF, DKIM) and domain verification, proving that a simple text field can become a critical infrastructure component.
