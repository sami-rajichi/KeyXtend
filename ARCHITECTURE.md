# Architecture

This file is a short map for anyone working on the code, human or agent. For *what* we build, read `docs/superpowers/specs/2026-09-26-accessible-keyboard-design.md`. For *why* a decision was made, read `docs/adr/`. This file only says *where things are* and *which rules must never be broken*.

## Bird's-eye view

```
┌──────────────────────────── keyxtend.exe (uiAccess, signed, Program Files) ───────────────────────┐
│  Qt Quick UI in QML (keyboard, panels, overlays, settings)                                         │
│        ▲ cxx-qt bridge objects (QML ⇄ Rust)                                                        │
│  ┌─────┴──────────────────────────── kx-kernel ───────────────────────────────┐                   │
│  │ module registry · lifecycle · typed event bus · service registry ·           │                   │
│  │ capability grants · settings + migrations · health monitor · tracing         │                   │
│  └─────┬───────────────┬───────────────┬───────────────┬──────────────┬────────┘                   │
│   kx-mod-keyboard  kx-mod-mouse  kx-mod-scroll  kx-mod-predict  kx-mod-clipboard …            │
│        │ (ports only: traits from kx-module-api)                                                  │
│  kx-platform-windows  (SendInput, hooks, UIA, clipboard, DPAPI, window styles)                    │
└────────┬───────────────────────────────────────────────────────────────────────────────────────────┘
         │ authenticated local pipe, versioned messages (kx-ipc)
┌────────┴─────────── keyxtend-worker.exe (normal rights, one process per job) ──────────┐
│  voice (mic, whisper.cpp, optional cloud with the user's own key) · ocr ·               │
│  media thumbnails · update check                                                         │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

**Why the keyboard itself is the uiAccess process:** Windows puts a window in the always-on-top band above the Start menu only if the process that owns the window has uiAccess (ADR-0003). uiAccess processes can also bypass UIPI, so the uiAccess process is kept **small and trusted**:

- it has no network access;
- it never parses untrusted images, files or audio.

All of that goes to workers (ADR-0004).

## Code map (Cargo workspace, flat `crates/`, folder name = crate name)

```
Cargo.toml               virtual workspace; internal crates use version 0.0.0
rust-toolchain.toml      pinned stable toolchain
deny.toml                cargo-deny: licences, advisories, bans, sources
apps/
  keyxtend/              composition root: picks the platform adapter and the enabled modules
                         (one Cargo feature per module)
  keyxtend-worker/       worker binary (subcommands: voice, ocr, media, update)
crates/
  kx-module-api/        Module trait, Manifest, Capability, ports (service traits), events,
                         Clock and Mono, settings contract, notices, Redacted<T>
  kx-kernel/            registry, lifecycle state machine, event bus, service registry,
                         grants, boot and runtime controls, redacted log
  kx-settings/          versioned TOML settings, repair, migrations (never edit an old migration)
  kx-ipc/               worker protocol: message enums, codec, size limits, pipe auth
  kx-platform/          Platform (clock, AppDirs and the data folder rule); later ports:
                         window, hooks
  kx-platform-windows/  the only crate that may call Win32/WinRT for input, hooks and UIA
  kx-platform-macos/    later
  kx-platform-linux/    later (X11 + Wayland portals)
  kx-platform-fake/     deterministic fake for tests (records injected input, fake clock)
  kx-platform-net/      worker-only HTTP (WinHTTP); `keyxtend` must never depend on it (P11)
  kx-ui/                qml/ (views and parts), src/bridges/ (cxx-qt objects), theme loader
  kx-mod-keyboard/      layout model, key state machine, modifiers, shortcuts layer
  kx-mod-layouts/       read OS layouts, label tables (EN/FR/AR + any installed)
  kx-mod-mouse/         hold engine (Right-click, Grab), modifier+click
  kx-mod-scroll/        scroll controller (2 speeds, target tracking)
  kx-mod-predict/       prefix trie, unigram/bigram counts, learner, email suggestions
  kx-mod-clipboard/     history store, retention, formats, exclusion rules
  kx-mod-vault/         encrypted vault, unlock, import/export
  kx-mod-selection/     selection helper
  kx-mod-voice/         voice UI + worker client (engine lives in the worker)
  kx-mod-snip/          capture overlay, editor, OCR client
  kx-mod-power/         power dialog
  kx-crypto/            data-key handling (DPAPI/Keychain/Secret Service port) + XChaCha20
  kx-test-support/      shared test helpers, golden files
