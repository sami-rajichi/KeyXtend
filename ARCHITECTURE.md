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
         │ inherited anonymous pipes, versioned frames (kx-ipc)
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
  keyxtend/              composition root: picks the platform adapter and the enabled modules (one Cargo feature
                         per module); the command line, one running copy, the log, the panic hook and Qt's
                         hardened start; the packs' folders and the sealed store's folder (`words.rs`, P7);
                         the clipboard history's folder, made at boot (`clip.rs`, P8);
                         `gate-input` makes the gate-only test build (`--sim-input`, no uiAccess, refused by the
                         stage and `dev-install`; ADR-0027)
  keyxtend-worker/       worker binary: the `media` job decodes PNG and DIB images into small RGBA pixels, and makes
                         the `CF_DIB` an inserted PNG goes with, one request at a time over its stdin and stdout
                         (P8, ADR-0034); voice, OCR and update come later
crates/
  kx-module-api/        Module trait, Manifest, Capability, ports (service traits, the pointer hook's too:
                         ADR-0027, the scroll target's: ADR-0029, `Shortcuts`, gated by `InjectInput`, and
                         `PowerDialog`, by `Capability::Power`: ADR-0031; the box watch, packs, sealed store and
                         word events: ADR-0033; `Clipboard`, `ClipboardWriter`, `SealedShelf`, `Pictures` and
                         the clipboard panel's events: ADR-0034, `ClipPanel` gated by `ReadClipboard`: ADR-0035;
                         `Calendar` and `probe_ok`, the one rule that only an absolute drive path is looked at,
                         any other being `FileFact::Unchecked`: P8), events, Clock and Mono, settings contract,
                         notices, Redacted<T>, and `text`: folding and casing, which prediction and the
                         clipboard search share (P8); it re-exports the worker's memory figures from `kx-ipc`
                         (`READ_TIMES`, `DECODE_BYTES`), so the clipboard's settings check follows the worker (P8, ADR-0036)
  kx-kernel/            registry, lifecycle state machine, event bus, service registry,
                         grants, boot and runtime controls, redacted log
  kx-settings/          versioned TOML settings, repair, migrations (never edit an old migration)
  kx-ipc/               worker frames (P8): asks, replies and their codec, each field and length checked before a
                         buffer grows; no auth, as the pipe is anonymous and only the started worker inherits its ends
  kx-platform/          Platform (clock, AppDirs and the data folder rule) and the UI ports (WindowGuard, with
                         the entry watch and `lay_under` from P5, Screens, SystemLook, Backdrop, Instance, Tray:
                         ADR-0019, Memory: ADR-0023, Speaker: P4, and `Reveal`, which shows a folder in the file
                         manager: P8); the OS-neutral halves that every adapter shares (P7): `packs` (folder
                         order, file name, size, over an adapter's `PackDisk`) and
                         `field` (the watch's run, its `kx-field` thread and read deadline, a dead watch, our
                         own focus keeping the last box, the kind order, over an adapter's `Probe` and `Reader`),
                         and `pictures` (P8: the worker host, a `kx-pictures` thread per session that starts the
                         worker on demand, one draw at a time within a deadline, checked replies, a backoff and
                         an idle stop, over an adapter's `Spawner`)
  kx-platform-windows/  the only crate that may call Win32/WinRT: input, hooks, UIA, windows, look, tray,
                         sound, the scroll target (`scroll`), the shortcuts (`WinShortcuts`, from the
                         `shortcuts.toml` and `keynames.toml` tables), the power dialog (`WinPower`), the box
                         watch (`WinFieldWatch`, kinds from `field.toml`), the packs' maps (`WinWordPacks`) and
                         the DPAPI key guard (`WinKeyGuard`, P7); the worker's start (`WinSpawner`: a restricted
                         token with no admin group or privilege but change-notify, uiAccess cleared, Medium
                         integrity, all read back before it runs, only its pipes inherited, in a job; P8), the
                         clipboard watch and writer (`WinClipboard`, `WinClipboardWriter`, formats and markers
                         from `clip.toml`; its file facts follow no link), the local time (`WinCalendar`), the
                         drive's free space (`WinDiskRoom`) and the reveal through the running Explorer
                         (`WinReveal`, P8)
  kx-platform-macos/    (P15)
  kx-platform-linux/    X11 + Wayland portals (P15)
  kx-platform-fake/     deterministic fake for tests (records injected input, fake clock, golden layout tables,
                         a layouts service, the UI ports and the speaker); `FakePointerHook` runs an engine on
                         the test's thread, and its `let_go` and `give_up` act as the real host does;
                         `FakeScrollTarget` answers scripted aims and reaches and records turns; `FakeShortcuts`
                         lists scripted shortcuts and records sends with their held keys, `FakePowerDialog`
                         counts opens or fails on cue, and the fake layout source's `types_any` follows a held
                         Ctrl as Windows does (P6); `FakeFieldWatch`, `FakeWordPacks` and `FakeSealedStore`
                         script boxes, packs and blobs (P7); `FakeClipboard`, `FakeClipboardWriter`, `FakeShelf`,
                         `FakePictures`, `FakeCalendar` and `FakeReveal` script copies and files, record writes, keep
                         folders of blobs, queue draws, set the wall time and record folders shown (P8);
                         kx-module-api's contract test uses it too
  kx-platform-net/      worker-only HTTP (WinHTTP); `keyxtend` must never depend on it (P11)
  kx-ui/                qml/ (views and parts), src/bridges/ (cxx-qt objects), the `ui` module and its hub, and the
                         EN/FR/AR translations (`i18n/*.ftl`, Fluent, compiled in; ADR-0017), in the keyboard's
                         language, the tray's in the system's (ADR-0020); the `keysTip` and `keysText` bridge calls
                         name a shortcut's keys in the keyboard's language, `layerPageText` the layer's page for
                         readers, and `PageDots.qml` draws the page dots the Next key and the Cycle design share
                         (P6); the chip lane in the top bar, its menu and the paged email list (`WordLane`,
                         `WordMenu`, `WordList`), over the hub's `words_json` (P7); the clipboard panel (`ClipPanel`
                         and its parts) over `clip_json`, its pictures through the `image://clip` provider, never
                         JSON, and its texts in `i18n/clip/` (P8)
  kx-theme/             theme tokens compiled in (tokens/: themes, shape, motion, icons, sounds), the look view,
                         WCAG contrast of what the keyboard draws, measured on the face's row height (`row_px`),
                         the high-contrast palette, and the strict reader kx-ui's build checks sounds with; no
                         Qt, so it tests everywhere (ADR-0016)
  kx-mod-keyboard/      the board (`board.toml`: rows, widths, scan codes, roles), its boxes at any size, size
                         steps and presets, and the place on screen per DPI; typing: sticky modifiers, legends
                         and each click's route (ADR-0009); the face (board and state JSON, clicks, and the
                         board's row height, `row_px`, for the theme's contrast check) and the language key's
                         Carousel, Cycle and List; `[keyboard]` settings. The side roles `layer` (the Shortcuts
                         key) and `power` (asks the power module over the bus), the layer's generated `shortcut`
                         and `next` cells, the pure `layer.rs` (pages, cells, Next) and the `shortcut_pages`
                         setting (P6); the typed pieces it tells and the picked words it types (P7)
  kx-mod-layouts/       the Layouts service over the OS layouts (EN/FR/AR + any installed): the shown layout's
                         table (keys from `scans.toml`), the list in the OS order, step and pick; short names
                         in `[layouts]`; `LayoutChanged` names the language, told at start and on `LayoutAsk` (P7)
  kx-mod-mouse/         hold engine (Right-click, Grab), modifier+click (P4); its hook also runs while the pad
                         scrolls, to tell presses (`PointerPressed`) and move the badge (P5), and while word
                         prediction wants presses told (P7)
  kx-mod-scroll/        the scroll pad: the tap machine (`pad`), the glide (`glide`), the stop rules
                         (`stops`) and the actor, over the `ScrollTarget` port; `[scroll]` (P5)
  kx-mod-power/         the Power key's module: an actor that opens the system's own power dialog over
                         the `PowerDialog` port at each click, each open on a short thread of its own,
                         and tells the keyboard whether it runs; no settings (P6)
  kx-mod-predict/       word prediction (P7): an actor on its `kx-predict` thread that follows the keyboard's
                         pieces and the focused box, tells the lane, asks the keyboard to type picks (`TypeWord`)
                         and learns, over the tracker (`track`), the scorer (`score`), the learner (`learn`), the
                         email chips (`email`) and the word-pack format (`pack`; the `build` feature's
                         `PackWriter` serves `cargo xtask words` only); it folds with `kx_module_api::text`;
                         `[predict]`
  kx-mod-clipboard/     clipboard history (P8, ADR-0034): the `kx-clip` actor over the watch, the kind rules
                         (`entry`, `media.toml`), the limits (`keep`), follow-the-clear and skipped apps (`follow`),
                         the sealed day folders (`store`), search (`find`), the panel and its page (`panel`,
                         `view`), insert (`insert`) and the media and files threads (`media`); `[clipboard]`
  kx-mod-vault/         encrypted vault, unlock, import/export (P9)
  kx-mod-selection/     selection helper (P10)
  kx-mod-voice/         voice UI + worker client (engine lives in the worker) (P11)
  kx-mod-snip/          capture overlay, editor, OCR client (P12)
  kx-crypto/            sealed files: XChaCha20-Poly1305 under a data key the `KeyGuard` port wraps
                         (DPAPI/Keychain/Secret Service); `SealedFiles` backs the `SealedStore` port (P7), and
                         `SealedShelfFiles` the `SealedShelf` port: folders of blobs under a root with its own
                         key, each tag naming the folder (P8)
  kx-test-support/      shared test helpers, golden files, the fake kernel host for module tests; it
                         depends on kx-settings, whose key names its fake settings sink checks as the store does
