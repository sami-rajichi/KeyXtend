# Test report: P3 keyboard core

- **Dates:** 2026-10-10 and 2026-10-12 (final build), on the development PC.
- **Windows:** Windows 11 Home Single Language 26H2, build 26300.9457.
- **Display:** 1920×1080, 125 % (120 DPI), 144 Hz; Ryzen 7 7735HS with Radeon graphics (Qt draws on it), RTX 4060, 24 GB.
- **Builds:** `keyxtend` and `kx-gates`, release, signed and installed under `Program Files\KeyXtend-dev` (uiAccess: secure folder, trusted signature and manifest checked by `cargo xtask check-uiaccess`); `kx-target-window` debug.
- **Scope:** the P3 gates (G1–G4, G8–G11, G13, G25), the plans' manual checks (key repeat, act on press) and the item's manual-check notes.

## Gates on the final build

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | `g2 keyxtend` (1000 clicks, each held 0.4 s with a 2 px slip) | Target keeps focus and caret every time | Pass | 1000/1000 matched; char p50 4.4 ms, p95 16.1, p99 18.3 |
| 2 | `g4 keyxtend` (admin Terminal) | Types with uiAccess alone | Pass | 200/200; nothing beyond uiAccess needed |
| 3 | `g3 keyxtend` | Above Start, Search and Task Manager | Pass | osk.exe running beside it |
| 4 | `g9 keyxtend` | Flags survive hide/show, topmost, click-through, resize | Pass | Failed before the guard's re-check after an outside drop; fixed in P3 |
| 5 | `g10 keyxtend` | Every key named, invokable and found at its centre | Pass | by point 76/76 |
| 6 | `g25 notepad` | Language key, Ctrl+C/V/Z, Win+V, Alt+Tab | Pass | |
| 7 | `g1 target`, `g1 chrome` | 1000 EN/FR/AR characters arrive intact | Pass | 1000/1000 each |
| 8 | `g8 keyxtend`, default settings | Spec §10 budgets | Partial; owner accepts | See notes 1 and 7 |
| 9 | `g11 keyxtend` at 100, 150 and 175 % | Size and sharpness kept | Pass | Drawn at 96, 144 and 168 DPI; 544×164 logical at each, gap 0 px |
| 10 | `g13 keyxtend` | Follows dark mode, accent and high contrast live | Partial; owner accepts | See note 5 |

## Hands-on checks (owner)

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | Quick clicks with natural slips on letters, Space, Backspace (Notepad) | Each key types once | Pass | Act on press (ADR-0024) |
| 2 | Press a key, slip onto its neighbour, let go | Only the pressed key types | Pass | |
| 3 | Hold Backspace and slip onto another key | Deletes until let-go | Pass | |
| 4 | Top-bar button with a slip; double click on the move grip | Acts once; the move runs until the next click | Pass | |
| 5 | Language list row with a slip | Picks once | Pass | |
| 6 | Tray menu item with a slip | Acts once | Pass | |
| 7 | French AltGr+E and an Arabic AltGr key with slips | Each types its sign once | Pass | |
| 8 | Letters into scrcpy (phone mirror) | Letters and numbers type | Pass | After `key_apps` lists `scrcpy.exe` (default since 2026-10-12) |
| 9 | Looks in Native, ET66 and Dolch; French Échap and Alt Gr; tips; tray menu (2026-10-10) | Readable, as approved | Pass | Dolch dark second legends accepted as is |
| 10 | Arabic marks and corners in all themes (2026-10-10) | Marks on ◌, corners clear | Pass | After the marks font and empty Shift/AltGr keys |
| 11 | Who can press a key: a normal program's SendInput and UIA Invoke (2026-10-10) | Nothing types | Pass | SetWindowPos from it: access denied |
| 12 | Hold "a" (Notepad) | One "a", a steady stream after about 1 s, stops on let-go | Pass | |
| 13 | Hold Backspace, then Delete | Each deletes steadily and stops on let-go | Pass | |
| 14 | Shift once, then hold "a" | "AAAA", then Shift is off | Pass | |
| 15 | Ctrl once, then hold Z | One undo only | Pass | |
| 16 | Arabic layout: hold a letter | The right letter repeats and stops | Pass | |
| 17 | Largest size (`size_max` = `size` = 4) | Fits the screen; every top-bar button reachable | Pass | S shrank it |
| 18 | Hold Backspace while the screen locks | The repeat stops; a click then deletes one character | Pass | See note 8 |
| 19 | High contrast (Desert): a letter, Enter, a lit Shift | Each press reads as a press | Pass | |
| 20 | Narrator on: rest the pointer on a letter, Backspace and the language key | Each key's name is read | Pass | G10's listening part; NVDA moves to P14 |
| 21 | Narrator on: rest the pointer on two top-bar buttons | Each is named clearly | Pass | |
| 22 | Caps Lock pressed outside the keyboard, on then off | The Caps light and the letters follow, with no click | Pass | Failed on the first build; fixed in 5c4bd81, see note 9 |

