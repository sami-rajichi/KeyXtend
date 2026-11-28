# Phase 12, part 2: Snip after the owner's hands-on check

## Goal
- Fix what the owner's check found on 2026-11-26 and build the changes they chose, then resume the check at the video Speed step.
- Design: `docs/superpowers/specs/2026-11-23-snip-design.md`, updated here; ledger `.superpowers/sdd/2026-11-23-phase-12-snip/progress.md` (T22 entries of 2026-11-26).

## Scope
- In: the editor coming back darker; a maximised editor not remembered; the delay before the bar and before the editor; an adjustable frame with handles after a Region's second click and after Crop's second click, with a confirm key in the editor; videos without an editor, Play and Speed on the strip; G24 and the docs to match.
- Out: playing or trimming a video inside KeyXtend (a later feature with its own ADR, as ADR-0004 keeps media decoding out of the app); handles on Window and Full screen picks.

## Design notes
- **Crates:** `kx-mod-snip` (flow, editor model, view), `kx-module-api` ports (`SnipClick`, strip and editor keys), `kx-ui` (hub, QML `SnipOverlay`, `SnipShade`, `SnipStrip`, `SnipEditor`, `SnipCanvas`, `SnipCrop`, `SnipTools`, `SnipBar`, `Glow`), `tools/kx-gates` (G24).
- **The frame:** a dashed box with eight handles, four sides and four corners. A click on a handle picks it up, the pointer moves that side or corner, and a second click sets it, never a drag. A click away from the frame starts a new region (Region) or does nothing (Crop).
- **Region:** after the second corner the frame shows on the frozen screen with the strip; Copy, Edit, Save and Record take the frame as it is then. A video region keeps to its screen as today.
- **Crop:** after the second click the frame shows over the whole picture; the tools row shows **Crop ✓** and **Cancel ✗**. ✓ cuts as one undoable step, and Undo brings the whole picture back. ✗, Esc or another tool drops the frame.
- **Videos:** after Stop, or a recording ending by itself, the strip shows the length, **Play**, **Speed** (1×, 2×, 4×, one key that steps and shows the length at that speed), **Copy**, **Save** and **Close**; no Edit. Play opens the user's own player as any app opens; the keyboard and the small strip stay on top (owner, 2026-11-26). The editor opens only for pictures.
- **New ADR-0060:** videos have no editor; the strip carries Play and Speed (owner, 2026-11-26). It supersedes the video parts of ADR-0055 and ADR-0056 only where they name the editor.
- **New theme tokens:** `shape.snip.handle_px`, `shape.snip.handle_hit_px` (the handle's size and its click area). **New setting:** `[snip] frame_min_px` (default 8), the smallest side a frame may keep.
- **New translation keys (EN, FR, AR):** `snip.crop-ok`, `snip.crop-cancel`, `snip.way-play`, `snip.way-speed`, `snip.handle-<side>` for readers; the editor's video keys go.

## Tasks
1. **The editor comes back darker.** *Files:* `SnipEditor.qml`, `Glow.qml`, the hub's editor view. *Test first:* an offscreen Quick test opens the editor, opens and closes a Save dialog, closes and reopens the editor, and finds one shadow and no shade left over it. *Implement:* find the stale layer (shadow shown twice, or the dialog's dim kept) and clear it on reopen and after the dialog. *Verify:* `tst_snip_editor` offscreen at scale 1 and 1.5.
2. **A maximised editor is remembered.** *Files:* `kx-mod-snip` settings and editor place, `SnipEditor.qml`, defaults. *Test first:* a module test saves a maximised editor and reopens it maximised at the same screen; the normal size and place are kept under it. *Implement:* the editor's saved place gains its maximised state. *Verify:* `cargo test -p kx-mod-snip`; the Quick test.
3. **The delays.** *Files:* `SnipBar.qml`, `Glow.qml`, `SnipApp.qml`, the editor's open path. *Test first:* a Quick test proves the bar's shadow never shows before the bar's window; a timing test on the fakes proves the editor's window shows once its picture is ready with no extra round trip. *Implement:* measure Snip key to bar and Edit to editor with debug timing lines (numbers only), show the shadow with its window, and make the editor open from a window made once. *Verify:* the tests; the measured times go into the report.
4. **The Region frame.** *Files:* `kx-mod-snip` flow (picking), `ports/snip` (a handle click), `SnipOverlay.qml`, `SnipShade.qml`, a new `SnipFrame.qml`. *Test first:* flow tests pick up and set each of the eight handles, keep the frame on its screen and at least `frame_min_px`, start a new region on a click away, and hand the adjusted part to Copy, Edit, Save and Record; Esc still ends the snip. *Implement:* the frame and its handles on the frozen screen, reused by Crop. *Verify:* `cargo test -p kx-mod-snip`; `tst_snip_*` offscreen.
5. **Crop with a confirm key.** *Files:* `kx-mod-snip` edit (`gesture.rs`, `history.rs`), `SnipCrop.qml`, `SnipTools.qml`. *Test first:* a second click shows a pending frame; handles adjust it; ✓ cuts in one undoable step and Undo restores the whole picture; ✗, Esc and another tool drop it. *Implement:* the pending crop in the editor model and its keys. *Verify:* `cargo test -p kx-mod-snip`; Quick tests.
6. **Videos without an editor.** *Files:* `kx-mod-snip` flow and strip keys, `ports/snip`, `SnipStrip.qml`, `SnipEditor.qml`, `SnipPlay.qml`. *Test first:* after Stop the strip shows Play, Speed, Copy, Save and Close and no Edit; Speed steps 1×, 2×, 4× and back and Copy and Save use it; Play asks the host and the strip and keyboard stay on top. *Implement:* move Play and Speed onto the strip and remove the editor's video page. *Verify:* `cargo test -p kx-mod-snip -p kx-ui`; Quick tests.
7. **G24 and the docs.** *Files:* `tools/kx-gates/src/g24prod/` (the sped copy from the strip, the frame after a region), spec, ADR-0060 and the ADR index, module README, `docs/security/snip.md`, `docs/items/P12.md`, i18n files. *Test first:* the gate's unit tests for the new clicks. *Verify:* `cargo test -p kx-gates`; `cargo xtask tidy`.
8. **Live and install.** *Verify:* `kx-gates g24 trap`, then full `g24 keyxtend`, with the owner's go; stage and dev-install; the owner resumes the check.