tools/
  kx-gates/              Windows gate runner (moved from the P1 harness; never shipped); reads kx-gates.toml
                         and, beside it, kx-gates-p3.toml (the P3 gates, the mouse guard and the run-length
                         checks), kx-gates-p4.toml (the test build and P4's gates), kx-gates-p5.toml (G6 on the
                         test build's pad, and the pad's steps in G8, G9 and G10) and kx-gates-p7.toml (G27's
                         word chips, G21's product half and G8's prediction part, on copies it starts with
                         learning off, and the owner-settings sidecar, `[owner_settings] backup`) and
                         kx-gates-p8.toml (G20 on the product, `[g20_product]`, and G8's clipboard part,
                         `[g8_clip]`); uses spike-core widely until its gates are aimed at the product;
                         `capture-layouts` writes the fake's golden layout tables from the layouts installed on the PC
  kx-target-window/      test window that logs every character it receives (CI); a library plus a binary; reads
                         kx-target-window.toml; times its log with the QPC clock from kx-platform-windows;
                         `--layout` switches the session to an installed layout and puts the previous one back
                         when the window closes while it holds the keyboard (gates bring it to the front first);
                         a failed restore logs `# layout-kept`, and a killed window leaves the session switched;
                         `--fill` gives it lines to scroll both ways, and `--dpi-unaware` makes it an unaware app (P5)
  kx-test-icons/         the Qt Quick tests' QML plugin: kx-ui's icon provider over the icons the
                         texts test lays out, and the soft shadows' adopter, so the tests draw as
                         the app does
