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
│  media thumbnails · vault (backup boxes, KeePass files) · update check                   │
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
                         the clipboard history's folder, made at boot (`clip.rs`, P8); the notebook's folder
                         (`vault.rs`, P9); the probe before the log and the settings, and a hidden run's memory
                         stores and banner, with the questions about copies left behind (`hidden.rs`, ADR-0037);
                         `gate-input` makes the gate-only test build (`--sim-input`, no uiAccess, refused by the
                         stage and `dev-install`; ADR-0027)
  keyxtend-worker/       worker binary, one request at a time over its stdin and stdout: the `media` job decodes
                         PNG and DIB images into small RGBA pixels, and makes the `CF_DIB` an inserted PNG goes
                         with (P8, ADR-0034); the `vault` job makes padlocks, seals, opens and checks backup boxes
                         and reads KeePass files, with `age` and `keepass` (P9, ADR-0038); voice, OCR and update
                         come later
crates/
  kx-module-api/        Module trait, Manifest, Capability, ports (service traits, the pointer hook's too:
                         ADR-0027, the scroll target's: ADR-0029, `Shortcuts`, gated by `InjectInput`, and
                         `PowerDialog`, by `Capability::Power`: ADR-0031; the box watch, packs, sealed store and
                         word events: ADR-0033; `Clipboard`, `ClipboardWriter`, `SealedShelf`, `Pictures` and
                         the clipboard panel's events: ADR-0034, `ClipPanel` gated by `ReadClipboard`: ADR-0035;
                         `Calendar` and `probe_ok`, the one rule that only an absolute drive path is looked at,
                         any other being `FileFact::Unchecked`: P8; `Boxes`, `Places`, `Random` and Quick-fill's
                         events and pages: ADR-0038, ADR-0039; `SelectionWatch`, `WebLink`, the chips' events and
                         the strip's place, `geom::anchor`: ADR-0041), events, Clock and Mono, settings contract,
                         notices, Redacted<T> and `Secret`, and `text`: folding and casing, which prediction and
                         the clipboard search share (P8), and Quick-fill's search too; it re-exports the worker's
                         memory figures and the box facts from `kx-ipc` (`READ_TIMES`, `DECODE_BYTES`), so the
                         settings checks follow the worker (P8, ADR-0036)
  kx-kernel/            registry, lifecycle state machine, event bus, service registry,
                         grants, boot and runtime controls, redacted log
  kx-settings/          versioned TOML settings, repair, migrations (never edit an old migration)
  kx-ipc/               worker frames (P8): asks, replies and their codec, each field and length checked before a
                         buffer grows; no auth, as the pipe is anonymous and only the started worker inherits its ends;
                         the vault job's frames, with secrets in wiped buffers, and the box's facts and caps (P9)
  kx-platform/          Platform (clock, AppDirs and the data folder rule, `VAULT_DIR`) and the UI ports (WindowGuard,
                         with the entry watch and `lay_under` from P5, Screens, SystemLook, Backdrop, Instance, Tray:
                         ADR-0019, Memory: ADR-0023, Speaker: P4, `Reveal`, which shows a folder in the file
                         manager: P8, and `Hidden`, the hidden-run probe and its copies: ADR-0037); the OS-neutral
                         halves that every adapter shares: `packs` (folder order, file name, size, over an adapter's
                         `PackDisk`) and `field` (the watch's run, its `kx-field` thread and read deadline, a dead
                         watch, our own focus keeping the last box, the kind order, over an adapter's `Probe` and
                         `Reader`; P7), `worker` (P8: the one worker host, which starts a job on demand, one ask at
                         a time within a deadline, checked replies, a backoff and an idle stop, over an adapter's
                         `Spawner`), `pictures` (its `kx-pictures` session thread and media frames), `boxes` (P9: its
                         `kx-boxes` session thread and vault frames) and `mirror` (P9: the probe's steps, the copies
                         list and the checked delete, over an adapter's `MirrorDisk`), and `selection` (P10: the
                         watch's `kx-select` thread, its settle wait, look cap and read deadline, over a `SelProbe`)
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
                         (`WinReveal`, P8); the backup's places (`WinPlaces`, from `places.toml`: cloud folders by
                         environment, label and parts, removable drives, the folder list, writes in one step) and
                         the hidden-run probe (`WinHidden`, from `redirect.toml`; P9); the selection watch
                         (`WinSelectionWatch`, kinds from `selection.toml`) and the web link (`WinWebLink`, through
                         Explorer's desktop shell as the reveal, `explorer.rs`), with every UIA handler locked (P10)
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
                         `FakeBoxes`, `FakePlaces`, `FakeHidden` and `FakeRandom` script boxes, places, hidden runs
                         and random bytes (P9); `FakeSelectionWatch` and `FakeWebLink` script spots and reads and
                         record links (P10); kx-module-api's contract test uses it too
  kx-platform-net/      worker-only HTTP (WinHTTP); `keyxtend` must never depend on it (P11)
  kx-ui/                qml/ (views and parts), src/bridges/ (cxx-qt objects), the `ui` module and its hub, and the
                         EN/FR/AR translations (`i18n/*.ftl`, Fluent, compiled in; ADR-0017), in the keyboard's
                         language, the tray's in the system's (ADR-0020); the `keysTip` and `keysText` bridge calls
                         name a shortcut's keys in the keyboard's language, `layerPageText` the layer's page for
                         readers, and `PageDots.qml` draws the page dots the Next key and the Cycle design share
                         (P6); the chip lane in the top bar, its menu and the paged email list (`WordLane`,
                         `WordMenu`, `WordList`), over the hub's `words_json` (P7); the clipboard panel (`ClipPanel`
                         and its parts) over `clip_json`, its pictures through the `image://clip` provider, never
                         JSON, and its texts in `i18n/clip/` (P8); the Quick-fill panel (`KbQuickFill`, the
                         `QuickFill*` views and the shared `Panel*` parts) over `fill_json`, which holds shown
                         values and is never logged, with its texts in `i18n/fill/` (P9); the selection strip
                         (`SelStrip`, `SelChips`, `SelKey`) over `sel_json`, placed by `sel_at`, texts in `i18n/sel/` (P10)
  kx-theme/             theme tokens compiled in (tokens/: themes, shape, motion, icons, sounds), the look view,
                         WCAG contrast of what the keyboard draws, measured on the face's row height (`row_px`),
                         the high-contrast palette, and the strict reader kx-ui's build checks sounds with; no
                         Qt, so it tests everywhere (ADR-0016); Quick-fill's icons sit in `icons.toml` (P9)
  kx-mod-keyboard/      the board (`board.toml`: rows, widths, scan codes, roles), its boxes at any size, size
                         steps and presets, and the place on screen per DPI; typing: sticky modifiers, legends
                         and each click's route (ADR-0009); the face (board and state JSON, clicks, and the
                         board's row height, `row_px`, for the theme's contrast check) and the language key's
                         Carousel, Cycle and List; `[keyboard]` settings. The side roles `layer` (the Shortcuts
                         key) and `power` (asks the power module over the bus), the layer's generated `shortcut`
                         and `next` cells, the pure `layer.rs` (pages, cells, Next) and the `shortcut_pages`
                         setting (P6); the typed pieces it tells and the picked words it types (P7); the fill key
                         and each fill it types (`run/fill`, `face/fill`), never past `fill_late_ms` or to another
                         app after a Tab (P9); the selection chips' edit keys, sent with nothing held and every latch
                         kept (`run/edit.rs`, P10)
  kx-mod-layouts/       the Layouts service over the OS layouts (EN/FR/AR + any installed): the shown layout's
                         table (keys from `scans.toml`), the list in the OS order, step and pick; short names
                         in `[layouts]`; `LayoutChanged` names the language, told at start and on `LayoutAsk` (P7)
  kx-mod-mouse/         hold engine (Right-click, Grab), modifier+click (P4); its hook also runs while the pad
                         scrolls, to tell presses (`PointerPressed`) and move the badge (P5), and while an asker
                         (prediction, P7; the selection chips, P10) or an open panel (ADR-0040) wants presses told
  kx-mod-scroll/        the scroll pad: the tap machine (`pad`), the glide (`glide`), the stop rules
                         (`stops`) and the actor, over the `ScrollTarget` port; `[scroll]` (P5)
  kx-mod-power/         the Power key's module: an actor that opens the system's own power dialog over
                         the `PowerDialog` port at each click, each open on a short thread of its own,
                         and tells the keyboard whether it runs; no settings (P6)
  kx-mod-predict/       word prediction (P7): an actor on its `kx-predict` thread that follows the keyboard's
                         pieces and the focused box, tells the lane, asks the keyboard to type picks (`TypeWord`)
                         and learns, over the tracker (`track`), the scorer (`score`), the learner (`learn`), the
                         email chips (`email`) and the word-pack format (`pack`; the `build` feature's
                         `PackWriter` serves `cargo xtask words` only); it folds with `kx_module_api::text`; the
                         notebook's emails, wiped and never stored (`email/saved`, P9); `[predict]`
  kx-mod-clipboard/     clipboard history (P8, ADR-0034): the `kx-clip` actor over the watch, the kind rules
                         (`entry`, `media.toml`), the limits (`keep`), follow-the-clear and skipped apps (`follow`),
                         the sealed day folders (`store`), search (`find`), the panel and its page (`panel`,
                         `view`), insert (`insert`), the take Paste asks for (`actor/take`, P9) and the media and
                         files threads (`media`); `[clipboard]`
  kx-mod-vault/         Quick-fill (P9, ADR-0038): the `kx-vault` actor over the notebook sealed with the account
                         key, its kinds and labels tables (`kinds`), the entry rules (`entry`), the notebook (`book`)
                         and its blob (`store`), search (`find`), order and By label (`order`), Make one (`make`),
                         the panel and its pages (`panel`, `view`), fills (`fill`), Paste (`paste`), the backup box,
                         memory checks and Bring back (`backup`, jobs on its `kx-vault-box` thread) and the KeePass
                         import (`import`); `[vault]`
  kx-mod-selection/     selection chips (P10, ADR-0041): which keycaps show (`chips`), the strip's show and hide
                         machine (`strip`), the Translate link and its site table (`link`, `sites.toml`) and the
                         `kx-selection` actor over the watch and the link; `[selection]`
  kx-mod-voice/         voice UI + worker client (engine lives in the worker) (P11)
  kx-mod-snip/          capture overlay, editor, OCR client (P12)
  kx-crypto/            sealed files: XChaCha20-Poly1305 under a data key the `KeyGuard` port wraps
                         (DPAPI/Keychain/Secret Service); `SealedFiles` backs the `SealedStore` port (P7), and
                         `SealedShelfFiles` the `SealedShelf` port: folders of blobs under a root with its own
                         key, each tag naming the folder (P8); `MemoryStore` and `MemoryShelf` for a hidden run
                         (ADR-0037) and `OsRandom` for the `Random` port (P9)
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
                         `[g8_clip]`) and kx-gates-p9.toml (G23 on the product, `[g23]`, test values only) and
                         kx-gates-p10.toml (G19 on the product, `[g19_product]`); uses spike-core widely until its
                         gates are aimed at the product;
                         `capture-layouts` writes the fake's golden layout tables from the layouts installed on the PC
  kx-target-window/      test window that logs every character it receives (CI); a library plus a binary; reads
                         kx-target-window.toml; times its log with the QPC clock from kx-platform-windows;
                         `--layout` switches the session to an installed layout and puts the previous one back
                         when the window closes while it holds the keyboard (gates bring it to the front first);
                         a failed restore logs `# layout-kept`; a killed window leaves the session switched; `--fill`
                         fills it to scroll both ways, `--dpi-unaware` makes it unaware (P5), `--rich` RichEdit (P10)
  kx-test-icons/         the Qt Quick tests' QML plugin: kx-ui's icon provider over the icons the
                         texts test lays out, and the soft shadows' adopter, so the tests draw as
                         the app does