tools/
  kx-gates/              Windows gate runner (moved from the P1 harness; never shipped);
                         reads kx-gates.toml; uses spike-core widely until its gates are
                         aimed at the product
  kx-target-window/      test window that logs every character it receives (CI); a library plus
                         a binary; reads kx-target-window.toml; uses spike-core only for the
                         QPC clock, until P3
spike/                   tested P1 code, moved out phase by phase (ADR-0014, docs/spike-move-map.md)
xtask/                   cargo xtask: tidy (architecture rules), dco (sign-off check), licences,
                         dist (stub until P14), dev-cert / dev-install / check-uiaccess
                         (uiAccess test builds), sign (later), sbom (later)
docs/
  adr/                   architecture decision records (never rewrite; supersede instead)
  superpowers/specs/     design specs
  superpowers/plans/     implementation plans (one per phase or module)
  test-reports/          manual Windows test results
design/                  HTML mock-ups (reference only, never shipped)
.claude/skills/          project agent skills
```

## The kernel contract

```rust
pub struct Manifest {
    pub id: ModuleId,                        // "keyboard", "clipboard", …
    pub version: &'static str,               // the crate version
    pub requires: &'static [ServiceId],      // services it consumes
    pub provides: &'static [ServiceId],      // services it registers
    pub capabilities: &'static [Capability], // what it may do (InjectInput, ReadClipboard, …)
    pub settings: Option<&'static SettingsSpec>, // version, defaults.toml, migrations, validator
}