assets/                  fonts, icons and sounds with their licences, compiled into kx-ui (ADR-0016); the word
                         packs in `words/`, with their licence and credits, ship beside the program (P7)
spike/                   tested P1 code, moved out phase by phase (ADR-0014, docs/spike-move-map.md)
xtask/                   cargo xtask: tidy (architecture rules), dco (sign-off check), licences, dist (stub until P14),
                         stage (the release app with its Qt files, the word packs and the media worker),
                         dev-cert / dev-install / check-uiaccess (uiAccess test builds), sounds (recipes, import,
                         join; format and RIFF walker from kx-theme), words (P7: source hashes in `words.toml`,
                         wordfreq lists checked against the Hunspell dictionaries; packs from kx-mod-predict's
                         writer, the index, the meter), sign, sbom (P14)
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
- **Services** are traits in `kx-module-api` (ports). Examples: `InputInjector`, `LayoutSource`, `Layouts`, `KeyboardFace`, `FieldWatch`, `WordPacks`, `SealedStore`, `Clipboard`, `ClipboardWriter`, `SealedShelf`, `Pictures`, `SpeechToText`, `OcrEngine`, `ScrollTarget`, `PointerHook`, `Shortcuts` and `PowerDialog` (ADR-0031).
  - `cx.service::<K>()` returns the service only if the manifest `requires` it **and** the requester holds the capability stored by the service's provider. A request for another capability is refused, and only the kernel builds a module's `Grants`.
  - A provider must itself hold the capability that gates its service.