assets/                  fonts, icons and sounds with their licences, compiled into kx-ui (ADR-0016); the word
                         packs in `words/`, with their licence and credits, ship beside the program (P7)
spike/                   tested P1 code, moved out phase by phase (ADR-0014, docs/spike-move-map.md)
xtask/                   cargo xtask: tidy (architecture rules), dco (sign-off check), licences, dist (stub until P14),
                         stage (the release app with its Qt files, the word packs and the worker),
                         dev-cert / dev-install / check-uiaccess (uiAccess test builds), sounds (recipes, import,
                         join; format and RIFF walker from kx-theme), words (P7: source hashes in `words.toml`,
                         wordfreq lists checked against the Hunspell dictionaries; packs from kx-mod-predict's
                         writer, the index, the meter), sign, sbom (P14)
docs/
  adr/                   architecture decision records (never rewrite; supersede instead)
  architecture/          this file's detail sections, one topic each
  superpowers/specs/     design specs
  superpowers/plans/     implementation plans (one per phase or module)
  test-reports/          manual Windows test results
design/                  HTML mock-ups (reference only, never shipped)
.claude/skills/          project agent skills
```

## The kernel contract

The module trait, its lifecycle, services, events and settings: [docs/architecture/kernel.md](docs/architecture/kernel.md).

## Invariants (enforced by `cargo xtask tidy` in CI)

1. `kx-mod-*` crates depend only on `kx-module-api` and pure utility crates. They never depend on each other, on `kx-kernel`, or on any `kx-platform-*`.
2. Only `apps/*` choose platform adapters and modules. The never-shipped test tools in `tools/` may use both, and `xtask` (`build_tool`) may use a module: `cargo xtask words` writes the packs with kx-mod-predict's writer (tidy D2).
3. Only `kx-platform-*` crates may contain `unsafe`. Every other crate has `#![forbid(unsafe_code)]`. Every `unsafe` block has a `// SAFETY:` comment.
   - Two narrow exceptions: the cxx-qt bridge blocks in `kx-ui/src/bridges/` (ADR-0013), and the never-shipped test tools in `tools/` (ADR-0014).
   - These use `#![deny(unsafe_code)]` with a scoped allow instead of `forbid`. Tidy enforces this for `tools/` and for `kx-ui`, where rule F3 refuses `unsafe` outside `src/bridges/` and any block without a `// SAFETY:` comment.
4. The `keyxtend` app has **no HTTP/TLS dependency**. `cargo tree -p keyxtend` must not contain `reqwest`, `hyper`, `ureq`, `rustls` or `native-tls`, or `kx-platform-net`, which only the worker's `fetch` job uses (ADR-0042).
5. The `keyxtend` app never decodes untrusted images, audio or documents. Those crates may appear only in `keyxtend-worker`.
   - The worker is the only PNG and DIB decoder and the only reader of backup boxes and KeePass files, and tidy D4 (`banned_media`) keeps `png`, `fdeflate`, `age` and `keepass` out of the app's tree (ADR-0034, ADR-0038).
   - The worker's `voice` job alone hears the microphone and runs the speech engine, so D4 keeps `cpal`, `transcribe-cpp` and `transcribe-cpp-sys` out of the app's tree too (ADR-0042).
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
15. The worker's two P11 jobs stay apart (tidy F5, `apart`): no file of its `src/voice` names `kx-platform-net`, and none of its `src/fetch` names `cpal` or `transcribe-cpp`. F5 reads names only (`docs/security/keyxtend-worker.md`).

**CI** (`.github/workflows/ci.yml`):
- Windows builds and tests everything with Qt, runs the Qt Quick tests and checks the stage.
- Windows sets `KX_LIVE_CLIPBOARD=1`, so the adapter's live clipboard tests run there; elsewhere they are ignored, as they overwrite the real clipboard (`ci_opts_in_to_these_tests` pins the line).
- Windows also lints and tests the gate-only `gate-input` build and the adapter's `sim-input` in steps of their own; the stage then refuses that flag (ADR-0027).
- Windows also lints the app with the scroll module's feature off, so the scroll module stays optional.
- Linux leaves out the app and builds `kx-ui` without Qt until P15 (ADR-0016), and the worker without its default speech engine (ADR-0044).

## Cross-cutting patterns

- **Hexagonal (ports and adapters).** Logic depends on traits, and adapters implement them per OS. Swapping the toolkit, or Windows for macOS, touches adapters only.
- **State machines as enums** with explicit transitions, for the keyboard, hold engine, scroll and voice session. Time is always injected through a `Clock` trait, so tests are deterministic.
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

Window flags, the window guard, owners and drawing: [docs/architecture/qt-windows.md](docs/architecture/qt-windows.md).

## Modules and platforms

- [The scroll pad (P5, ADR-0029)](docs/architecture/scroll-pad.md)
- [Word prediction (P7, ADR-0032, ADR-0033)](docs/architecture/prediction.md)
- [Quick-fill (P9, ADR-0037, ADR-0038, ADR-0039)](docs/architecture/quick-fill.md)
- [Selection chips (P10, ADR-0041)](docs/architecture/selection.md)
- [Platform notes](docs/architecture/platforms.md): each concern on Windows, macOS and Linux.
