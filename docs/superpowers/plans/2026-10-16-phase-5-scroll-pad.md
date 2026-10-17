# Phase 5: scroll pad (v0.3)

## Goal
- The round pad scrolls what the user last pointed at, gliding at two speeds, with an outline on the target and the scroll in the badge.
- Spec §5.2, §3.2, §3.3; `docs/FEATURES.md` P5; ADR-0029; the move map's P5 row; `docs/items/P5.md`.

## Scope
- In: `kx-mod-scroll`, the `ScrollTarget` port and its Windows adapter, the pad (placement B, speed dots, press colours), the outline, the badge's scroll part, our lists paged by the pad, the mouse hook while scrolling, P5's hook notes, and G6 aimed at the product.
- Out: the Settings window's steppers for `[scroll]` (P13); macOS and Linux adapters (P15); Word in any gate (owner, 2026-10-16); Dependabot #9 (its own session).

## Owner decisions (2026-10-16)
- Stops: third tap, Stop, end of content, a press outside our windows, 2 minutes after the last pad tap, the target closing.
- A smooth glide; slow 3 and fast 10 lines a second, both settings.
- The outline shows while the pointer is on the pad and while scrolling; our lists turn one page per tap.
- Arrows centred in the ring (B); speed dots inside Stop; every feature built once for all platforms.

## Design notes
- **Ports (`kx-module-api/src/ports/scroll.rs`):** `Dir` (up, down, left, right), `ScrollConfig`, `Aim` (point, on the pointer, box, ours, target id), `Reach` (more, end, unknown), `ScrollError`, `ScrollTarget` (`open(config)` gives a `ScrollRun`), `ScrollRun` (`aim`, `turn(aim, dir, lines)`, `reach`; drop closes), `ScrollTargetKey` with the new capability `Scroll`.
- **Bus events (same file):** `ScrollTap`, `ScrollStop`, `ScrollPeek` (pointer on the pad or off), `ScrollChanged` (live, direction, fast), `ScrollAimed` (the box to outline, or none), `ScrollPage` (back or on, to our open list). In `ports/assist.rs`: `PointerPressed` (ours or not), ungated; it carries no point.
- **`FaceAction::Pad`** (a direction or Stop) and `FaceAction::PadHover` in `ports/face.rs`.
- **`kx-mod-scroll`:** `pad.rs` (the tap machine), `glide.rs` (lines owed per tick, remainders kept), `stops.rs` (end, press, limit, gone), `actor.rs` (ticks only while scrolling, on the injected clock), `settings.rs`, `manifest.rs` (requires `ScrollTargetKey`; capability `Scroll`).
- **Settings `[scroll]`:** `slow = 3`, `fast = 10` (lines a second, 1–60, fast above slow), `limit_s = 120` (0 = none, else 10–1800), `stop_at_end = true`, `stop_on_press = true`, `tick_ms = 16` (8–100), `check_ms = 250` (100–2000), `read_ms = 300` (100–2000), `edge_px = 8` (2–64).
- **Windows (`kx-platform-windows/src/scroll*`):** `scroll.rs` (pure: lines to wheel delta from the wheel settings, remainders, route choice, the fallback rule, history maths), `scroll/win.rs` (post, send, hung check, wheel settings), `scroll/uia.rs` (own COM thread: element, box, percent, `ScrollPattern` steps, each read capped by `read_ms`), `scroll/entry.rs` plus the guard's entry watch (`GetMouseMovePointsEx`).
- **UI ports (`kx-platform/src/ui.rs`):** `WindowGuard::watch_entries(title)` (keep the last point outside that window) and `WindowGuard::lay_under(title, under)` (just under the keyboard).
- **`kx-ui`:** `qml/{DPad,Badge,Overlay,Outline}.qml`, `src/hub/{scroll,assist,bus}.rs`; the overlay is made while scrolling too.
- **Tokens:** `shape.toml [dpad]` `reach = 0.5` (replaces `icon_pad`), `dot_px`, `dot_gap_px`, `dot_off`; `shape.toml [outline]` `px = 2.0`; `motion.toml` `moves.outline`; the outline takes `ring` (`hover_line` in high contrast).
- **Texts (EN, FR, AR):** `badge.slow`, `badge.fast`, `tip.scroll-off`, `notice.scroll-none`, `notice.scroll-end`, `notice.scroll-point`, `window.outline`, `module.scroll`.

## Tasks
1. **Ports and events.**
   - *Files:* `kx-module-api/src/{capability.rs, ports.rs, ports/scroll.rs, ports/scroll/tests.rs, ports/assist.rs, ports/face.rs}`, contract tests.
   - *Test first:* key id and capability gate; each event's name and gate; `Dir`'s opposite and axis; `FaceAction::Pad` round-trips its JSON.
   - *Implement:* the types and traits above.
   - *Verify:* `cargo test -p kx-module-api` passes.
