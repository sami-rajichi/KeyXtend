# Phase 6: shortcuts layer and Power key (v0.3)

## Goal
- The Shortcuts key swaps the side block for paged one-click shortcuts, each sent with the platform's own keys; the Power key opens the system's own power dialog.
- Spec §4.2, §8.5; ADR-0031; `docs/FEATURES.md` P6; the move map's P6 rows; `docs/items/P6.md`.

## Scope
- In: the `Shortcuts` and `PowerDialog` ports, their Windows adapters and fakes, `kx-mod-power`, the layer with pages and a Next key, 37 shortcuts, tips with keys, `[keyboard] shortcut_pages`, icons and texts, the fake's held keys (P3 note), G25 aimed at the product, the spike's G25 language helpers moved.
- Out: the Settings window's shortcuts page (P13); Win+V routing (P8); macOS and Linux tables (P15); user-made shortcuts (never, owner 2026-10-23); removing the test certificate (owner decides).

## Owner decisions (2026-10-23)
- The Shortcuts key is the toggle: lit while the layer shows, a second click goes back; no Back key.
- The layer stays after a shortcut; latched modifiers join every step, then let go.
- Pages the user arranges; a Next key with dots in the bottom-right cell wraps; 13 a page, 14 when alone.
- Chains (Copy all, Cut all, Delete all); F1–F12 as their own page; tips name the keys sent.
- Switch app keeps the system switcher open; Power stays, dialog only, never confirmed by tests.
- French Power name «Marche/Arrêt»; Win+V routing moves to P8; the 23 Lucide icons are approved.

