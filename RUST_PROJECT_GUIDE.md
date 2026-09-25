# Rust Project Structure Guide for This `grep`

This guide explains the Rust concepts needed to build this project, using C++ comparisons where they help.

## 1. The four names that are easy to mix up

### Package

A **package** is the project managed by Cargo. This repository is one package because it has one `Cargo.toml`:

```text
grep/
  Cargo.toml
  src/
```

A package can contain one library crate and one or more binary crates.

C++ comparison: a package is roughly the whole CMake project or repository target group. It is not the same thing as a library.

### Crate

A **crate** is one compilation unit. It is the unit Rust compiles and the unit that has its own crate-level namespace.

This package can have:

- a library crate, whose root is `src/lib.rs`;
- a binary crate, whose root is `src/main.rs`.

The library and binary are separate crates, even though they live in the same package. The binary can depend on the library crate by package name.

C++ comparison: a crate is closer to a library or executable target than to a header file. A crate is compiled as a whole, including its module tree.

### Module

A **module** is a namespace inside a crate. Modules organize code and control visibility.

A module is declared with `mod`:

```rust
mod cli;
```

Rust then looks for one of these files:

```text
src/cli.rs
src/cli/mod.rs
```

For this project, `src/cli/mod.rs` is the natural choice because `cli` contains several files.

### Item

Functions, structs, enums, constants, traits, and type aliases are **items** inside modules.

```rust
pub(crate) fn run() {}
struct Config;
enum ExitCode {}
```

## 2. What `Cargo.toml` controls

The current manifest is:

```toml
[package]
name = "grep"
version = "0.1.0"
edition = "2024"

[dependencies]
```

Important fields:

- `name`: the package name and, normally, the library crate name used in Rust code;
- `version`: package version metadata;
- `edition`: Rust language rules and defaults. This project uses the 2024 edition;
- `[dependencies]`: external crates downloaded and compiled by Cargo.

Useful commands:

```text
cargo check       # Fast compile check; does not create a final executable.
cargo build       # Compile the project.
cargo run -- ...  # Build and run the binary; arguments after -- go to grep.
cargo test        # Compile and run tests.
cargo fmt         # Format Rust source.
cargo clippy      # Run extra lints, when installed.
```

`target/` contains build artifacts. It is generated output, not source code.

## 3. The crate roots in this project

The two conventional roots are:

```text
src/lib.rs   # root of the library crate
src/main.rs  # root of the binary crate
```

A library crate should contain reusable application logic. A binary crate should be thin: collect process arguments, call the library, and turn the result into an operating-system exit code.

The intended relationship is:

```rust
// src/main.rs
use grep::run;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let exit_code = run(&args);
    std::process::exit(exit_code.as_i32());
}
```

```rust
// src/lib.rs
mod exit;

pub fn run(args: &[String]) -> exit::ExitCode {
    // Parse, search, format output, and return the final status.
    let _ = args;
    exit::ExitCode::Success
}
```

The exact API can change, but the boundary is useful: tests can call `grep::run` without spawning a process.

### Important current-state note

The current `main.rs` contains `mod lib;`. That makes `src/lib.rs` a private module nested inside the binary crate; it does **not** import the separately compiled library crate. In a package with both `src/lib.rs` and `src/main.rs`, prefer `use grep::...` in `main.rs` and remove `mod lib;`.

## 4. How folders become modules

A likely module tree for this project is:

```text
src/lib.rs
  ├── cli        -> src/cli/mod.rs
  │     ├── args -> src/cli/args.rs
  │     ├── config -> src/cli/config.rs
  │     └── usage -> src/cli/usage.rs
  ├── input      -> src/input/mod.rs
  ├── output     -> src/output/mod.rs
  ├── walk       -> src/walk/mod.rs
  ├── errors     -> src/errors.rs
  └── exit       -> src/exit.rs
```

The parent must declare a child module before the child is part of the crate:

```rust
// src/lib.rs
mod cli;
mod errors;
mod exit;
mod input;
mod output;
mod walk;
```

Then `src/cli/mod.rs` declares its children:

```rust
mod args;
mod config;
mod usage;
```

Declaring a module does two things:

1. it tells the compiler which file belongs to the module;
2. it creates a namespace named after the module.

Rust does not automatically compile every `.rs` file in a directory. A file becomes part of the crate only through this module declaration chain.

## 5. Visibility: private, `pub`, and `pub(crate)`

Rust items are private by default. Visibility is deliberately explicit.

```rust
fn private_helper() {}

pub fn public_api() {}

pub(crate) fn crate_only_api() {}
```

### Private (no modifier)

An item can be used by its defining module and, depending on the item and nesting, its descendants. It is not part of the public interface outside that module.

Use this for implementation details.

### `pub`

`pub` exposes an item outside its module, but the containing modules on the path must also be reachable.

```rust
// src/lib.rs
pub mod cli;

// src/cli/mod.rs
pub mod config;
```

This makes `grep::cli::config` reachable from an external crate. Do not mark everything public: public APIs are contracts that make future changes harder.

### `pub(crate)`

`pub(crate)` exposes an item anywhere inside the current crate, but not to users of the crate.

```rust
// src/lib.rs
pub(crate) fn run_internal() {}
```

This is often the right visibility for helpers shared by `lib.rs`, unit tests, and sibling modules.

C++ comparison:

- private Rust item: implementation detail, similar to a private class member or an unexported symbol;
- `pub(crate)`: visible across this crate, similar to an internal project-wide interface;
- `pub`: exported API, similar to a public header/library interface.