- **Time:** `Clock` and `Mono` live in `kx-module-api`, because modules may use only that crate. The real clock comes from the platform adapter; tests use `kx-platform-fake`.
- **Platform:** `kx-platform` holds `Platform` and `AppDirs`, which carry the data folder rule (ADR-0015).
- **Events** are typed, and an event may name a capability (`Event::CAPABILITY`) that a module needs to subscribe, intercept or publish it:
  - *notify* events are broadcast (`KeyActivated`, `LayoutChanged`, `ModeChanged`, `ModuleStateChanged`);
  - *intercept* events are an ordered chain that can stop propagation. No module intercepts today: a password box is the field watch's `Password` kind, which each module asks for itself (ADR-0033).
- **Settings** (ADR-0015):
  - one `settings.toml` in the data folder holds one flat section per module, each with its own `version`. Only the user's changes are stored, and the defaults live in the module's `defaults.toml`;
  - migrations are pure functions tested with golden files;
  - a bad value resets alone, a damaged file is set aside and the last good copy comes back, and each repair shows a notice. A file locked by another program is not damage: defaults run in memory and nothing is saved that run;
  - a Settings-window change restarts the module and the modules that need it, or tries a failed module again unless it waits for a provider. A module saves values it sets itself (size, position) through its `SettingsHandle`, with no restart; `set_all` saves values that belong together (x, y, placed) in one checked write;
  - `Kernel::load` reads the settings before any module starts, so the app opens its log first; `Kernel::start` then starts the modules, and `boot` does both.

## Invariants (enforced by `cargo xtask tidy` in CI)

1. `kx-mod-*` crates depend only on `kx-module-api` and pure utility crates. They never depend on each other, on `kx-kernel`, or on any `kx-platform-*`.
2. Only `apps/*` choose platform adapters and modules. The never-shipped test tools in `tools/` may use both, and `xtask` (`build_tool`) may use a module: `cargo xtask words` writes the packs with kx-mod-predict's writer (tidy D2).
3. Only `kx-platform-*` crates may contain `unsafe`. Every other crate has `#![forbid(unsafe_code)]`. Every `unsafe` block has a `// SAFETY:` comment.
   - Two narrow exceptions: the cxx-qt bridge blocks in `kx-ui/src/bridges/` (ADR-0013), and the never-shipped test tools in `tools/` (ADR-0014).
   - These use `#![deny(unsafe_code)]` with a scoped allow instead of `forbid`. Tidy enforces this for `tools/` and for `kx-ui`, where rule F3 refuses `unsafe` outside `src/bridges/` and any block without a `// SAFETY:` comment.
4. The `keyxtend` app has **no HTTP/TLS dependency**. `cargo tree -p keyxtend` must not contain `reqwest`, `hyper`, `ureq`, `rustls` or `native-tls`.
5. The `keyxtend` app never decodes untrusted images, audio or documents. Those crates may appear only in `keyxtend-worker`.
   - The media worker is the only PNG and DIB decoder, and tidy D4 (`banned_media`) keeps `png` and `fdeflate` out of the app's tree (ADR-0034).
