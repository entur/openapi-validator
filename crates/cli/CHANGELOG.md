# Changelog

All notable changes to OpenAPI Validator will be documented in this file.

## [0.7.1](https://github.com/entur/openapi-validator/compare/cli-v0.7.0...cli-v0.7.1) (2026-07-02)


### Bug Fixes

* **cli:** point install script, action, and docs at the merged repo ([6d7ced5](https://github.com/entur/openapi-validator/commit/6d7ced5c69bdd46cb88e2ee51f09a5f5d424dbc4))
* **cli:** warn instead of failing on unknown generators before running ([458c9aa](https://github.com/entur/openapi-validator/commit/458c9aa34b41587eebf9c5554fd89c066b43ba68))

## v0.7.0

- Support validating specs from a URL (`oav validate --spec https://...`)
- Support JSON spec files alongside YAML
- Show discovered spec file path and ask for confirmation during `oav init`
- Streamline README and extract GitHub Action docs

## v0.6.1

- Fix custom generator commands bypassing image entrypoints due to `sh -c` wrapping
- Use POSIX shell-word splitting (`shell-words`) for correct handling of quoted arguments
- Add example configurations with integration tests

## v0.6.0

- Add custom generator support via `custom_generators_dir` config and YAML definitions
- Warn about modified generator configs during `oav clean`
- Replace deprecated `serde_yaml` with `yaml_serde`
- Relax dependency version pins

## v0.5.0

- Add JSON output (`--output json`) for machine-readable results
- Add parallel generation and compilation (`-j` / `--jobs`)
- Add interactive init wizard when `--spec` is omitted
- Add `oav clean --nuke` to remove all oav artifacts
- Add `oav completions install` / `uninstall` for shell completion management
- Add `oav agent install` / `uninstall` for Claude Code skill integration
- Add `oav config validate` and `oav config list-generators` subcommands
- Add configurable docker timeout (`docker_timeout`) and search depth (`search_depth`)

## v0.4.0

- Add reusable GitHub Actions for setup and validation
- Add test coverage, exit codes, and config validation
- Add docker timeouts and configurable search depth
- Centralize generator definitions
- Add Spectral linting as default linter (replacing Redocly)
- Add input validation for `config set` and CLI args
- Improve terminal output handling and respect `-q`/`-v` flags

## v0.3.0

- Add Homebrew formula and curl installer
- Add release workflow with binary builds

## v0.2.0

- Remove `openapi-validator` binary; `oav` is now the sole entrypoint

## v0.1.0

- Initial Rust CLI: lint, generate, compile pipeline with Docker
- `.oavc` config file and `.oav/` output directory
