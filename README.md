# openapi-validator

Tools for working with OpenAPI specs locally. Three interfaces live in this workspace:

- [`oav`](crates/cli/README.md) — CLI for linting, generating, and compiling specs.
- [TUI](crates/tui/README.md): interactive mode of `oav` for exploring, linting, and fixing specs. Run `oav` with no subcommand.
- [OpenAPI Validator](crates/gui/README.md) — desktop GUI for linting, validating, and editing specs.

A shared library (`oav-lib`) holds logic used by all of them. New interfaces (daemon, etc.) can sit alongside as additional crates.

## Layout

```
crates/
  cli/   oav-cli  → binary: oav
  tui/   oav-tui  (TUI library, bundled into oav)
  gui/   oav-gui  → app: OpenAPI Validator (Tauri)
  lib/   oav-lib  (shared)
```

## Build

```bash
cargo build --workspace
cargo build --release -p oav-cli       # just the CLI
cargo build --release -p oav-cli --no-default-features  # CLI without the TUI
```

## Install

See each crate's README for Homebrew taps and shell installers:

- CLI: [crates/cli/README.md](crates/cli/README.md)
- GUI: [crates/gui/README.md](crates/gui/README.md)

## License

EUPL-1.2. See [LICENSE.md](LICENSE.md).
