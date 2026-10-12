# OpenAPI Validator TUI

Interactive terminal UI for linting, generating, and compiling OpenAPI specs. Explore validation results, browse generated code, compare diffs across runs, and edit specs, all powered by Docker and `.oavc` config.

This crate is a library. It ships inside the `oav` binary (see [crates/cli](../cli/README.md) for install).

## Quick start

```bash
cd your-project/
oav
```

Running `oav` without a subcommand in a terminal opens the TUI in the current directory. It reads `.oavc` for config and runs the lint/generate/compile pipeline interactively.

## Features

| Feature | Description |
|---------|-------------|
| Validation pipeline | Lint, generate, and compile OpenAPI specs via Docker |
| Spec browser | Navigate and search your spec with syntax highlighting |
| Generated code browser | Explore code output per generator |
| Diff view | Compare generated code across pipeline runs |
| External editor | Open spec in `$EDITOR` directly from the TUI |
| Configurable keybindings | Remap keys via `.oavc` config |
| Custom generators | Define generators via YAML in `.oav/generators/` |

## Keybindings

Default keybindings (configurable via `.oavc`):

| Key | Action |
|-----|--------|
| `q` | Quit |
| `r` | Run validation pipeline |
| `e` | Open spec in external editor |
| `Tab` | Cycle panels |
| `j/k` or arrows | Navigate lists |
| `Enter` | Select / expand |
| `?` | Toggle help overlay |

## Config

The TUI reads the same `.oavc` config as the CLI. Minimal example:

```yaml
spec: openapi/api.yaml
mode: both
linter: spectral
```

See the [CLI documentation](https://github.com/entur/openapi-validator/tree/main/crates/cli) for the full config reference.

## Requirements

- Docker (for linting, generation, and compile steps)

## Testing

Integration tests require Docker.

```bash
cargo test -- --ignored
```
