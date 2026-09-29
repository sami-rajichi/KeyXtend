# P2: kernel, settings and test tools

## Goal
Build the kernel that starts each feature in order and contains its failures, plus the settings store and a redacted log: roadmap "Phase 2", ARCHITECTURE.md "The kernel contract", ADR-0005.
Move the test tools out of the spike and delete the Slint and Tauri faces (`docs/spike-move-map.md`, P2; ADR-0014).

Owner's decisions (2026-09-29):
- Two editions. The portable one keeps a `data` folder beside the program; the installed one uses `%LocalAppData%\KeyXtend`. A new user starts clean.
- The settings file holds only the user's changes; defaults live in the app.
- A damaged file is repaired: only bad values reset, the last good copy returns, a copy of the damaged file is kept, and a notice says so.
- Log files: this run and the previous one, never typed text.
- A failed feature leaves the rest running, and a notice offers Try again.
- The Qt CI job comes in P3. The tools are named `tools/kx-gates` and `tools/kx-target-window`.

## Scope
- **In:** `kx-module-api`, `kx-kernel`, `kx-settings`, `kx-platform`, `kx-platform-fake`, `kx-test-support`; the moves into `tools/`; the Slint and Tauri removal; tidy for `tools/`; ADR-0015.
- **Out:** the app binary, windows and translated texts (P3); the health monitor (P4, P11); IPC and workers (P11); the Settings window (P13).

## Design notes
- **Workspace:**
  - members gain `crates/*` and `tools/*`, and `exclude = ["spike"]`;
  - tools reach `spike/core` by path, through Windows-only dependencies, so Linux CI never builds it.