## Edge cases
1. A handle set so the frame would turn inside out swaps its sides, never a negative size.
2. A frame never goes past its screen (Region) or the picture (Crop), nor under `frame_min_px`.
3. A second click on the spot it picked a handle up sets it there, unchanged.
4. Esc while a handle is held drops the hold first, then ends the snip as today.
5. Undo after ✓ restores the whole picture; Redo cuts again.
6. A recording that ends by itself shows the same strip as Stop.
7. Speed's length shows at each step; a Copy made at 2× and then Speed 4× copies again at 4×.
8. Play while the player cannot open shows a note and keeps the strip on top.
9. A maximised editor reopened on a screen that is gone opens at its normal size on the main screen.
10. The editor reopened after a Save dialog shows exactly one shadow.

## Security
- No new capability, no new data class. Play hands the file's path to the system's default app, as today, and nothing decodes media in the app (ADR-0004).
- The frame's clicks are pointer positions only; nothing new is logged; timing lines carry numbers only.

## Manual Windows check (resume)
- Region with handles: adjust each side once, then Copy, Edit, Save.
- Editor: Crop with handles, ✓, Undo, Redo; reopen after a Save, with no darker editor; maximise, close, reopen maximised.
- The bar and its shadow show together; the editor opens without a wait.
- Video region with Mic: Stop, Speed 2×, Play opens the laptop's player while the keyboard and strip stay on top, Copy into a chat and Explorer.
- Video full screen with Sound, switching Mic and Sound mid-recording; the first second plays right.
- Bubble, Hidden and Shown; close without keeping asks; Don't save leaves nothing in `unsaved/`.

## Done when
- The Definition of Done in `docs/roadmap.md` holds, with every edge case above tested.
- `g24 trap` and full G24 pass, and the owner's resumed check passes.
