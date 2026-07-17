# openapi-validator

Tools for working with OpenAPI specs locally. Three interfaces live in this workspace:

- [`oav`](crates/cli/README.md) — CLI for linting, generating, and compiling specs.
- [`lazyoav`](crates/tui/README.md) — TUI for exploring, linting, and fixing specs interactively.
- [OpenAPI Validator](crates/gui/README.md) — desktop GUI for linting, validating, and editing specs.

A shared library (`oav-lib`) holds logic used by all of them. New interfaces (daemon, etc.) can sit alongside as additional crates.

## Layout

```
crates/
  cli/   oav-cli  → binary: oav
  tui/   oav-tui  → binary: lazyoav
  gui/   oav-gui  → app: OpenAPI Validator (Tauri)
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
- GUI: [crates/gui/README.md](crates/gui/README.md)

## License

EUPL-1.2. See [LICENSE.md](LICENSE.md).
