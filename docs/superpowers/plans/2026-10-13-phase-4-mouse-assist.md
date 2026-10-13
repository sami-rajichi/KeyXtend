# Phase 4: mouse assist (v0.2)

## Goal
- Right-click by a still hold, Grab (drag-and-drop and text selection) and modifier + click, with the ring, burst, badge and realistic sounds.
- Spec §3.2, §5.1, §5.3; `docs/FEATURES.md` P4; ADR-0027; the move map's P4 rows.

## Scope
- In: `kx-mod-mouse`, the `PointerHook` port and its Windows hook, the overlay, the sounds and their `Speaker` port, the side keys Right-click and Grab, the health check, the hook budget, and the gates G5, G12, G17, G18 and the hand try aimed at the product.
- Out: the Settings window's pages (P13); the scroll pad and the last-point-outside target (P5); handing a Grab selection to the Selection helper (P10); macOS and Linux adapters (P15).

## Owner decisions (2026-10-13)
- Grab grabs at the next press, with no wait; a click drops; Right-click comes back after.
- Hold time 1.5 s, settable from 0.5 to 3.0 s; the ring's waves always last the hold less 150 ms, in the theme's ring colour.
- The badge shows while a mode is on, and hides over our windows. No banner when a mode turns on.
- Right-click is remembered across restarts; Grab never is.
- Sounds: one realistic, soft key-press sound for every theme, on every key (not the top bar), once per press; a soft mouse click on a hold's right-click, a pick-up and a drop sound.
- All sounds start on, at our own soft level under the Windows volume, picked by ear from candidates.
- Dependabot #9 waits for its own session.

