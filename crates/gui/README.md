# OpenAPI Validator GUI

[![GitHub Release](https://img.shields.io/github/v/release/entur/openapi-validator?filter=gui-*&style=flat-square&label=release)](https://github.com/entur/openapi-validator/releases?q=gui-)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Homebrew](https://img.shields.io/github/v/release/entur/openapi-validator?filter=gui-*&style=flat-square&label=homebrew&color=fbb040)](https://github.com/entur/openapi-validator#install)
[![License](https://img.shields.io/badge/license-EUPL--1.2-blue?style=flat-square)](LICENSE.md)

Desktop app for linting, validating, and editing OpenAPI specs. Run the validation pipeline, browse lint diagnostics, and edit specs in a built-in editor. Built with Tauri and Entur's Linje design system.

## Install

### Homebrew (macOS)

```bash
brew tap entur/openapi-validator https://github.com/entur/openapi-validator
brew install --cask oav-gui
```

The app is not signed or notarized yet, so macOS quarantines it on first launch. Either install with `--no-quarantine` or right-click the app in Finder and choose Open the first time.

### Manual download

Download an installer from the [releases page](https://github.com/entur/openapi-validator/releases?q=gui-): `.dmg` for macOS, `.deb`/`.rpm`/`.AppImage` for Linux.

### Uninstall

| Method   | Command                           |
|----------|-----------------------------------|
| Homebrew | `brew uninstall --cask oav-gui`   |
| Manual   | Move the app to the trash         |

## Development

Requires Node 24+ and the Rust toolchain.

```bash
cd crates/gui
npm ci
npx tauri dev
```

## Build

```bash
cd crates/gui
npx tauri build
```

Bundles land in `target/release/bundle/`.