- **kx-module-api** (modules see only this; pure deps `serde`, `toml`, `thiserror`):
  - `ModuleId`, `ServiceId` and `Capability`: InjectInput, PointerHook, ReadClipboard, ReadFocusedText, Microphone, Network, ScreenCapture, SecretStore;
  - `Manifest`: its `version` is the crate version string, so no `semver` dependency; `settings` is a `SettingsSpec`;
  - `Module`, `ModuleCx` (typed front of the kernel's object-safe `Host`), `ModuleError`, `ServiceError`;
  - `ServiceKey`: the typed key naming a service's trait, id and needed capability;
  - `Event`, `Flow` (Continue, Stop), `Bus` (cloneable handle for worker threads), `Subscription` (drop to leave);
  - `SettingsSpec` (version, embedded `defaults.toml`, migrations, validator), `Settings` (serde + `check`), `Migration`;
  - `Notice` (translation key, module, arguments), `ModuleStateChanged`, `SettingsChanged`;
  - `Redacted<T>`, printing `‹redacted›`;
  - `Clock` and `Mono` (monotonic µs) with the time-unit constants. **Move-map refinement:** the `Clock` port lives here, not in `kx-platform`, because tidy D1 lets modules use only this crate.
- **kx-kernel:** `lifecycle` (the state enum and its transitions), `order` (dependency order), `registry` (services), `grants` (`Policy`, `Grants`), `bus`, `host`, `kernel` (`Kernel`: start, stop, retry, enable), `log` (redaction layer, log files).
- **kx-settings:** `file` (safe save, recovery), `section` (versions, migrations, per-value repair), `store` (`Store`: read, set, save changes only).
- **kx-platform:** `Platform` (what an adapter hands the kernel) and `AppDirs`, with the data-folder rule. **kx-platform-fake:** `FakeClock`, `FakePlatform`.
- **kx-test-support:** `pct` and `ringstats` (moved), `golden` (compare, `KX_BLESS=1` rewrites), `TempDir`.
- **Files in the data folder** (names in one table in `kx-settings`): `settings.toml`, `settings.previous.toml`, `settings.broken.toml`, `logs/keyxtend.log`, `logs/keyxtend.previous.log`.
- **New settings, `[kernel]` version 1:** `disabled = []`, `log_level = "info"`, `log_max_mb = 10`.
- **Schema limit:** the settings file is capped at 1 MiB, a constant in `kx-settings`, because the file cannot set its own limit.
- **Translation keys** (ids only; P3 adds EN/FR/AR text): `notice.settings-reset`, `notice.settings-restored`, `notice.settings-defaults`, `notice.settings-newer`, `notice.settings-unsaved`, `notice.module-failed`.
- **Theme tokens:** none.

## Tasks
1. **Remove Slint and Tauri.** *Files:* `spike/slint-kb/`, `spike/tauri-kb/` (deleted), `spike/spike.toml` (`[keyboard] titles` becomes one `title`), `spike/core/src/{config,window}.rs`, `spike/harness/src/{usage,main}.rs`, `spike/qt-kb/src/main.rs`, `spike/Cargo.toml`.
   *Test first:* a harness parse test refuses `slint` and `tauri`. *Implement:* delete the faces, their settings and `window::click_through`; repoint `worker_dir` to the Qt stage.
   *Verify:* spike tests pass; `qt-kb` builds offline; `qmltestrunner` 29/29.
2. **Tidy learns `tools/`.** *Files:* `xtask/src/tidy/{mod,files,deps}.rs` and their tests, root `Cargo.toml` metadata, `xtask/src/dco/git.rs`.
   *Test first:* a tool root with `#![deny(unsafe_code)]` passes F2; a shipped crate depending on a tool or on `kx-test-support`/`kx-platform-fake` fails new rule D5.
   *Implement:* add `tools` to the scan, the tool attribute exception and D5; add `-c core.fsmonitor=false` to the test git helper.
   *Verify:* `cargo test -p xtask` passes; `cargo xtask tidy` 0/0.
3. **`kx-module-api` time and redaction.** *Files:* `crates/kx-module-api/src/{lib,clock,redacted}.rs`.
   *Test first:* `Mono` arithmetic saturates; `Redacted` prints the marker through Debug, Display and nested Debug.
   *Implement:* `Clock`, `Mono`, units, `Redacted<T>`. *Verify:* `cargo test -p kx-module-api`.
4. **`kx-test-support` with moves.** *Files:* `crates/kx-test-support/src/{lib,pct,ringstats,golden,tempdir}.rs`; `spike/core/src/lib.rs`; `spike/qt-kb/{Cargo.toml,src/ring.rs}`.
   *Test first:* the moved tests pass in the new crate; a golden mismatch fails and `KX_BLESS=1` rewrites.
   *Implement:* `git mv` the files, take the unit constant from `kx-module-api`, point spike users at the crate. *Verify:* both workspaces' tests pass; `qt-kb` builds.
5. **Move `target-window` to `tools/kx-target-window`.** *Files:* its `src/{lib,log,config,main}.rs`, `kx-target-window.toml` (from `spike.toml [target]`), `build.rs`; spike config drops `[target]`.
   *Test first:* the log-format and config tests pass; the config loads beside the exe, else from the crate folder.
   *Implement:* library (log format, config) plus binary; the QPC clock stays `spike_core::clock` until P3; non-Windows builds print "Windows only".
   *Verify:* workspace clippy and tests pass.
6. **Move the harness to `tools/kx-gates`.** *Files:* `tools/kx-gates/**`, `kx-gates.toml` (from `harness.toml`, paths rebased), `build.rs`.
   *Test first:* the 120 moved tests pass, plus a test that every configured path resolves inside the repo.
   *Implement:* `git mv`, Windows-only code, its own config lookup; target-window reads through `kx-target-window`'s library; `pct` comes from `kx-test-support`.
   *Verify:* `cargo test -p kx-gates` passes.
7. **Harden `kx-gates` to the product lints.** *Files:* `tools/kx-gates/src/**`.
   *Test first:* none new; clippy `-D warnings` is the test. *Implement:* fix about 150 findings; add a scoped `unsafe` allow with a `// SAFETY:` note on each block; allow `print_stdout` at the root with a reason, since printing results is its job; split files over 300 lines.
   *Verify:* clippy clean; tidy 0/0.
8. **`kx-module-api` contract.** *Files:* `src/{ids,capability,manifest,service,event,settings,notice,cx,error}.rs`.
   *Test first:* `ModuleCx` refuses a service the manifest did not require; typed events round-trip through a fake `Host`.
   *Implement:* the types in Design notes. *Verify:* `cargo test -p kx-module-api`.
9. **`kx-platform` and `kx-platform-fake`.** *Files:* `crates/kx-platform/src/{lib,dirs}.rs`, `crates/kx-platform-fake/src/{lib,clock}.rs`.
   *Test first:* a `data` folder beside the exe wins; a file named `data` does not; the fake clock moves only when advanced.
   *Implement:* `Platform`, `AppDirs`, `FakeClock`, `FakePlatform`. *Verify:* tests pass.
10. **`kx-settings` file safety.** *Files:* `crates/kx-settings/src/{lib,file,names}.rs`.
    *Test first:* a failed save leaves the old file; an unreadable file restores the previous copy; both unreadable give defaults and keep a broken copy; an oversized file counts as damaged.
    *Implement:* temp write, then rename, keeping the previous copy. *Verify:* `cargo test -p kx-settings`.
11. **`kx-settings` sections.** *Files:* `src/{section,store,merge}.rs`, `tests/golden/`.
    *Test first (with proptest):* only bad values reset; migrations run in order (golden files); a newer or unknown section is kept word for word; a value set back to its default leaves the file; save then load gives the same settings.
    *Implement:* `Store` and its notices. *Verify:* tests pass.
12. **Kernel lifecycle and order.** *Files:* `crates/kx-kernel/src/{lifecycle,order}.rs`.
    *Test first (with proptest):* random events never reach a state outside the ARCHITECTURE.md diagram; in random graphs providers start first, a cycle fails only its members, and stop runs in reverse.
    *Implement:* the state enum, and Kahn ordering in insertion order. *Verify:* `cargo test -p kx-kernel`.
13. **Bus, registry and grants.** *Files:* `src/{bus,registry,grants,host}.rs`.
    *Test first:* Stop ends an intercept chain; a handler may publish or unsubscribe mid-publish; a panicking handler is contained; an ungranted capability gives `NotGranted`; a second provider of a service fails.
    *Implement:* as named. *Verify:* tests pass.
14. **The `Kernel`.** *Files:* `src/kernel.rs`, `src/kernel/tests.rs`, `defaults.toml`.
    *Test first:* the roadmap edge cases below, with sample modules on `kx-platform-fake`.
    *Implement:* start, stop, retry, enable/disable (saved in `[kernel] disabled`), contained panics, notices. *Verify:* tests pass.
15. **Logging.** *Files:* `src/log.rs`, `src/log/tests.rs`.
    *Test first:* sensitive field names print the marker; start rotates this run's log to the previous name; writing stops at `log_max_mb` with one final line.
    *Implement:* the `tracing-subscriber` field formatter and the file writer. *Verify:* tests pass.
16. **Settings demo.** *Files:* `crates/kx-kernel/examples/settings_demo.rs`.
    *Test first:* none; it serves the manual check. *Implement:* boots two sample modules with the real `Store` in a folder given on the command line, prints notices, and saves a change.
    *Verify:* runs on this PC.
17. **Docs.** ADR-0015 (data folder, changes-only settings, repair, logs); ARCHITECTURE.md; spec §8.6 and §9.3; move map; README "Where your settings live"; CONTRIBUTING (running the tools); `docs/security/kx-kernel.md`; P3 notes.

## Edge cases (each has a test)
1. A module fails during start, or panics: it is `Failed`, the rest are `Active`, its services and subscriptions are gone, and a notice appears.
2. Try again after a failure restarts it and the dependents that failed with it.
3. Dependency cycle: its members fail; the others start.
4. A missing required service fails the module and its dependents.
5. An ungranted capability: `NotGranted`, and the module fails if it needs the service.
6. A service the manifest did not require: `NotRequired`. Two providers: the second fails.
7. Stop order: dependents stop first, also when one provider is disabled.
8. Corrupt file: the previous copy is used; if both are bad, defaults are used and a broken copy is kept.
9. A file from a newer version: that section keeps its text, the module uses defaults, a notice appears, and save keeps the text.
10. One bad value: only it resets, it is named in the notice, and the next save drops it.
11. An unknown section (module not built in) survives save unchanged.
12. A section without `version` counts as current; a failing migration counts as a bad section.
13. A value equal to its default is not written.
14. A read-only data folder: the app runs on settings in memory and warns once.
15. Settings file over 1 MiB: treated as damaged.
16. Log over `log_max_mb`: writing stops after one final line.
17. Intercept Stop, and re-entrant publish or unsubscribe, never deadlock.

## Security
- **Capabilities:** P2 uses none; the kernel enforces them. Policy is not a sandbox (ADR-0005).
- **Data:** settings are personal, not secret, and hold no API keys; logs keep typed text, clipboard data, secrets and transcripts out.
- **Threats to test:** a hand-edited or tampered file (each value checked, size cap); a runaway log (cap); a secret in a log field (redaction layer); a shipped crate depending on a tool or fake (tidy D5).
- **Record:** `docs/security/kx-kernel.md` via `kx-security-check`.

## Manual Windows check
1. Build `kx-gates` and `kx-target-window` from the root workspace.
2. `kx-gates g1 target --count 200` passes, and `kx-gates g2 qt --clicks 100` passes against the staged Qt test keyboard.
3. `settings_demo` in a portable `data` folder: the owner sees a readable `settings.toml` holding only the change.
4. I damage the file, and after a second run the owner sees the repair, the broken copy and the notice.
5. Report in `docs/test-reports/2026-09-29-p2-kernel.md`.

## Done when
- The Definition of Done in `docs/roadmap.md` is met.
- The spike still builds and its tests pass, and Dependabot alert #1 closes after the push.
