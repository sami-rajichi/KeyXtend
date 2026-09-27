# Architecture

This file is a short map for anyone working on the code, human or agent. For *what* we build, read `docs/superpowers/specs/2026-09-26-accessible-keyboard-design.md`. For *why* a decision was made, read `docs/adr/`. This file only says *where things are* and *which rules must never be broken*.

## Bird's-eye view

```
┌──────────────────────────── keyxtend.exe (uiAccess, signed, Program Files) ───────────────────────┐
│  Slint UI (keyboard, panels, overlays, settings)                                                   │
│        ▲ adapters (ui/*.slint globals ⇄ Rust)                                                      │
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
                         Redacted<T>
  kx-kernel/            registry, lifecycle state machine, event bus, service registry,
                         grants, health
  kx-settings/          versioned TOML settings + migrations (never edit an old migration)
  kx-ipc/               worker protocol: message enums, codec, size limits, pipe auth
  kx-platform/          platform ports that are not module services (window, hooks, clock)
  kx-platform-windows/  the only crate that may call Win32/WinRT for input, hooks and UIA
  kx-platform-macos/    later
  kx-platform-linux/    later (X11 + Wayland portals)
  kx-platform-fake/     deterministic fake for tests (records injected input, fake clock)
  kx-ui/                ui/theme.slint, ui/widgets/, ui/views/*.slint (Adapter globals), src/adapters/
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
    pub id: ModuleId,                       // "keyboard", "clipboard", …
    pub version: semver::Version,
    pub requires: &'static [ServiceId],     // services it consumes
    pub provides: &'static [ServiceId],     // services it registers
    pub capabilities: &'static [Capability],// what it may do (InjectInput, ReadClipboard, …)
    pub settings_version: u32,
}

pub trait Module: Send + 'static {
    fn manifest(&self) -> &'static Manifest;
    fn start(&mut self, cx: &mut ModuleCx) -> Result<(), ModuleError>; // registrations return Drop guards held by cx
    fn stop(&mut self) {}
}
```

- **Lifecycle** of each module: `Pending → Starting → Active | Failed → Stopping → Stopped`.
  - A failure puts the module in `Failed`. It never crashes the app.
  - Dependents stop before their providers do.
- **Services** are traits in `kx-module-api` (ports). Examples: `InputInjector`, `LayoutProvider`, `Predictor`, `SecretStore`, `ClipboardSource`, `SpeechToText`, `OcrEngine`, `ScrollTarget`, `PointerHook`.
  - `cx.service::<dyn InputInjector>()` returns the service only if the manifest `requires` it **and** the matching capability is granted.
  - Capability handles are types that only the kernel can construct.
- **Events** are typed:
  - *notify* events are broadcast (`KeyActivated`, `LayoutChanged`, `ModeChanged`, `ModuleStateChanged`);
  - *intercept* events are an ordered chain that can stop propagation. Example: the "focused field is a password field" interceptor suppresses prediction learning.
- **Settings:**
  - each module owns a section with its own `settings_version`;
  - migrations are pure functions tested with golden files;
  - invalid settings fail loudly and fall back to defaults with a visible notice.

## Invariants (enforced by `cargo xtask tidy` in CI)

1. `kx-mod-*` crates depend only on `kx-module-api` and pure utility crates. They never depend on each other, on `kx-kernel`, or on any `kx-platform-*`.
2. Only `apps/*` choose platform adapters and modules.
3. Only `kx-platform-*` crates may contain `unsafe`. Every other crate has `#![forbid(unsafe_code)]`. Every `unsafe` block has a `// SAFETY:` comment.
4. The `keyxtend` app has **no HTTP/TLS dependency**. `cargo tree -p keyxtend` must not contain `reqwest`, `hyper`, `ureq`, `rustls` or `native-tls`.
5. The `keyxtend` app never decodes untrusted images, audio or documents. Those crates may appear only in `keyxtend-worker`.
6. No module logs typed text, clipboard content, secrets or transcripts. Such values travel as `Redacted<T>`.
7. Every new dependency passes `cargo deny check` (licences allowed for GPL-3.0-or-later: see `deny.toml`) and is recorded in `THIRD_PARTY.md` if it ships data, fonts, models or icons.
8. UI never needs scrolling or dragging to operate. Every list is paged, with 5 rows per page (spec §3.3). UI reviews check this.
9. **Nothing hardcoded.**
   - Timings, sizes, speeds and limits live in the settings schema defaults.
   - Colours, fonts and spacing live in theme tokens (`ui/theme.slint`).
   - User-visible text lives in translation files.
   - OS mappings live in adapter tables.
   - `kx-review` checks this.