## Design notes
- **Port `ports/shortcut.rs`:** `ShortcutId`, `KeyName`, `Step` (names held in order, the last tapped), `ShortcutInfo` (id and steps), `ShortcutError`, `Shortcuts` (`known()` lists the platform's ids with steps; `send(id, held)` holds the latched scans around every step), `ShortcutsKey` gated by `InjectInput`; the keyboard now requires it.
- **Port `ports/power.rs`:** `PowerDialog` (`open()`), `PowerError`, `PowerDialogKey` gated by the new capability `Power`; events `PowerAsked`, `PowerChanged { live }`, `PowerAsk`, all ungated as the keyboard publishes and hears them.
- **Windows:** `src/shortcut.rs` (pure: the table, names to virtual keys, steps to events in the app's layout, latches merged once) with `shortcuts.toml` (id → steps, or `call = "lock"` for `LockWorkStation`) and `keynames.toml` (name → virtual key); `shortcut/win.rs`; `src/power.rs` with `power/win.rs` (`Shell.Application` `ShutdownWindows` under an apartment-threaded `Com`); `WindowsPlatform::shortcuts()` and `power_dialog()`.
- **Fakes:** `FakeShortcuts` (scripted ids, records sends with their held scans, scripted refusal), `FakePowerDialog` (counts opens, scripted failure); the fake layout's `types_any` honours held keys.
- **`kx-mod-power`:** no settings; an actor thread hears `PowerAsked` and opens the dialog, tells `PowerChanged` on start, ask and stop, and shows `notice.power-failed` on an error.
- **Keyboard:** roles `layer` (Shortcuts) and `power` in `board.toml`, and `[layer] next = [3, 5]`; generated cells take roles `shortcut` and `next`, refused in the file. New `src/layer.rs` (pure: pages filtered by `known()`, empty pages dropped, capacity, cells in reading order, Next wraps) and `src/view/layer.rs`.
- **Setting `[keyboard] shortcut_pages`** (a new default needs no migration, as the file holds only changes): page 1 `copy, paste, cut, select-all, undo, redo, copy-all, cut-all, delete-all, find, save, print, emoji`; page 2 `switch-app, show-desktop, file-explorer, lock-pc, task-manager, close-window, new-tab, close-tab, reopen-tab, refresh, zoom-in, zoom-out`; page 3 `f1`–`f12`. The validator checks id form, no duplicates, the page capacity and `MAX_PAGES`.
- **Windows keys:** Ctrl+A/C/X/V/Z/Y/F/S/P/T/W, Ctrl+Shift+T, Alt+F4, F5, F1–F12, Ctrl+Alt+Tab, Win+D, Win+E, Win+period, Ctrl+Shift+Esc; zoom uses the number pad's + and −, which every layout has in one place and Word does not take for subscript; Lock PC calls `LockWorkStation` (its tip still shows Win+L).
- **kx-ui:** `qml/{Plate,Key,NamedFace,KbWindow}.qml` (the layer swap with a motion token), new `PageDots.qml`; `src/texts` composes a tip's keys in the keyboard's language (`res.combo`); `hub/bus.rs` wakes on `PowerChanged`.
- **Tokens:** `icons.toml [shortcuts]` (25 names); the Cycle design's dot sizes move to a shared `shape.toml [dots]` that the Next key also uses; `motion.toml` `moves.layer`.
- **Texts (EN, FR, AR):** `shortcut.<id>` ×37, `keyname.<name>` for named keys (letters, digits and F-keys print as themselves), `combo.join`, `combo.then`, `key.next` with page arguments, `tip.power-off`, `tip.no-shortcuts`, `notice.power-failed`, `module.power`; `label.power` becomes «Marche/Arrêt».

## Tasks
1. **Ports and events.** *Files:* `kx-module-api/src/{capability.rs, ports.rs, ports/shortcut*.rs, ports/power*.rs, notice.rs}`, contract tests. *Test first:* key ids and gates; event names unique; `Power` is its own capability; ids refuse bad forms. *Implement:* the types above. *Verify:* `cargo test -p kx-module-api`.
2. **Fakes and held keys.** *Files:* `kx-platform-fake/src/{shortcut.rs, power.rs, layout.rs}` with tests. *Test first:* sends are recorded with held scans; a scripted refusal comes back; Ctrl+letter types and Ctrl+digit does not. *Implement:* the two fakes; `types_any` with held keys. *Verify:* `cargo test -p kx-platform-fake`.
3. **Windows shortcut table, pure part.** *Files:* `kx-platform-windows/src/{shortcut.rs, shortcut/tests.rs, shortcuts.toml, keynames.toml}`. *Test first:* every entry's names are known; Select all on French AZERTY sends the key at QWERTY Q; Ctrl latched + Copy holds one Ctrl; a refusal releases every held key. *Implement:* parse, map, build events. *Verify:* `cargo test -p kx-platform-windows shortcut`.
4. **Windows services.** *Files:* `shortcut/win.rs`, `power.rs`, `power/win.rs`, `com.rs` (apartment start), `platform.rs`, `lib.rs`. *Test first:* `known()` lists the table; the services are created on another thread; COM starts apartment-threaded. *Implement:* send, lock call, `ShutdownWindows`. *Verify:* `cargo test -p kx-platform-windows`.
5. **`kx-mod-power`** (`kx-new-module`). *Files:* the crate, workspace members, CI `PACKAGES` and the scroll-off feature list, `docs/security/kx-mod-power.md`. *Test first:* the manifest needs only the dialog and `Power`; `PowerAsked` opens once; a failure gives the notice; stop tells `live: false`; a panic tells OFF. *Implement:* module and actor. *Verify:* `cargo test -p kx-mod-power`; `cargo xtask tidy` 0/0.
6. **Board and setting.** *Files:* `kx-mod-keyboard/{board.toml, defaults.toml, src/spec*.rs, src/settings*.rs}`. *Test first:* the new roles parse; a generated role in the file is refused; `[layer] next` must be a free side cell; the default pages pass; a 15th id, a duplicate or a bad id each refuse; property: no page list panics the validator. *Implement:* roles, `[layer]` and the setting. *Verify:* `cargo test -p kx-mod-keyboard spec settings`.
7. **The layer machine.** *Files:* `kx-mod-keyboard/src/layer.rs` with tests and property tests. *Test first:* toggle on and off; Next wraps; one page holds 14 and has no Next; unknown ids and empty pages drop; a smaller list moves the page back in range. *Implement:* the pure machine. *Verify:* `cargo test -p kx-mod-keyboard layer`.
8. **Face, plans and views.** *Files:* `src/{plan.rs, run.rs, face.rs, face/bus.rs, face/inner*.rs, view.rs, view/{board,layer,power}.rs}`, goldens, `tests/keyboard/{layer,power}.rs`. *Test first:* a shortcut sends its id with latches and they let go; the lone Win press is skipped; a dead key's wait ends; nothing repeats; opening the layer stops the pad; Power publishes `PowerAsked` only while live, else dims with `tip.power-off`; the board lists the page's cells and the state lights the toggle and the dot. *Implement:* the arms, state and redraws. *Verify:* `cargo test -p kx-mod-keyboard`.
9. **The app.** *Files:* `apps/keyxtend/{Cargo.toml, src/boot.rs, src/run.rs}`, `tests/life/{main,rig,ships}.rs`. *Test first:* `mod-power` is a default; the dialog reaches only the power module; the shortcuts service reaches the keyboard. *Implement:* the wiring. *Verify:* `cargo test -p keyxtend`; `cargo xtask stage`.
10. **UI hub and texts.** *Files:* `kx-ui/src/{hub/bus.rs, texts*.rs, bridges/ui/texts.rs}`, `i18n/{en,fr,ar}.ftl`, `tests/texts/files.rs`, `apps/keyxtend/tests/agree/{texts,icons}.rs`. *Test first:* `PowerChanged` wakes the keys; Copy all's keys read "Ctrl+A, Ctrl+C", French Shift reads «Maj», Arabic joins with «،»; every default id has a name in three languages and an icon or label; every default id has a Windows entry. *Implement:* the wake, the combo text and the texts. *Verify:* `cargo test -p kx-ui -p keyxtend`.
11. **Views, tokens and icons.** *Files:* `kx-ui/qml/{Plate,Key,NamedFace,PageDots,LangCycle}.qml`, `build.rs`, `kx-theme/tokens/{icons,shape,motion}.toml` and `src/{icons,shape*,drawn*}.rs`, `assets/icons/lucide/` (23 files), `assets/README.md`, `THIRD_PARTY.md`, `qmltest/tst_{layer,window,keys}.qml`, fixtures. *Test first:* the toggle is lit while the layer shows; a shortcut key shows its icon only and its tip says name and keys; F5 shows its text; the dots light the current page; the swap stops under reduced motion; contrast holds in light and dark. *Implement:* views and assets. *Verify:* `qmltestrunner` at both scalings; `cargo test -p kx-ui -p kx-theme`.
12. **Gates.** *Files:* `tools/kx-gates/src/{g25.rs, langs.rs, g10steps.rs, g22.rs}`, `kx-gates.toml`, `spike/core/src/langkey.rs`, `docs/spike-move-map.md`, `docs/security/dev-tools.md`. *Test first:* the moved helpers' tests; a config check. *Implement:* G25 clicks the product's Select all, Copy, Copy all, Paste and Undo in Notepad, opens Switch app and Win+V and closes each with Esc, and opens Power and closes it with Cancel. *Verify:* `cargo test -p kx-gates`; `cargo xtask tidy`.
13. **Docs.** `ARCHITECTURE.md`, README sections, `docs/security/{kx-mod-keyboard,kx-platform-windows,kx-ui,keyxtend}.md`, `docs/items/{P6,P8,P13}.md`.
14. **Review and security.** `kx-review` on every changed file, `kx-security-check`, `/code-review` and `/security-review`; fix every finding.
15. **Gates, manual check, done.** The controller runs G25 and G10 with a heads-up, then `kx-windows-manual-test` with the owner, then `kx-module-done` and the records.

## Edge cases
1. Toggle twice returns; the layer stays after a shortcut; restart opens on the usual side keys.
2. Next wraps from the last page; one page has no Next and holds 14.
3. Ids the platform lacks and pages left empty are skipped; nothing to show dims the Shortcuts key with `tip.no-shortcuts`.
4. A bad `shortcut_pages` resets alone with the settings notice.
5. Shift + New tab sends Ctrl+Shift+T; Ctrl + Copy holds one Ctrl; "Stay held" keeps the latch.
6. A lone-ready Win, then a shortcut: no lone Win press.
7. A chain sends its steps in order; a refused send releases held keys and shows the input notice.
8. AZERTY and Arabic layouts send the right keys for letters; zoom works on all three.
9. A dead key waits, then a shortcut: the wait ends; then Ctrl+digit keeps it (fake held keys).
10. A held shortcut key never repeats; the list panel open: shortcut keys still go to the app.
11. Opening the layer while the pad scrolls stops it; Right-click and Grab stay on.
12. Power module off: the key dims with a tip; the dialog cannot open: a notice.
13. Pages shrink while page 3 shows: the page comes back in range.
14. Lock PC uses the OS call, never an injected Win+L.

## Security
- Capabilities: `InjectInput` (keyboard, unchanged) covers `Shortcuts`; `Power` is new and only `kx-mod-power` holds it.
- Data: ids and key names are public; no typed text; latches stay redacted in logs.
- Threats: a tampered settings file names only ids, never keys, so it cannot inject new key sequences; held keys are released after a partial send; the dialog is never confirmed by code; a power call that fails leaves only a notice.

## Manual Windows check
- Open the layer, page through all three pages and back to page 1; the dots and the lit toggle.
- In Notepad: Copy all, then Paste; Undo and Redo; Delete all. In Chrome: New tab, Close tab, Shift + New tab, Zoom in and out, Refresh.
- Switch app, Show desktop, File Explorer, Emoji, Task Manager; Power opens the dialog and Cancel closes it.
- Tips in EN, FR and AR; Lock PC only if the owner wants to try it.

## Done when
- The Definition of Done in `docs/roadmap.md` holds.
- G25 passes on the product in Notepad, and the owner's manual check passes.