6. No module logs typed text, learned words, emails, clipboard content, secrets or transcripts. Such values travel as `Redacted<T>`.
7. Every new dependency passes `cargo deny check` (licences that allow a closed, sold app: ADR-0030 and `deny.toml`) and is recorded in `THIRD_PARTY.md` if it ships data, fonts, models or icons.
   - No outside code is merged, and the licence-key check and signing secrets never enter this repository (ADR-0030).
8. UI never needs scrolling or dragging to operate. Every list is paged, with 5 rows per page (spec §3.3). UI reviews check this.
9. **Nothing hardcoded.**
   - Timings, sizes, speeds and limits live in the settings schema defaults.
   - Colours, fonts, spacing, motion, icon names and sounds live in theme tokens (`crates/kx-theme/tokens/`: `themes.toml`, `shape.toml`, `motion.toml`, `icons.toml`, `sounds.toml`).
   - User-visible text lives in translation files.
   - OS mappings live in adapter tables.
   - `kx-review` checks this.
10. Files ≤ 400 lines (aim ≤ 300), functions ≤ 40 lines, and doc comments of one or two sentences. `xtask tidy` warns about file length.
11. Only tools, dev-dependencies and test-only crates may use `kx-test-support` and `kx-platform-fake` (tidy D5). No shipped crate depends on them.
12. No package turns on a crate feature that `banned_features` in the root `Cargo.toml` lists (tidy D6). Today that is toml's `unbounded`, which lifts the nesting limit that keeps a settings file from overflowing the stack.
13. Nothing in the app's normal-dependency tree turns on a feature that `build_only_features` lists (tidy D7). Today that is kx-theme's `files` (the asset readers and the WAV reader), which only kx-ui's build and tests and xtask turn on, so the uiAccess app parses no media (invariant 5), and kx-mod-predict's `build` (the pack writer), which only xtask turns on.
14. Product code in `apps/` and `crates/` starts no process with `std::process::Command` or `QProcess` (tidy F4, `banned_spawns`). A worker starts through the platform's spawner (`WinSpawner`), which lowers its rights and lets it inherit only its pipes; tests, test modules, build scripts and `tools/` may start processes.

**CI** (`.github/workflows/ci.yml`):
- Windows builds and tests everything with Qt, runs the Qt Quick tests and checks the stage.
- Windows sets `KX_LIVE_CLIPBOARD=1`, so the adapter's live clipboard tests run there; elsewhere they are ignored, as they overwrite the real clipboard (`ci_opts_in_to_these_tests` pins the line).
- Windows also lints and tests the gate-only `gate-input` build and the adapter's `sim-input` in steps of their own; the stage then refuses that flag (ADR-0027).
- Windows also lints the app with the scroll module's feature off, so the scroll module stays optional.
- Linux leaves out the app and builds `kx-ui` without Qt until P15 (ADR-0016).

## Cross-cutting patterns

