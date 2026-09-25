# Project Structure

This project is a Rust reimplementation of GNU grep. The layout below keeps command-line compatibility, search semantics, input handling, and output formatting separate so each area can be tested independently.

## Current crate

```text
.
├── Cargo.toml
├── README.md
├── STRUCTURE.md
├── src
│   ├── main.rs
│   ├── lib.rs
│   ├── cli
│   │   ├── mod.rs
│   │   ├── args.rs
│   │   ├── config.rs
│   │   └── usage.rs
│   ├── matcher
│   │   ├── mod.rs
│   │   ├── literal.rs
│   │   ├── regex.rs
│   │   ├── fixed.rs
│   │   ├── basic.rs
│   │   ├── extended.rs
│   │   └── context.rs
│   ├── input
│   │   ├── mod.rs
│   │   ├── sources.rs
│   │   ├── line_reader.rs
│   │   └── binary.rs
│   ├── output
│   │   ├── mod.rs
│   │   ├── format.rs
│   │   ├── color.rs
│   │   └── writer.rs
│   ├── walk
│   │   ├── mod.rs
│   │   ├── files.rs
│   │   └── recursion.rs
│   ├── errors.rs
│   └── exit.rs
├── tests
│   ├── cli.rs
│   ├── matching.rs
│   ├── files.rs
│   ├── output.rs
│   └── compatibility
│       ├── basic.txt
│       ├── recursive.txt
│       └── binary.txt
└── benches
    └── search.rs
```

## Responsibilities

### `src/main.rs`

The thin executable entry point. It should construct the process-level environment, call the library application, and convert the resulting status into a process exit code. Search behavior should not live here.

### `src/lib.rs`

The reusable crate entry point. It exposes the application pipeline to integration tests and keeps the binary easy to exercise without spawning a child process for every unit test.

### `src/cli/`

Command-line compatibility with GNU grep:

- `args.rs`: parse short and long options, option clusters, operands, and pattern sources.
- `config.rs`: turn parsed arguments into validated immutable runtime configuration.
- `usage.rs`: help text, version text, diagnostics, and option-specific errors.

This layer should preserve GNU grep's precedence rules, aliases, default values, and invalid-option behavior.

### `src/matcher/`

Pattern compilation and line matching. Keep this independent of paths, streams, and terminal output.

- `literal.rs`: ordinary substring search and fast literal paths.
- `regex.rs`: shared regular-expression abstraction.
- `fixed.rs`: `-F` fixed-string matching, including multiple patterns.
- `basic.rs` and `extended.rs`: POSIX basic and extended regular-expression behavior.
- `context.rs`: before/after context and match ranges.

The matcher should return structured match information rather than formatted strings.

### `src/input/`

Reading bytes and lines from stdin, regular files, and special inputs. This layer owns buffering, newline handling, invalid UTF-8 policy, NUL-data detection, and binary-file behavior. Prefer byte-oriented APIs so grep can search arbitrary files correctly.

### `src/output/`

Turn matches into GNU-compatible output. It owns filename prefixes, line and byte numbers, separators, context lines, color, escaping, and writing to stdout or stderr. Formatting must consume structured results from the matcher and input layers.

### `src/walk/`

File and directory traversal for `-r` and `-R`, including symlink policy, hidden or excluded paths when applicable, deterministic error reporting, and the special `-` stdin operand.

### `src/errors.rs`

Typed internal errors and diagnostic context. Keep user-facing wording in the CLI/output boundary, while preserving the original I/O error source for debugging and tests.

### `src/exit.rs`

The three primary grep outcomes should be explicit throughout the application:

- `0`: at least one selected line was found.
- `1`: no selected line was found.
- `2`: an error occurred, such as invalid arguments or an unreadable input.

Do not collapse these outcomes into a boolean.

## Testing strategy

- Unit tests live beside the implementation for small parser, matcher, and formatter rules.
- Integration tests in `tests/` exercise the public application pipeline and process exit behavior.
- Compatibility fixtures compare this implementation with a known GNU grep version for the same input, options, stdout, stderr, and exit status.
- Keep test inputs byte-oriented and include empty files, missing final newlines, invalid UTF-8, NUL bytes, long lines, multiple patterns, stdin, duplicate paths, and unreadable files.
- Add benchmarks only for hot paths such as literal search, fixed-string sets, regular expressions, and large-file scanning.

## Dependency guidance

Keep the core small and choose dependencies for behavior that is difficult to reproduce correctly:

- a maintained argument parser for GNU-compatible option handling, if it can represent the required compatibility rules;
- a regex engine with the required POSIX semantics, or a dedicated POSIX-compatible implementation;
- an error-reporting crate only if it does not obscure stable diagnostics;
- a benchmark framework for `benches/`.

Do not add a dependency merely to wrap straightforward byte scanning or filesystem traversal. Record any compatibility limitation in `README.md` and cover it with a regression test.

## Recommended implementation order

1. Move argument parsing into `src/cli/` and define the runtime configuration.
2. Add byte-oriented single-file and stdin input handling.
3. Implement literal and fixed-string matching.
4. Implement GNU-compatible output and the `0/1/2` exit contract.
5. Add regular-expression modes and pattern-file support.
6. Add multiple files, directory traversal, binary handling, context, color, and remaining options incrementally.
7. Expand differential compatibility tests against GNU grep before declaring an option complete.

The public library boundary should remain small. A useful target is one application function that accepts configuration and input/output handles, while the binary is responsible only for process integration.
