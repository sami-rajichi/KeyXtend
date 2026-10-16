# Test report: P4 mouse assist

- **Dates:** 2026-10-14 to 2026-10-16, on the development PC.
- **Windows:** Windows 11 Home Single Language 26H2, build 26300.9711.
- **Display:** 1920×1080, 125 % (120 DPI), 144 Hz; Ryzen 7 7735HS with Radeon graphics (Qt draws on it), RTX 4060, 24 GB.
- **Apps:** Notepad 11.2607.14.0, Chrome 155.0.8059.41, Explorer 10.0.26100.8875, target-window (debug); PresentMon 2.6.0; Qt 6.11.2.
- **Builds:**
  - `keyxtend` and `kx-gates`, release, signed and installed under `Program Files\KeyXtend-dev`, with uiAccess;
  - the product's test build `--features gate-input` (never signed or installed), which G5, G12, G17, G18 and G8's hook parts drive;
  - `kx-target-window` and the plain `kx-gates`, debug.
- **Scope:** the P4 gates (G5, G6, G12, G17, G18, G8 with the hook's p99), G10 for the now live Right-click and Grab keys, the hand try, and the plan's manual checks.

## Gates

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | `g5 plain` (7 cases × 4 rounds) | Clicks, drags and holds act as the hold says; no lost echo | Pass | 28/28, lost 0, hook p99 0.11 ms; see note 1 |
| 2 | `g17 explorer`, `notepad`, `chrome` | Grab picks up, carries and drops; Esc puts back | Pass | `word` not run: Word shows a dialog at each start |
| 3 | `g18 notepad`, `chrome`, `explorer` | Shift-click and Ctrl-click with the product's own latches | Pass | Needs the owner's On-Screen Keyboard minimised; it covered the test points |
| 4 | `g6 notepad`, `chrome`, `explorer` | Arrow keys reach the app by every path | Pass | |
| 5 | `g8 hook`, `g8 hook-weak` | Hook p99 ≤ 1 ms, no lost echo | Pass | p99 0.12 / 0.12 ms, lost 0 / 0 |
| 6 | `g12 keyxtend` | The ring runs at 60 Hz without dropping clicks under it | Pass | At f1c6d9c and aecf605: 200/200 rings at 62 fps, frame p99 21 ms, lost 0; 200 clicks reached, 0 blocked; see notes 2 and 4 |
| 7 | `g8 keyxtend` and `g8 weak`, default settings | Spec §10 budgets | Partial; owner accepts | Every part passes but memory: see notes 3 and 7 |
| 8 | `g8 arabic` | The same budgets with the Arabic layout in front | Partial; P13 must-fix | Arabic held in front; start and memory over budget; see note 8 |
| 9 | `g10 keyxtend`, default settings | Every key named, pressable by UI Automation and found at its centre | Pass | 76/76 by point; Right-click and Grab pressed twice each, focus kept |

## Hands-on checks (owner)

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | Type into Notepad, then open Start | Notepad keeps its caret; the keyboard stays above Start | Pass | `check-uiaccess`: secure folder, trusted signature, uiAccess |
| 2 | Right-click on, then a short click in text | The light and badge show; the click only moves the caret | Pass | |
| 3 | Hold still 1.5 s on a word in Notepad | The ring shrinks, bursts with a click sound, and the menu opens | Pass | |
| 4 | Hold still on a file in Explorer | The file's menu opens | Pass | |
| 5 | A small tremble during a hold | Still a hold | Pass | |
| 6 | Restart with a 0.5 s hold | Right-click stays on; the ring runs in about 0.5 s | Pass | |
| 7 | Restart with a 3 s hold; also let go at 1 s | The ring runs in about 3 s; 1 s gives no menu | Pass | The ring's time follows the hold setting |
| 8 | Grab a file onto a folder | Pick-up and drop sounds; the file moves; Right-click comes back | Pass | |
| 9 | Grab a line of text | The line is selected; Right-click comes back | Pass | |
| 10 | Start a carry, then the Grab key again | The carry stops; Right-click comes back | Pass | |
| 11 | Shift latch, then a click | The text up to the click is selected; Shift lets go | Pass | |
| 12 | Ctrl latch, Grab a file onto a folder | A copy lands in the folder | Pass | |
| 13 | Keys, the top bar and a hold | Keys and the hold sound; the top bar is silent | Pass | See note 6 |
| 14 | Hold Backspace on our keyboard with Right-click on | Backspace repeats; no ring or menu | Pass | |
| 15 | Hold over elevated Task Manager | Its menu opens | Pass | With the owner's yes; Task Manager elevated, KeyXtend not |
| 16 | Lock, unlock, then a hold | The hold still works | Pass | |
| 17 | Clicks, a hold and Grab with all 16 threads busy for 90 s | All behave; the button never stays down | Pass | Hook p99 0.14 ms, max 4.1 ms, lost 0 |
| 18 | Idle with Right-click on, mouse still | Spec §10 budgets | Fail; owner accepts | Commit 149.4 MB (budget 140) at f1c6d9c, before ADR-0028; see notes 3 and 7 |
| 19 | Hand try, Right-click and Grab (900 s) | Numbers only | Done | Events 3,666, hook p99 0.11 ms, lost 0, reinstalls 0 |

## Notes

1. The first gate runs failed G5 with 32 lost echoes and a second release in the held drag, for two reasons. Windows hands a mouse hook only the low 32 bits of `dwExtraInfo`, which cut off our act ids, and it runs our hooks inside our own `SendInput` while the host is busy, so no echo was noted. The ids now sit in the low 32 bits, and the hook notes echoes heard inside its send.
2. G12 found the ring late or missing after a pause (161 of 200 drawn, 48 fps). The overlay window changed size on every read, and Qt's next frame then waited up to 1 s for its swap chain, freezing the GUI thread. A fixed-size window drew 200 of 200 at 62 fps.
3. G8 on a quiet PC (product at aecf605, kx-gates at 7517574): start 356 ms (weak 384), idle CPU and dropped frames pass, key light p50 15 ms, char p99 14–15 ms, no growth over the churn. Private working set reads 100.3–100.5 MB cold and 104–105 MB after use (budget 100); commit reads 135.3–135.6 MB cold and 141.4–142.1 MB after use (budget 140). P3 read 98.5–102.7 / 134.6–138.8 MB cold, and the owner accepted P4's memory as P3's (2026-10-16).
4. G12's test build put its hooks back once per run, 2–3 s after start, and never in G5 or G8's hook parts. Windows saw input our hook did not, most likely mouse noise that the gate's mouse guard swallowed ahead of our hook. Nothing was lost.
5. Gate logs, PresentMon captures and the diagnostic builds' logs are deleted when P4 is recorded; only these numbers are kept.
6. The owner found volume 50, 35 and 25 too loud on their laptop, and 15 about right, so `[ui] sound_volume` defaults to 15.
7. Idle with Right-click on, at f1c6d9c before ADR-0028: CPU 0.16 % of one core (0.36 % after a lock and unlock, budget 0.5 %), private working set 80.4 MB, private commit 149.4 MB. The overlay itself costs 1 to 2 MB: the rest is what the keyboard window keeps after use. Qt's renderer kept spare buffers (now off, ADR-0028: five resizes cost 3.6 MB instead of 13.4), and the first theme, tip and bubble steps keep about 20 MB, which P13 tracks.
8. `g8 arabic` on a quiet PC (product at aecf605, kx-gates at 0781351) held the Arabic layout in front through the run and put the owner's layout back. Start took 624 ms and memory read 104.8 / 143.6 MB cold and 125.9 / 169.5 MB after use; every other part passed, and the owner made the memory a P13 must-fix (2026-10-16), and its start goes to P14. The app in front reported English while the session typed Arabic, which P13 checks.
9. The G8 and G12 runs at 13:12 on 2026-10-15 ran while scrcpy, WSL and Docker were busy. PresentMon then had no display time for about 99 % of all frames, ours and other apps', and start-up took 1,176 ms. With those programs closed, every frame showed and start-up took 356–384 ms.