2. **The `kx-mod-scroll` crate and `[scroll]`** (`kx-new-module`).
   - *Files:* the crate, `defaults.toml`, `settings.rs` with tests, `manifest.rs`, workspace members, CI's `PACKAGES`, `docs/security/kx-mod-scroll.md`.
   - *Test first:* defaults are consistent; every bound refuses; fast must exceed slow.
   - *Implement:* the manifest, settings and `ScrollConfig` from them.
   - *Verify:* `cargo test -p kx-mod-scroll`; `cargo xtask tidy` 0/0.
3. **The tap machine and the glide.**
   - *Files:* `kx-mod-scroll/src/{pad.rs, glide.rs, stops.rs}` with tests and property tests.
   - *Test first:* §5.2's taps; Stop; each stop rule; properties: lines owed over any tick pattern equal speed × time within one tick, a long gap owes at most `check_ms`, and every stop leaves the machine idle.
   - *Implement:* pure enums and maths on `Mono`.
   - *Verify:* `cargo test -p kx-mod-scroll` passes.
4. **The fake target.**
   - *Files:* `kx-platform-fake/src/scroll.rs` with tests.
   - *Test first:* scripted aims and reaches come back in order; turns are recorded; a dropped run closes.
   - *Implement:* `FakeScrollTarget`.
   - *Verify:* `cargo test -p kx-platform-fake`.
5. **The module's runtime.**
   - *Files:* `kx-mod-scroll/src/{lib.rs, actor.rs, actor/notes.rs}`, `tests/scroll/`.
   - *Test first:* ticks run only while scrolling; notices for no target, nothing to scroll and already at the end; peek outlines without scrolling; our list pages; a retarget moves the outline; the start order does not matter; one stats line holds numbers only.
   - *Implement:* the actor, the bus handlers and publishers.
   - *Verify:* `cargo test -p kx-mod-scroll`.
6. **The keyboard's pad.**
   - *Files:* `kx-mod-keyboard/{board.toml, src/spec.rs, src/view*.rs, src/face*.rs, src/panel.rs}`, `tests/keyboard/pad.rs`.
   - *Test first:* the pad is live while the scroll module is; it dims with `tip.scroll-off` otherwise; taps publish `ScrollTap` or `ScrollStop`; the state JSON carries direction and speed; `ScrollPage` turns the open list's page.
   - *Implement:* the pad actions, state and bus wiring.
   - *Verify:* `cargo test -p kx-mod-keyboard`.
7. **The mouse module while scrolling, and P5's hook notes.**
   - *Files:* `kx-mod-mouse/src/{engine.rs, actor.rs, actor/notes.rs}`, `kx-platform-windows/src/hook/{stats.rs, host*.rs}`, the hook's property tests.
   - *Test first:* scrolling keeps the hook on and moves noted; a press publishes `PointerPressed` with `ours`; the user's events heard inside our send are counted; the inside-send property mode drops an echo.
   - *Implement:* the `scrolling` input to `needs_hook`, the event, and the stats count.
   - *Verify:* `cargo test -p kx-mod-mouse -p kx-platform-windows hook`.
8. **Windows adapter, pure part.**
   - *Files:* `kx-platform-windows/src/{scroll.rs, scroll/tests.rs, scroll/entry.rs}`.
   - *Test first:* lines to delta for 3 lines, 1 line and page settings; remainders never drift; the pointer on the target picks send, else post; the fallback trips after `check_ms` of still percents short of the end; history points with negative coordinates; a history miss pushes the entry point `edge_px` past the nearest edge, on screen.
   - *Implement:* the pure helpers, with the routes moved from kx-gates' `scroll.rs`.
   - *Verify:* `cargo test -p kx-platform-windows scroll`.
9. **Windows adapter, OS part.**
   - *Files:* `kx-platform-windows/src/{scroll/win.rs, scroll/uia.rs, window/guard.rs, window/win.rs, platform.rs}`, `kx-platform/src/ui.rs`, `kx-platform-fake/src/ui.rs`.
   - *Test first:* over the test window: a posted step reaches it; a hung window gets none; UIA reads a box and percent with the cap; the guard keeps the last outside point; `lay_under` puts a window just under another.
   - *Implement:* `WindowsPlatform::scroll_target` and the two guard calls.
   - *Verify:* `cargo test -p kx-platform-windows`.
10. **The app.**
    - *Files:* `apps/keyxtend/{Cargo.toml, src/boot.rs, src/run.rs}`, `tests/life/`.
    - *Test first:* `mod-scroll` is a default; the `Scroll` grant reaches only the scroll module; quitting while scrolling stops it.
    - *Implement:* the wiring.
    - *Verify:* `cargo test -p keyxtend`; `cargo xtask stage`.