- **Hexagonal (ports and adapters).** Logic depends on traits, and adapters implement them per OS. Swapping the toolkit, or Windows for macOS, touches adapters only.
- **State machines as enums** with explicit transitions, for the keyboard, hold engine, scroll, vault lock and voice session. Time is always injected through a `Clock` trait, so tests are deterministic.
- **Actors for long-running work.** A module that owns a thread exposes a cloneable handle that sends messages. The thread ends when the last handle drops, or on a quit message where an adapter holds a sender too (prediction's box watch). No cycles of bounded channels.
- **No global mutable state.** The service registry, owned by the kernel, is the only "global". The one exception is the set-once hub `kx_ui::install` hands to the QML objects (ADR-0018). State a Win32 callback needs, kept inside the Windows adapter behind a port (per-thread window state, the COM apartment's set-once flag, the entry watch's one kept entry), is not app state. A lazy constant computed once from compiled-in data, such as `kx-ui`'s `theme_ids` memo, is not state, nor is a debug-only test flag or counter that release builds lack (`kx-ui`'s `CAPS_FLIPPED` churn flag, the `Ui` wake counters).
- **Errors:** `thiserror` enums in libraries and `anyhow` only in `apps/*`. Never panic on user input or IPC input.
- **UI binding (Qt, ADR-0013):** Rust decides and QML only draws and forwards clicks. Each view has one cxx-qt bridge object (`#[qml_element]`): read-only settings as constant properties, state as small properties or JSON it redraws from, and invokables for clicks. Accessible roles and names go on every control from day one.
- **UI hub (`kx-ui`).** QML makes the bridge objects without arguments, so they reach the app's `Hub` through `kx_ui::install`, set once before QML loads (ADR-0018). Adapter threads wake the Qt thread through cxx-qt's queue, never blocking. Try again and Quit go to the app through a request queue it drains on its own thread; Quit, from the user or a second copy, is the last request, and the queue then ends.
- **IPC:**
  - The platform's spawner starts each worker, which inherits only its two anonymous pipe ends. The worker connects to nothing else.
  - If a named pipe is ever needed: user-SID ACL, `first_pipe_instance`, `reject_remote_clients`, and a check of the client's image path and signature.
  - Frames have size limits and every field is checked; one request at a time within a deadline bounds the rate. Property tests stand in for fuzzing until the nightly toolchain is allowed (`docs/security/kx-ipc.md`).

## Qt window rules (ADR-0013)