## Notes
1. G8 on default settings, build 1e73f75, nine runs on 2026-10-12: start 350–788 ms (median 527, budget 500), slower under memory pressure; working set 98.5–102.7 MB (median 99.0, budget 100) and commit 134.6–138.8 MB (budget 140); char p99 12.5–20.3 ms; churn growth 0.4–3.0 / −0.3–2.8 MB, one run at 3.04 over the 3.0 limit. Dropped frames were 0–1 before a reboot (32 in one run under memory pressure); after it PresentMon marked 43–49 frames unseen while G8's screen copies saw all 20 presses lit within 15 ms. The owner accepted the results, and P14 tracks start-up and the frame count. The owner's Dolch at size 0.8 idles at 121.8 / 171.6 MB, which P13 records.
2. The key light: the frame log showed the key action (6.2 ms) and a refresh wait (to 10.6 ms) before the press frame. Drawing the press frame first gives a median of 14.2–14.7 ms, the same as Windows' own keyboard (14.2–14.5 ms) by the same method, against 28.1 ms before; the target now matches Windows' keyboard (ADR-0025). Presenting without vsync dropped 142 frames, so frames keep the refresh wait (ADR-0026).
3. The mouse guard held every guarded run; the hand's events were dropped and counted, and a still hand passed G2 30 and 200.
4. When the settings file is removed, the app restores it from `settings.previous.toml`, as `kx-settings` treats a lost file like a damaged one (ADR-0015); a default run needs both files set aside.
5. G13 on Native with `follow_accent` on: the keyboard followed all 5 changes (dark on and off, three accents). Three were within 0.7–0.9 s; two accents took 1–2 s, so the gate's 1 s rule failed though the owner saw each change. The time counts from when Windows stores the setting, before it tells apps. No contrast theme was applied in the run; the owner judged the live following good. A run stops when the keyboard moves, as the tool keeps the keys' first screen places.
6. The saved pictures differ from the 2026-10-10 baseline in 49 of them, all explained: icon edges by 1 step of 255 (icons tinted in one place), ET66 and Dolch light's second legends and space bar name 8 steps darker to keep AA under the press shade, and the approved press shade itself.
7. G8's tip count is 40 of 60 with the shipped order: whichever key is hovered third, about 1.6 s into a round, misses its tip after the round's bubble and theme steps (20 of 20), while Shift first with longer rests gave 60 of 60. The owner's own tip check passed; the cause is open in P13.
8. Admin prompts do not show on this PC (UAC raises rights without asking), so the hold check took the mouse away with the lock screen instead.
9. Caps Lock pressed outside KeyXtend left its light unchanged until the next key click: reading the state worked, but nothing asked for a redraw. The relabel tick now also redraws on a Caps Lock change (build 5c4bd81, installed and checked on and off).

## Clean-up
- Gate logs, screenshots, PresentMon captures and timing scripts are deleted when P3 is recorded; only these numbers are kept.