10. Files ≤ 400 lines (aim ≤ 300), functions ≤ 40 lines, and doc comments of one or two sentences. `xtask tidy` warns about file length.

## Cross-cutting patterns

- **Hexagonal (ports and adapters).** Logic depends on traits, and adapters implement them per OS. Swapping Slint for another toolkit, or Windows for macOS, touches adapters only.
- **State machines as enums** with explicit transitions, for the keyboard, hold engine, scroll, vault lock and voice session. Time is always injected through a `Clock` trait, so tests are deterministic.
- **Actors for long-running work.** A module that owns a thread exposes a cloneable handle that sends messages. The thread ends when the last handle drops. No cycles of bounded channels.
- **No global mutable state.** The service registry is the only "global", and it is owned by the kernel.
- **Errors:** `thiserror` enums in libraries and `anyhow` only in `apps/*`. Never panic on user input or IPC input.
- **UI binding (Slint):** each `ui/views/*.slint` exports a `global XxxAdapter` (properties and callbacks), and Rust `src/adapters/*` connects it to module handles, following Slint's `todo-mvc` example. Accessible roles and labels go on every control from day one.
- **IPC:**
  - The main process spawns each worker and hands it an inherited pipe handle. The worker connects to nothing else.
  - If a named pipe is ever needed: user-SID ACL, `first_pipe_instance`, `reject_remote_clients`, and a check of the client's image path and signature.
  - Messages have size limits, are validated and rate-limited, and are fuzzed.

## Slint window rules (ADR-0002)

- Pin `slint = "=1.18.1"`. The `unstable-winit-030` API may change in any minor release.
- Renderer: FemtoVG-OpenGL or Skia-OpenGL, never Skia-DX12.
- The Windows adapter subclasses every Slint window we create:
  - In `WM_STYLECHANGING`, it puts back `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`.
  - It returns `MA_NOACTIVATE` for `WM_MOUSEACTIVATE`.
  - Windows are created with `with_active(false)` and `with_skip_taskbar(true)`.
- Never call `bring_to_front()`. Set size and position only through Slint's API.
- Keyboard scale is a `.slint` global. Right-to-left mirroring for the Arabic UI is done by hand with a `Dir.rtl` global.
- The pointer ring, burst and badge are **native** layered windows owned by the platform adapter, not Slint windows.

## Platform notes

| Concern | Windows (v1) | macOS (later) | Linux (later) |
|---|---|---|---|
| Top-most, no focus | uiAccess + `WS_EX_NOACTIVATE/TOPMOST/TOOLWINDOW` (guarded) | `NSPanel` nonactivating via class swap until Slint moves to winit 0.31 | X11: `_NET_WM_STATE_ABOVE`; Wayland: layer-shell on KDE/wlroots (layer-shika); **not possible on GNOME Wayland** |
| Type | `SendInput` Unicode + VK | `CGEventPost` (Accessibility permission) | X11 XTest; Wayland RemoteDesktop portal / libei (asks the user once) |
| Mouse hook | `WH_MOUSE_LL` | `CGEventTap` | X11 XInput2; GNOME Wayland has no global hook, so we switch on GNOME's own "simulated secondary click" |
| Scroll | UIA ScrollPattern → posted wheel → SendInput | `CGEventCreateScrollWheelEvent` | XTest buttons 4–7 / portal |
| Secrets | DPAPI + Windows Hello | Keychain + Touch ID | Secret Service |
| Power dialog | `Shell.Application.ShutdownWindows` | loginwindow prompt | session-manager prompt |
