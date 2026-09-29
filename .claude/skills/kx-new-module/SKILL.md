---
name: kx-new-module
description: Use when starting a new feature module (kx-mod-*) or a new platform adapter in the KeyXtend project. Creates the crate from the project template, wires it into the workspace, and sets up tests, a security note and docs so the module can be built and tested on its own.
---

# New KeyXtend module

**Before you start:**
- The roadmap phase for this module has an approved plan in `docs/superpowers/plans/`. If not, stop and use `superpowers:writing-plans` first.
- Read the module's section in the design spec, `ARCHITECTURE.md` (invariants) and `docs/adr/`.

## Steps
1. **Create** `crates/kx-mod-<name>/` with:
   - `Cargo.toml`: `version = "0.0.0"`, `publish = false`, `license.workspace = true`. Dependencies: only `kx-module-api` plus pure utility crates (`serde` for the settings). Dev-dependencies: `kx-platform-fake`, `kx-test-support`, `proptest`.
   - `src/lib.rs`: `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, and a `pub struct <Name>Module` that implements `Module`.
   - `src/manifest.rs`: a `static MANIFEST: Manifest` with `id`, `version: env!("CARGO_PKG_VERSION")`, `requires`, `provides`, the **smallest** `capabilities` list that works, and `settings: Some(&SPEC)`.
   - `src/settings.rs`: the settings struct, and `SPEC`, a `SettingsSpec` with `version: 1`, the embedded `defaults.toml`, the `migrations` list, and `validate: validate_as::<T>` (`T` is the settings struct).
     - The struct derives serde with `#[serde(deny_unknown_fields)]`, implements `Settings` (`check` holds the range rules) and has **no `Default` values**.
     - `defaults.toml`, beside `Cargo.toml`, is the only place the defaults live: every setting, no `version` key.
   - `src/logic/`: pure state machines and algorithms. No I/O. Time comes from `Clock`.
   - `tests/`: integration tests against `kx-platform-fake`. Golden files go through `kx_test_support::golden` (`assert_golden`), not `insta`.
   - `README.md`: a purpose paragraph, the services used and provided, the capabilities with a one-line reason each, and the settings.
2. **Wire it up:**
   - add it to the workspace `members`;
   - add a Cargo feature `mod-<name>` in `apps/keyxtend/Cargo.toml` (on by default only if the spec says so);
   - register the module in `apps/keyxtend/src/modules.rs` behind that feature.
3. **Security note:** copy `docs/security/TEMPLATE.md` to `docs/security/<name>.md` and fill in the capability table now. The threats get filled in later with `kx-security-check`.
4. **Check** that the ARCHITECTURE.md code map lists the crate; add a line if it does not.
5. **Run** `cargo xtask tidy` and `cargo test -p kx-mod-<name>`. Both must pass with the empty module.
6. **Commit:** `feat(<name>): scaffold module` with `Signed-off-by`.

## Rules
- A module never talks to the OS. If it needs something new from the OS, add a **port** (a trait) to `kx-module-api`, implement it in `kx-platform-fake` first (with tests), and only then in `kx-platform-windows`.
- UI for the module goes in `crates/kx-ui/qml/views/<Name>.qml`, with one cxx-qt bridge object in `crates/kx-ui/src/bridges/`. Its lists are paged, 5 rows per page (ADR-0008).
