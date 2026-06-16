# openapi-validator

Tools for working with OpenAPI specs locally. Two binaries live in this workspace:

- [`oav`](crates/cli/README.md) — CLI for linting, generating, and compiling specs.
- [`lazyoav`](crates/tui/README.md) — TUI for exploring, linting, and fixing specs interactively.

A shared library (`oav-lib`) holds logic used by both. New interfaces (GUI, daemon, etc.) can sit alongside as additional crates.

## Layout

```
crates/
  cli/   oav-cli  → binary: oav
  tui/   oav-tui  → binary: lazyoav
  lib/   oav-lib  (shared)
```

## Build

```bash
cargo build --workspace
cargo build --release -p oav-cli       # just the CLI
cargo build --release -p oav-tui       # just the TUI
```

## Install

See each crate's README for Homebrew taps and shell installers:

- CLI: [crates/cli/README.md](crates/cli/README.md)
- TUI: [crates/tui/README.md](crates/tui/README.md)

## License

EUPL-1.2. See [LICENSE.md](LICENSE.md).