- Pin Qt 6.11.2 and cxx-qt 0.10.0; review every upgrade.
- QML, fonts and icons are compiled into the binary as Qt resources; they are never loaded from disk.
- Keyboard windows are `Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus`; click-through windows add `Qt.WindowTransparentForInput`.
- The Windows adapter guards every window we create except Settings:
  - In `WM_STYLECHANGING`, it puts back `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`, and refuses click-through on a window that took clicks when first guarded.
  - In `WM_WINDOWPOSCHANGED`, it lifts a window that left the topmost band straight back, as `HWND_NOTOPMOST` never reaches `WM_STYLECHANGING`. After such a drop by another program, the keyboard comes back above a shown banner or list until a guard run names that pop-up as `top`.
  - It returns `MA_NOACTIVATE` for `WM_MOUSEACTIVATE`.
  - Every window it covers must start topmost (Qt's `WindowStaysOnTopHint`); guarding a plain hidden window can leave it below the topmost band.
- Never call `requestActivate()` on a no-focus window. Only the full Settings window takes focus, and it gives it back when it closes.
- Owned windows (shadows, bubble, tips, pill, caption, ring) are declared inside their owner, so Qt keeps them above it.
- The scroll outline's four edge windows have no owner (`transientParent: null`) and join the guard's quiet run, so `lay_under` can put them just under the keyboard each time a set shows or the keyboard shows again. They exist only while there is a box.
- A tip also lifts itself alone to the front as it shows (`SWP_NOOWNERZORDER`), since the guard may lift a banner or the list after its owner. A shown tip hides as a guard run starts, or the guard would lift its owner with it.
- A hidden window shows again at its old place, so a pop-up's guard run names it as `top` to land in front.
- Arabic flips text only: no window or control mirrors, and Arabic text reads right to left in the theme's Arabic font (ADR-0020).
- Every window draws on one shared graphics device, on the GUI thread (Qt's basic render loop) and without a depth buffer; a lost device, or a window that cannot draw, brings the windows back over the same hub, with the keyboard shown, closed or minimised as the user left it, up to `[ui] device_rebuilds` times, and Qt's loop runs between tries (ADR-0021).
- Only the shown theme's bundled fonts stay in Qt's font list: the hub loads them from the resources before the QML and again on each theme switch (ADR-0021).

## The scroll pad (P5, ADR-0029)

- **Module.** `kx-mod-scroll` holds every rule on the injected clock: the tap machine (`Pad`), the glide (`Glide`, lines owed per tick, a gap capped at `check_ms`) and the stop rules (`Stops`: the end, a press outside our windows, `limit_s` after the last tap, the target gone).
  - Its actor, on the `kx-scroll` thread, ticks every `tick_ms` only while scrolling, aims and reads the reach every `check_ms`, and logs one line of counts when it quits.
- **Port.** `ScrollTarget::open(ScrollConfig)` gives a `ScrollRun`: `aim` (the point, whether the pointer is on it, the box, ours or not, an id), `turn(aim, dir, lines)`, `reach` (more, end or unknown) and `alive`; dropping the run closes it.
  - `ScrollTargetKey` needs `Scroll`, which only the scroll module declares. The actor opens the run at the first peek or tap and keeps it until the module stops.
- **Events.** The keyboard sends `ScrollTap`, `ScrollStop` and `ScrollPeek` (the pointer on the pad); the module sends `ScrollChanged` (live, way, speed), `ScrollAimed` (the box, or none, and whether it is ours) and `ScrollPage` (our open list); `ScrollAsk` asks for the state and box again.
  - None of them names a capability, nor does the mouse module's `PointerPressed`, which carries no point; steps still go only where the user pointed.
- **Windows (`kx-platform-windows::scroll`).** A turn is a small wheel delta worked out from the user's wheel settings, posted to the window at the target while the pointer is elsewhere. It is sent by `SendInput` only while the pointer is on the target and Windows routes wheels to the pointer.
  - UI Automation runs on the run's own COM thread (`kx-scroll-uia`), each read capped by `read_ms`: from the target point, or, when ours covers that point, from the deepest element at it inside the target's own window, never going into one of ours, it finds the first element up that scrolls along the latest way's axis, its box and percents. It takes `ScrollPattern` line steps when the wheel setting is 0 or a page, or when wheel deltas move nothing (`Course`, spec §5.2).
  - A ping thread (`kx-scroll-ping`) asks the target for a `WM_NULL` answer; a window that leaves one unanswered, or that Windows calls hung, gets no step.
  - The guard's entry watch (`WindowGuard::watch_entries`) reads `GetMouseMovePointsEx` once as the pointer enters the keyboard and keeps the newest point outside it on a work area. A miss pushes the entry point `edge_px` past a keyboard edge, nearest first, onto the first work area it lands on.
  - It watches only while the scroll module runs; it passes over points on our windows that do not page, such as the banner, keeps the last entry when only they lie outside, and `unwatch_entries` drops the kept point when the module stops.
  - The open list asks `WindowGuard::mark_pages` as it opens and at each guard try until it holds, and a marked window that shows wins the aim, whatever point was kept, so a tap turns its page and a peek outlines it. A kept point that our other windows now cover aims at the first other app's window behind them (`scroll::win::behind`).
  - With no app behind, or with the pointer on our banner, there is no target: a check mid-scroll keeps the last target, and a tap asks the user to point first. A target that closed before a check stops the scroll, though another window lies at its point now.
- **UI (`kx-ui`).** The hub keeps the scroll state and the box (`scroll_json`, one wake per read), adds the badge's scroll part to `assist_json`, wakes the keys on `ScrollChanged`, and turns `ScrollPage` into the open list's page key on the Qt thread with `[ui] page_rows`.
  - `Outline.qml` makes four click-through edge windows (`Edge.qml`) only while there is a box, inside it, or just outside our own list's, which would cover them, and `WindowGuard::lay_under` puts them just under the keyboard.
- **Other platforms (P15).** macOS fills the same port with `CGEvent` scroll events and Accessibility for boxes, and Linux with XTest or the portal and AT-SPI. Wayland has no global pointer history or posted event, so there it scrolls only what is under the pointer.

## Word prediction (P7, ADR-0032, ADR-0033)

- **The keyboard stays the only typist.** `kx-mod-predict` holds no `InjectInput`: a pick sends `TypeWord`, and the keyboard types it through its own route, latches and dead-key rule, within `[keyboard] text_cap`, or types nothing. It refuses a word holding a hidden character (`kx_module_api::is_hidden`: controls, `HIDDEN_MARKS`, `INVISIBLE`), and `TEXT_CAP`'s floor fits the longest word chip.
- **Events.** `Typed` and `TypingMode` (keyboard to predict), `TypingModeAsk` (predict to the keyboard, at each start), `Suggested` (predict to the ui; `Lane::Stopped` hides the menu), `ChipPicked` and `WordsAsk` (ui to predict) and `TypeWord` (predict to the keyboard) all name `ReadFocusedText`. `PressesWanted` (a flag) and `PressesAsk` (nothing) carry no typed data and name none: they keep the mouse module's hook telling presses (`PointerPressed`) while predict wants them, in any start order.
- **Ports.** `FieldWatch` (`ReadFocusedText`) tells the focused box's kind and reads at most `[predict] context_words` words before the cursor as a `FieldRead`, with the kind the read saw, never in a password or unknown box. `WordPacks` (no gate: public data) maps one language's pack, and `SealedStore` (`SecretStore`) keeps sealed blobs up to `MAX_BLOB_BYTES`.
- **Who holds what.** `ReadFocusedText`: the keyboard, predict and the ui. `SecretStore`: predict, and the clipboard from P8. `KeyboardUi`, which gates `KeyboardFaceKey`, `AssistNote` and `PointerMoved`: the ui, and their sources, the keyboard and the mouse module. The app's `tests/life/ships.rs` pins each list.
- **Files.** The packs (`<iso>.kxw`), the index `PACK_INDEX` (`PackIndex`: each pack, then the layout codes it also serves, capped by `MAX_INDEX_BYTES`), their licence and credits sit in `PACK_DIR` beside the program, and a debug build also reads where `cargo xtask words` writes them; only the packs and the index are read through read-only maps. The learned words and emails are sealed in `SEALED_DIR` under the data folder (ADR-0015), by one `SealedFiles` per data folder.
- **Adapters.** Each OS fills only `Probe`, `Reader`, `PackDisk`, `KeyGuard` and the watch thread's body (its OS start-up, client and focus handler); the shared rules live in `kx-platform` and `kx-crypto` (code map), and the OS calls are in Platform notes below.

## Platform notes

| Concern | Windows (v1) | macOS (later) | Linux (later) |
|---|---|---|---|
| Top-most, no focus | uiAccess + `WS_EX_NOACTIVATE/TOPMOST/TOOLWINDOW` (guarded) | Qt `Tool` windows are non-activating `NSPanel`s | X11: `_NET_WM_STATE_ABOVE`; Wayland: LayerShellQt on KDE/wlroots; **not possible on GNOME Wayland** |
| Type | `SendInput` Unicode + VK | `CGEventPost` (Accessibility permission) | X11 XTest; Wayland RemoteDesktop portal / libei (asks the user once) |
| Mouse hook | `WH_MOUSE_LL` | `CGEventTap` | X11 XInput2; GNOME Wayland has no global hook, so we switch on GNOME's own "simulated secondary click" |
| Scroll | Wheel deltas posted, or `SendInput` while the pointer is on the target; UIA reads the box and end, and steps as the fallback (ADR-0029) | `CGEventCreateScrollWheelEvent`; Accessibility for boxes | XTest buttons 4–7 / portal; AT-SPI for boxes; Wayland only under the pointer |
| Secrets | DPAPI + Windows Hello | Keychain + Touch ID | Secret Service |
| Focused box (P7) | UI Automation focus handler, called on UI Automation's threads; reads on the watch's COM thread; kinds from `field.toml` | Accessibility (`AXUIElement`) | AT-SPI |
| Word packs (P7) | A read-shared file, mapped `PAGE_READONLY` | A read-only `mmap` from a folder only the system writes, else a copy | The same as macOS |
| Clipboard (P8) | `AddClipboardFormatListener` on a message-only window, markers from `clip.toml`; Open folder through the running Explorer | `NSPasteboard`'s change count, polled; Finder | XFixes on X11, data-control on Wayland where the compositor has it; the desktop's file manager |
| Power dialog | `Shell.Application.ShutdownWindows` | loginwindow prompt | session-manager prompt |
| Shortcuts | `SendInput` from `shortcuts.toml`; Lock PC by `LockWorkStation` (ADR-0031) | (P15) | (P15) |