## Design notes
- **Ports (`kx-module-api`):**
  - `ports/pointer.rs`: `Button`, `PointerEvent` (down with `ours` and the screen's DPI, up, move, Esc), `PointerAct` (move, down, up, key down, key up by `Scan`), `PointerEngine` (`on` inside the callback fills a lent buffer; `settle` after it; `deadline`), `PointerHook` (`start(engine, HookConfig)` gives a `HookRun`), `HookRun` (`wake`, `stats`; drop stops), `HookStats`, `HookError`, `PointerHookKey` (capability `PointerHook`).
  - `ports/assist.rs`, bus events: `AssistToggle` (keyboard to mouse), `AssistChanged` (mouse to all: Right-click, Grab off/armed/carrying, live), `AssistAsk`, `AssistNote` (hold start with its timing, hold end, fired, picked, dropped, cancelled), `PointerMoved`, and `LatchesChanged`, `LatchesUsed`, `LatchesAsk` gated by `InjectInput`.
  - Modules talk only over the bus, so the keyboard and the UI start without the mouse module.
- **`kx-mod-mouse`:** the engine (moved from `spike/core/src/hold.rs`), an actor thread that starts the hook only while needed and turns engine notes into bus events, the `[mouse]` section, and the stats log line (numbers only).
- **`kx-platform-windows`:** `hook.rs` (pure: event mapping, accept rules, the queue behind our echo, the timer, the histogram, the health decision) and `hook/win.rs` (thread, both hooks, `WindowFromPoint` with our process id, the per-press DPI, `SendInput`). Moved from kx-gates' `hookhost`, `hookstate`, `hookio` and `mouse` (normalise).
  - A second tag, `REPLAY`, marks the engine's input, so kx-gates' guard can still drop a hand's input that we replay.
  - `release_now` for the panic hook.
- **`Speaker` port** (`kx-platform/src/ui/sound.rs`, in `UiPorts`): `prepare(clips)`, `play(id, gain)`. Windows: an XAudio2 actor thread (COM MTA) with a small voice pool per clip; the engine stops after `[ui] sound_idle_ms`, and a changed default device rebuilds it. Fake: records plays.
- **Sounds as data:** `crates/kx-theme/tokens/sounds.toml` (event to file and gain), files in `assets/sounds/` with their licences; `kx-ui`'s build checks each is PCM16 mono 48 kHz and embeds the samples, so nothing parses audio at run time.
- **Overlay (`kx-ui`):** `Overlay.qml` (one click-through window, made on first need, unowned), `HoldRing.qml` (from the spike's `RingFace.qml`), `Badge.qml`; the hub keeps the assist state, the notes and the last pointer point, and wakes Qt (`Wake::Assist`); pointer moves coalesce to one wake in flight.
- **Keyboard:** side keys get `role = { assist = "rclick" | "grab" }`, lit from `AssistChanged`, dimmed with a tip while the mouse module is not live; a tap publishes `AssistToggle`. The latches publish `LatchesChanged` and handle `LatchesUsed` and `LatchesAsk`.
- **Settings `[mouse]`:** `rclick = false`, `hold_ms = 1500` (500–3000), `still_px = 7` (logical, scaled by the press's DPI), `ring_after_ms = 150` (below the shortest hold), `health_ms = 1000`, `health_slack_ms = 500`, `echo_ms = 250`.
- **Settings `[ui]`:** `sound_key`, `sound_rclick`, `sound_pick`, `sound_drop` (true), `sound_volume = 50` (0–100), `sound_idle_ms = 3000`.
- **Tokens:** `shape.toml [badge]` (padding, icon, gap, text size, edge margin), `motion.toml` `badge` fade; the ring's existing tokens; `sounds.toml`.
- **Texts (EN, FR, AR):** `tip.rclick`, `tip.grab`, `tip.mouse-off`, `badge.holding`, `window.pointer`, `module.mouse`, notice `mouse-hook-failed`.
- **Test build:** the app's `gate-input` feature adds `--sim-input`: the engine then acts only on the gate tool's tagged input, never on the hand's. Stage refuses an exe that holds it.

## Tasks
1. **Sound candidates, chosen by ear.**
   - *Files:* `xtask/src/sounds.rs`, `assets/sounds/recipes.toml`, a listening page (Artifact), then the chosen files in `assets/sounds/`, `THIRD_PARTY.md`.
   - *Test first:* a recipe renders to the same bytes twice, as a PCM16 mono 48 kHz WAV of its length that never clips.
   - *Implement:* recipe-made candidates plus CC0 recordings (Kenney.nl first; each download and `kx-licence-check` asks the owner). The owner picks key, right-click, pick-up and drop.
   - *Verify:* `cargo test -p xtask sounds`; the owner's picks recorded.
2. **Ports.**
   - *Files:* `kx-module-api/src/ports/{pointer,assist}.rs`, `ports.rs`, the contract tests.
   - *Test first:* key ids and capability gates; each event's name and gate; a `HookRun` drop is the only stop.
   - *Implement:* the types and traits above.
   - *Verify:* `cargo test -p kx-module-api`.
3. **The `kx-mod-mouse` crate and `[mouse]`** (`kx-new-module`).
   - *Files:* the crate, `defaults.toml`, `settings.rs`, `manifest.rs`, workspace, CI's Linux list, `docs/security/kx-mod-mouse.md`; the `kx-new-module` skill's wrong `modules.rs` path becomes `boot.rs`.
   - *Test first:* defaults are consistent; each bound refuses; `ring_after_ms` must stay below the shortest hold.
   - *Implement:* the manifest (requires `PointerHookKey`; capabilities `PointerHook`, `InjectInput`).
   - *Verify:* `cargo test -p kx-mod-mouse`; `cargo xtask tidy`.
4. **The engine.**
   - *Files:* `kx-mod-mouse/src/engine.rs` with `engine/tests.rs` and property tests.
   - *Test first:* every spike case, the roadmap's edge cases, and Grab at once; properties: after `reset` no button or modifier stays down, and every swallowed press gets its release.
   - *Implement:* the §5.1 machine with latched modifiers by `Scan`, the DPI-scaled still radius, and notes for the UI.
   - *Verify:* `cargo test -p kx-mod-mouse engine`.
5. **The fake hook.**
   - *Files:* `kx-platform-fake/src/pointer.rs`.
   - *Test first:* scripted events reach the engine in order, its acts are recorded, and `wake` runs `settle`.
   - *Implement:* `FakePointerHook` on the test's thread with the fake clock.
   - *Verify:* `cargo test -p kx-platform-fake`.
6. **The module's runtime.**
   - *Files:* `kx-mod-mouse/src/{lib,actor,notes}.rs`, `tests/mouse/`.
   - *Test first:* the hook runs only while needed; toggles follow §5.1; Right-click is saved and comes back; latches reach the engine and `LatchesUsed` follows the click; the start order does not matter; one stats line logs no point.
   - *Implement:* the actor, the bus handlers and publishers, and a notice when the hook cannot start.
   - *Verify:* `cargo test -p kx-mod-mouse`.
7. **The keyboard's side keys and latches.**
   - *Files:* `kx-mod-keyboard/{board.toml, src/spec.rs, src/face*.rs, src/latch.rs}`, `tests/keyboard/assist.rs`, the three `.ftl` files.
   - *Test first:* the keys light from `AssistChanged`, dim with `tip.mouse-off` when not live, and publish `AssistToggle`; latches publish and clear on `LatchesUsed` in Release mode only.
   - *Implement:* the `assist` role and the latch events.
   - *Verify:* `cargo test -p kx-mod-keyboard`.
8. **The Windows hook.**
   - *Files:* `kx-platform-windows/src/{hook.rs, hook/win.rs, hook/tests.rs, keys.rs, platform.rs}`, `Cargo.toml` features.
   - *Test first, over a fake OS:* injected input never acts; ours passes untouched; let-through input waits behind our echo; the timer never fires early; the histogram's p99; the health check reinstalls once per period; stopping lets go.
   - *Implement:* the thread, both hooks, the `REPLAY` tag, `release_now`, and `WindowsPlatform::pointer_hook`.
   - *Verify:* `cargo test -p kx-platform-windows hook`.
9. **The app.**
   - *Files:* `apps/keyxtend/{Cargo.toml, src/boot.rs, src/run.rs, src/quiet.rs, src/args.rs}`, `tests/life/`, `xtask/src/stage.rs`.
   - *Test first:* `mod-mouse` is a default; `--sim-input` exists only with `gate-input`; the panic hook calls `release_now`; stage refuses an exe holding the flag.
   - *Implement:* the wiring.
   - *Verify:* `cargo test -p keyxtend`; `cargo xtask stage`.
10. **The `Speaker` port and XAudio2.**
    - *Files:* `kx-platform/src/ui/sound.rs`, `kx-platform-windows/src/{sound.rs, sound/win.rs, sound/tests.rs}`, `kx-platform-fake/src/sound.rs`.
    - *Test first:* the pool overlaps plays; the engine stops when idle; a lost device rebuilds once per play; gain follows the volume.
    - *Implement:* the port, the adapter and the fake.
    - *Verify:* `cargo test -p kx-platform-windows sound`.
11. **Sounds as data.**
    - *Files:* `kx-theme/tokens/sounds.toml`, `kx-theme/src/assets.rs`, `kx-ui/build.rs`.
    - *Test first:* each event names a shipped file; each file is PCM16 mono 48 kHz, short and licensed; a bad file fails the build.
    - *Implement:* the bundle and the embedded samples.
    - *Verify:* `cargo test -p kx-theme -p kx-ui`.
12. **Playing sounds.**
    - *Files:* `kx-ui/{defaults.toml, src/settings.rs, src/hub.rs, src/hub/sound.rs, src/bridges/keyboard/frame.rs}`, `tests/hub/sound.rs`.
    - *Test first:* a key press plays once after its frame, a repeat and a top-bar button never; each mouse note plays its sound; a sound switched off or at volume 0 never plays.
    - *Implement:* the hand-off to the `Speaker`, which never blocks.
    - *Verify:* `cargo test -p kx-ui --test hub`.
13. **The overlay.**
    - *Files:* `kx-ui/qml/{Overlay,HoldRing,Badge}.qml`, `App.qml`, `build.rs`, `src/hub/assist.rs`, `src/bridges/ui/wake.rs`, tokens, the `.ftl` files, `qmltest/tst_overlay.qml`, and the moved `tst_ring.qml` with its look fixtures.
    - *Test first:* the ring's waves last the hold less `ring_after_ms` and take the ring colour in every theme; reduced motion keeps the ring; the badge shows per mode, hides over our windows and stays on screen; the window is click-through, topmost and made only when needed.
    - *Implement:* the window, its pieces and the hub side, and the pointer size from `SystemLook`.
    - *Verify:* `qmltestrunner` (both scalings); `cargo test -p kx-ui`.
14. **Gates aimed at the product, spike copies deleted.**
    - *Files:* `tools/kx-gates/src/{g5,g12,g17,g18,hand,g6,simuser,mouseguard,config,cfgfile}.rs`, `kx-gates-p4.toml`, `spike/core` (hold out, `Pt` kept), `spike.toml`, `spike/qt-kb` ring parts, `docs/security/dev-tools.md`.
    - *Test first:* config checks; the guard drops `REPLAY` input unless the gate drives the engine; under uiAccess the config is read only beside the exe.
    - *Implement:* G5, G17 and G18 drive the `gate-input` build; G18 latches by the product's own keys; G12 clicks under the overlay and times its frames with PresentMon; `hand` reads the stats lines; G6 tracks the pointer by polling until P5; G8 adds the hook's p99.
    - *Verify:* `cargo test -p kx-gates`; `cargo xtask tidy`.
15. **Review and security.**
    - `kx-review` on every changed file, `kx-security-check` (new `kx-mod-mouse.md`; updates to `kx-platform-windows.md`, `kx-ui.md`, `keyxtend.md`), `/code-review` and `/security-review`; fix every finding.
16. **Gates, manual check, done.**
    - The controller runs G5, G12, G17, G18 and G8 with a heads-up, then `kx-windows-manual-test` with the owner, then `kx-module-done` and the records.

## Edge cases
1. A press on our own window is never held; a held Backspace repeats with Right-click on.
2. A 6 px and an 8 px move at 100 %, and the same at 125 %.
3. A release exactly at the hold time counts as a hold.
4. A second button during a hold replays the held press first.
5. The app panicking during a carry lets the button go.
6. A hold over an admin window (manual, with the owner's yes).
7. A hung target still gets its right-click when it wakes (G5 case).
8. Esc, the Grab key again, or a click on our window cancels a carry.
9. A mode switch during a hold replays the held press.
10. Right-click toggled during a carry cancels it.
11. Shift with a hold gives Shift + right-click; Ctrl latched before Grab stays down until the drop.
12. A latch changed during a click keeps the click's own modifiers.
13. In "Stay held", latches stay after the click.
14. Injected input from another program never acts.
15. The hook removed by Windows comes back; a false alarm reinstalls at most once per period.
16. The hook cannot start: a notice, and the keys stay off.
17. Mouse module off or failed: the keys dim with a tip, and typing is unchanged.
18. Either start order of keyboard and mouse.
19. A late timer fires the hold on the next event; a huge hold time or a far move never overflows.
20. Two screens with different DPI.
21. The badge near a screen edge stays on screen; a large Windows pointer pushes it clear.
22. No audio device, or one unplugged: one log line, no notice storm.
23. Fast typing overlaps key sounds; after `sound_idle_ms` the audio engine stops.
24. Quit, or a mouse-module restart, during a carry lets go.

## Security
- Capabilities: `PointerHook` and `InjectInput` (mouse); none new for the UI.
- Data: pointer points and modifier scan codes, never logged; the stats line holds numbers only.
- Threats: a confused deputy (only physical input acts; `gate-input` never ships); a stuck button (every stop and the panic path let go); the hook timing out (the health check); a hostile sound file (build-time checks, compiled in).

## Manual Windows check
- Right-click: its light, the badge, a still hold in Explorer and Notepad (ring, burst, click sound, menu), short clicks and drags unchanged, the hold at 0.5 s and 3.0 s.
- Grab: move a file to a folder, select text in Notepad, cancel with the Grab key; Right-click comes back.
- Modifiers: Shift + click extends a selection; Ctrl + Grab copies a file.
- Sounds: the key sound, soft, not on the top bar; the mouse sounds.
- Restart keeps Right-click; a hold over an admin window; a held Backspace repeats.

## Done when
- The Definition of Done in `docs/roadmap.md` holds.
- G5, G12, G17 and G18 pass on the product; the hook's p99 is at most 1 ms; idle CPU stays about 0 with Right-click on and a still mouse; idle memory stays within ADR-0021's budget with the overlay made.