`pub(crate)` is not a textual include trick. Rust still checks types, borrowing, and visibility at compile time.

### Other useful visibility forms

- `pub(super)`: visible to the parent module;
- `pub(in path::to::module)`: visible within a specific ancestor path;
- `pub(crate) use ...`: re-export a name for internal crate users.

## 6. `use`, paths, and re-exports

A path names an item:

```rust
crate::exit::ExitCode       // from the current crate root
self::helper                 // from the current module
super::config::Config        // from the parent module
```

`use` creates a shorter local name:

```rust
use crate::exit::ExitCode;

fn run() -> ExitCode {
    ExitCode::Success
}
```

A `use` statement does not change visibility. To expose a name from another module, use a re-export:

```rust
pub use exit::ExitCode;
```

Then callers can use `grep::ExitCode` instead of `grep::exit::ExitCode`.

Prefer a small, intentional public surface in `lib.rs`:

```rust
mod cli;
mod exit;

pub use exit::ExitCode;
pub fn run(args: &[String]) -> ExitCode { /* ... */ }
```

## 7. Rust equivalents of common C++ concepts

### Headers and source files

Rust has no normal header/source split. A module declaration identifies the source file, and the compiler sees the complete crate. Function signatures do not need to be duplicated in a header.

The visibility modifier replaces much of the “what is exported?” role of a header.

### Namespaces

Rust modules are namespaces:

```rust
crate::input::LineReader
```

There is no need for a separate namespace declaration plus namespace-closing syntax.

### Classes

Rust has no single `class` construct. Use:

- `struct` for data;
- `impl` for methods and associated functions;
- `trait` for shared behavior/interfaces;
- `enum` for a value that can have one of several variants.

```rust
pub struct Config {
    pub recursive: bool,
}

impl Config {
    pub fn new() -> Self {
        Self { recursive: false }
    }
}
```

Fields are private by default even when the struct itself is public.

### Constructors

Rust conventionally uses an associated function named `new` instead of a special constructor:

```rust
let config = Config::new();
```

### `nullptr` and errors

Rust uses `Option<T>` instead of nullable pointers and `Result<T, E>` for operations that can fail:

```rust
fn find_pattern() -> Option<String> { /* Some(value) or None */ }
fn read_file() -> Result<Vec<u8>, std::io::Error> { /* ... */ }
```

Handle them with `match`, `if let`, `?`, or combinators. Avoid `unwrap()` in the main application path unless failure is genuinely impossible.

### RAII

Rust’s ownership system drops values automatically when they leave scope. This is similar to RAII, but the compiler verifies who owns each value and whether references are valid.

```rust
let file = std::fs::File::open(path)?;
// file is closed automatically at the end of its scope.
```

## 8. Ownership rules needed for this project

Every value has one owner. Assigning or passing a value may move it:

```rust
let args = String::from("pattern");
let other = args; // args moved; it cannot be used now
```

Borrow when a function only needs to inspect data:

```rust
fn run(args: &[String]) { /* borrow the slice */ }
```

- `&T`: shared borrow; many readers are allowed;
- `&mut T`: exclusive mutable borrow; only one at a time;
- `String`: owned, growable UTF-8 text;
- `&str`: borrowed string slice;
- `Vec<T>`: owned growable array;
- `&[T]`: borrowed slice.

For grep, prefer byte-oriented APIs for file contents because input may not be valid UTF-8:

```rust
fn search_line(pattern: &[u8], line: &[u8]) -> bool {
    line.windows(pattern.len()).any(|window| window == pattern)
}
```

Use `String` and `&str` for command-line text and diagnostics where UTF-8 is appropriate.

## 9. A practical responsibility split for this project

Keep each module focused:

- `cli`: parse arguments and build validated configuration;
- `input`: read stdin/files as bytes and lines;
- `walk`: expand files and directories;
- matcher module: decide whether bytes match a pattern;
- `output`: format matches, filenames, line numbers, colors, and diagnostics;
- `errors`: preserve useful typed failure information;
- `exit`: represent grep's process outcomes.

The matcher should return data, not print it. The output layer should format data, not decide matching semantics. This separation makes unit and integration tests much easier.

## 10. Exit codes are not ordinary booleans

GNU grep has at least three meaningful outcomes:

```text
0 = a selected line was found
1 = no selected line was found
2 = an error occurred
```

Model that explicitly:

```rust
pub enum ExitCode {
    Success,
    NoMatch,
    Error,
}
```

Then convert it to an integer only at the binary boundary. This prevents “no match” from being confused with “the program failed.”

## 11. Tests and visibility

Unit tests inside a source file are compiled as a child module and can test private implementation details:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn finds_a_literal() {
        assert!(true);
    }
}
```

Integration tests in `tests/` compile as separate crates. They can only use the library crate’s public API. This is one reason `pub fn run(...)` or another small public application entry point is useful.

A good progression is:

1. make `cargo check` pass;
2. define `Config`, `ExitCode`, and the public application entry point;
3. add unit tests for parsing and matching;
4. add integration tests for complete command behavior and exit codes;
5. compare output with GNU grep for compatibility cases.

## 12. A small mental model

When adding a file, ask these questions:

1. Which crate owns it: the library or the binary?
2. Which parent module declares it with `mod`?
3. Which names need to cross a module boundary?
4. Should those names be private, `pub(crate)`, or public API?
5. Does the function borrow data, mutate it, or take ownership?
6. Is failure represented by `Option`, `Result`, or an explicit domain enum?

If those answers are clear, the Rust file layout usually follows naturally.