pub trait Module: Send + 'static {
    fn manifest(&self) -> &'static Manifest;
    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError>; // handlers leave the bus before `stop` runs, services after it
    fn stop(&mut self) {}
}
```

- **Lifecycle** of each module: `Pending → Starting → Active | Failed`, then `Active → Stopping → Stopped`. The table `MOVES` in `crates/kx-kernel/src/lifecycle.rs` is the one source.
  - A failure puts the module in `Failed`. It never crashes the app.
  - Containing a panic needs unwinding, so no build profile may set `panic = "abort"`.
  - Dependents stop before their providers do.
  - `Kernel::retry` (Try again) moves `Failed → Starting`, and restarts the features that wait on it.
  - Switching a module off stops the active modules that need it, then the module itself, which ends `Stopped`. From `Pending` or `Failed` the module is skipped, with no failure notice.
  - Switching it on moves `Stopped → Starting`. A module that waits for a switched-off module stays `Stopped`, with no notice.
- **Services** are traits in `kx-module-api` (ports). Examples: `InputInjector`, `LayoutProvider`, `Predictor`, `SecretStore`, `ClipboardSource`, `SpeechToText`, `OcrEngine`, `ScrollTarget`, `PointerHook`.
  - `cx.service::<K>()` returns the service only if the manifest `requires` it **and** the requester holds the capability stored by the service's provider. A request for another capability is refused, and only the kernel builds a module's `Grants`.
  - A provider must itself hold the capability that gates its service.
- **Time:** `Clock` and `Mono` live in `kx-module-api`, because modules may use only that crate. The real clock comes from the platform adapter; tests use `kx-platform-fake`.
- **Platform:** `kx-platform` holds `Platform` and `AppDirs`, which carry the data folder rule (ADR-0015).
- **Events** are typed:
  - *notify* events are broadcast (`KeyActivated`, `LayoutChanged`, `ModeChanged`, `ModuleStateChanged`);
  - *intercept* events are an ordered chain that can stop propagation. Example: the "focused field is a password field" interceptor suppresses prediction learning.
- **Settings** (ADR-0015):
  - one `settings.toml` in the data folder holds one flat section per module, each with its own `version`. Only the user's changes are stored, and the defaults live in the module's `defaults.toml`;
  - migrations are pure functions tested with golden files;
  - a bad value resets alone, a damaged file is set aside and the last good copy comes back, and each repair shows a notice. Changing a module's settings restarts that module and the modules that need it.

## Invariants (enforced by `cargo xtask tidy` in CI)

1. `kx-mod-*` crates depend only on `kx-module-api` and pure utility crates. They never depend on each other, on `kx-kernel`, or on any `kx-platform-*`.
2. Only `apps/*` choose platform adapters and modules.
3. Only `kx-platform-*` crates may contain `unsafe`. Every other crate has `#![forbid(unsafe_code)]`. Every `unsafe` block has a `// SAFETY:` comment.
   - Two narrow exceptions: the cxx-qt bridge blocks in `kx-ui/src/bridges/` (ADR-0013), and the never-shipped test tools in `tools/` (ADR-0014).
   - These use `#![deny(unsafe_code)]` with a scoped allow instead of `forbid`; tidy enforces this for `tools/` now and learns it for `kx-ui` in P3.
4. The `keyxtend` app has **no HTTP/TLS dependency**. `cargo tree -p keyxtend` must not contain `reqwest`, `hyper`, `ureq`, `rustls` or `native-tls`.
5. The `keyxtend` app never decodes untrusted images, audio or documents. Those crates may appear only in `keyxtend-worker`.
6. No module logs typed text, clipboard content, secrets or transcripts. Such values travel as `Redacted<T>`.
7. Every new dependency passes `cargo deny check` (licences allowed for GPL-3.0-or-later: see `deny.toml`) and is recorded in `THIRD_PARTY.md` if it ships data, fonts, models or icons.
8. UI never needs scrolling or dragging to operate. Every list is paged, with 5 rows per page (spec §3.3). UI reviews check this.
9. **Nothing hardcoded.**
   - Timings, sizes, speeds and limits live in the settings schema defaults.
   - Colours, fonts, spacing and motion live in theme tokens (`themes.toml`, `shape.toml`, `motion.toml`).
   - User-visible text lives in translation files.
   - OS mappings live in adapter tables.
   - `kx-review` checks this.
10. Files ≤ 400 lines (aim ≤ 300), functions ≤ 40 lines, and doc comments of one or two sentences. `xtask tidy` warns about file length.
11. Only tools, dev-dependencies and test-only crates may use `kx-test-support` and `kx-platform-fake` (tidy D5). No shipped crate depends on them.

## Cross-cutting patterns

- **Hexagonal (ports and adapters).** Logic depends on traits, and adapters implement them per OS. Swapping the toolkit, or Windows for macOS, touches adapters only.
- **State machines as enums** with explicit transitions, for the keyboard, hold engine, scroll, vault lock and voice session. Time is always injected through a `Clock` trait, so tests are deterministic.
- **Actors for long-running work.** A module that owns a thread exposes a cloneable handle that sends messages. The thread ends when the last handle drops. No cycles of bounded channels.
- **No global mutable state.** The service registry is the only "global", and it is owned by the kernel.
- **Errors:** `thiserror` enums in libraries and `anyhow` only in `apps/*`. Never panic on user input or IPC input.
- **UI binding (Qt, ADR-0013):** Rust decides and QML only draws and forwards clicks. Each view has one cxx-qt bridge object (`#[qml_element]`): read-only settings as constant properties, state as small properties or JSON it redraws from, and invokables for clicks. Accessible roles and names go on every control from day one.
- **IPC:**
  - The main process spawns each worker and hands it an inherited pipe handle. The worker connects to nothing else.
  - If a named pipe is ever needed: user-SID ACL, `first_pipe_instance`, `reject_remote_clients`, and a check of the client's image path and signature.
  - Messages have size limits, are validated and rate-limited, and are fuzzed.

## Qt window rules (ADR-0013)

- Pin Qt 6.11.2 and cxx-qt 0.10.0; review every upgrade.
- QML is compiled into the binary as a resource module; it is never loaded from disk.
- Keyboard windows are `Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus`; click-through windows add `Qt.WindowTransparentForInput`.
- The Windows adapter guards every window we create except Settings:
  - In `WM_STYLECHANGING`, it puts back `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`.
  - It returns `MA_NOACTIVATE` for `WM_MOUSEACTIVATE`.
- Never call `requestActivate()` on a no-focus window. Only the full Settings window takes focus, and it gives it back when it closes.
- Owned windows (shadows, bubble, pill, caption, ring) are declared inside their owner, so Qt keeps them above it.
- Right-to-left layouts use `LayoutMirroring`; Arabic text uses the theme's Arabic font.

## Platform notes

| Concern | Windows (v1) | macOS (later) | Linux (later) |
|---|---|---|---|
| Top-most, no focus | uiAccess + `WS_EX_NOACTIVATE/TOPMOST/TOOLWINDOW` (guarded) | Qt `Tool` windows are non-activating `NSPanel`s | X11: `_NET_WM_STATE_ABOVE`; Wayland: LayerShellQt on KDE/wlroots; **not possible on GNOME Wayland** |
| Type | `SendInput` Unicode + VK | `CGEventPost` (Accessibility permission) | X11 XTest; Wayland RemoteDesktop portal / libei (asks the user once) |
| Mouse hook | `WH_MOUSE_LL` | `CGEventTap` | X11 XInput2; GNOME Wayland has no global hook, so we switch on GNOME's own "simulated secondary click" |
| Scroll | UIA ScrollPattern → posted wheel → SendInput | `CGEventCreateScrollWheelEvent` | XTest buttons 4–7 / portal |
| Secrets | DPAPI + Windows Hello | Keychain + Touch ID | Secret Service |
| Power dialog | `Shell.Application.ShutdownWindows` | loginwindow prompt | session-manager prompt |