11. **The UI hub.**
    - *Files:* `kx-ui/src/hub/{scroll.rs, assist.rs, bus.rs}`, `src/bridges/ui*.rs`, `tests/hub/scroll.rs`.
    - *Test first:* the scroll state and outline box reach QML with one wake; the badge holds a mode and a scroll part; pad hover publishes `ScrollPeek` once per change.
    - *Implement:* the state, JSON and wakes.
    - *Verify:* `cargo test -p kx-ui --test hub`.
12. **The views, tokens and texts.**
    - *Files:* `kx-ui/qml/{DPad,Badge,Overlay,Outline,App}.qml`, `build.rs`, tokens, `i18n/{en,fr,ar}.ftl`, `qmltest/{tst_dpad,tst_outline,tst_badge}.qml`, fixtures.
    - *Test first:* arrows sit at `reach`; dots show the speed; a pressed arrow takes the palette's `press` and `press_ink`; the badge shows the arrow and word; the outline is four click-through edges in the ring colour (Highlight in high contrast), made only when needed; Arabic words read right to left.
    - *Implement:* the views and assets.
    - *Verify:* `qmltestrunner` at both scalings; `cargo test -p kx-ui -p kx-theme`.
13. **Gates aimed at the product, spike copies deleted.**
    - *Files:* `tools/kx-gates/src/{g6,scroll,g8*,g9,g10*}.rs`, `kx-gates-p5.toml`, `g6track` deleted, `docs/spike-move-map.md`, `docs/security/dev-tools.md`.
    - *Test first:* config checks; G6's judging of speed, stop, end, press-stop and sideways from pictures and percents.
    - *Implement:* G6 drives the test build in Notepad, Chrome and Explorer; G8 adds a scroll run; G9 checks the outline's flags; G10 the pad.
    - *Verify:* `cargo test -p kx-gates`; `cargo xtask tidy`.
14. **Docs.** `ARCHITECTURE.md`, the README's scroll section, `docs/security/{kx-platform-windows,kx-ui,keyxtend,kx-mod-mouse}.md`, `docs/items/P5.md`.
15. **Review and security.** `kx-review` on every changed file, `kx-security-check`, `/code-review` and `/security-review`; fix every finding.
16. **Gates, manual check, done.** The controller runs G6, G8, G9 and G10 with a heads-up, then `kx-windows-manual-test` with the owner, then `kx-module-done` and the records.

## Edge cases
1. Tap, tap, tap; another direction mid-scroll goes slow; Stop at either speed.
2. The pointer never left the keyboard since the start: a notice, nothing scrolls.
3. Nothing scrollable there, or already at the end: a notice, and it stops.
4. The end is reached: it stops; with `stop_at_end` off it runs to the limit.
5. A press outside our windows stops it; a press on the pad does not; `stop_on_press` off keeps it going.
6. The limit counts from the last pad tap; 0 means none.
7. The target closes, or the app hangs (no steps pile up, and it resumes after).
8. The pointer moves onto another window while scrolling: retarget and outline; back on the keyboard keeps the last point.
9. The pointer on the target sends; elsewhere posts; never both in one tick.
10. A window that ignores wheel steps falls back to `ScrollPattern`.
11. Our list: one page per tap, ◀▲ back, ▶▼ on; no list open does nothing.
12. Wheel settings of 0 lines or one page; sideways uses the character setting.
13. Screens left of or above the main one; a history miss stays on screen; 125 % and mixed DPI.
14. Scroll module off: the pad dims with a tip. Mouse module off: it scrolls without the badge or press-stop.
15. Any start order; a quit or restart while scrolling leaves no outline and releases the hook.
16. Right-click or Grab on while scrolling: a hold still right-clicks, and its press stops the scroll.
17. The badge near a screen edge; the outline of a target partly off-screen or under the keyboard.
18. A provider's bad box (empty, huge, negative) or bad percent (NaN, infinite, below 0) is refused; one above 100 reads as 100 (G6 run 1).

## Security
- Capabilities: `Scroll` (new: the scroll module only; the platform provides it); the mouse module's `PointerHook` unchanged.
- Data: pointer points and target boxes, never logged; stats lines hold counts only.
- Threats: runaway scrolling (the limit), a hung or hostile target (no pile-up, capped UIA reads, checked values), a confused deputy (steps go only where the user pointed), and the outline never taking focus or clicks.

## Manual Windows check
- Notepad, Chrome and Explorer: point, tap ▼ (slow), again (fast), again (stop); ▲, ◀ and ▶ where the content is wide.
- The outline on pad hover and while scrolling; the badge's arrow and word; Stop and its dots.
- End of page, a click in the page, and the 2-minute limit (shortened in settings) each stop it.
- The language list (List design) turns a page per tap; the speeds tuned by hand.

## Done when
- The Definition of Done in `docs/roadmap.md` holds.
- G6 passes on the product in Notepad, Chrome and Explorer, sideways included where the content is wide; idle CPU and memory stay within ADR-0021's budget with the outline shown.
