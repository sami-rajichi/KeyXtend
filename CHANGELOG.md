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
- The keyboard core (P3): typing in English, French and Arabic from a no-focus uiAccess window, modifiers, key repeat and act on press, the Native, ET66 and Dolch looks, the bubble and the tray (ADR-0016 to ADR-0026).
- Mouse assist (P4): a still hold right-clicks, Grab drags and selects with clicks only, Shift or Ctrl + click, a ring and badge at the pointer, soft CC0 sounds, and a mouse hook that reinstalls itself (ADR-0027, ADR-0028).
- The scroll pad (P5): arrows and Stop that glide the window last pointed at, slow then fast, with its outline, a badge, stops at the end, a press or a limit, our open list paged, and a lone second click on Win that opens Start (ADR-0029).
- The shortcuts layer and Power key (P6): 37 shortcuts on pages the user arranges, named by meaning and sent by each platform's table, tips that name their keys, and the system's own power dialog (ADR-0031).

### Changed
- The licence: KeyXtend is now view only, all rights reserved, and the app is sold under its own user licence (ADR-0030). Versions published before keep the GPL-3.0-or-later.

### Removed
- The Slint and Tauri prototype keyboards from the toolkit test round.
