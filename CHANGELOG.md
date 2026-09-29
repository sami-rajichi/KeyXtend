# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- The Rust workspace and its lint rules (`rustfmt.toml`, `clippy.toml`, `deny.toml`).
- `cargo xtask tidy`, `dco`, `licences` and the `dist` stub.
- CI on Windows and Ubuntu, with Dependabot for Cargo and GitHub Actions.
- The licence, code of conduct and GitHub community templates.
- ADR-0012, recording the Inno Setup 7 installer decision.
- A README that describes the product, who it is for and how it differs.
- `cargo xtask dev-cert`, `dev-install` and `check-uiaccess`, for signed uiAccess test builds.
- ADR-0013: Qt 6 Quick with a Rust core draws every window, with the toolkit test round's results.
- ADR-0014 and `docs/spike-move-map.md`: the tested test-round code moves into the product, phase by phase.
- The kernel and module system (P2): module lifecycle, dependency order, event bus, capability-gated services, a settings store that repairs itself, and a redacted local log (ADR-0015).
- The Windows test tools `kx-gates` and `kx-target-window`, moved in from the toolkit test round.

### Removed
- The Slint and Tauri prototype keyboards from the toolkit test round.
