# Test report: P5 scroll pad

- **Dates:** 2026-10-17, on the development PC.
- **Windows:** Windows 11 Home Single Language 26H2, build 26300.9711.
- **Display:** 1920×1080, 125 % (120 DPI), 144 Hz; Ryzen 7 7735HS with Radeon graphics (Qt draws on it), RTX 4060, 24 GB.
- **Apps:** Notepad 11.2607.14.0, Chrome 155.0.8059.80, Explorer 10.0.26100.8875, the Settings app 10.0.26100.8875, PowerShell ISE 10.0.26100.8875 (Windows PowerShell 5.1), target-window (debug); PresentMon 2.6.0; Qt 6.11.2.
- **Builds:**
  - `keyxtend` (7c97c80; b3dfadb for check 25; 574a7a6 for G10's last run) and `kx-gates` (7c97c80; 92197a0 for G9's last run; e181156 and its no-scroll copy for the churn A/B; 574a7a6 for G10's last run), release, signed and installed under `Program Files\KeyXtend-dev`, with uiAccess (`check-uiaccess`: secure folder, trusted signature, uiAccess in the manifest);
  - the product's test build `--features gate-input` (7c97c80; 8656554 for G6's hang and unaware records; never signed or installed), which G6 drives;
  - `kx-target-window` and the plain `kx-gates`, debug.
- **Load:** the owner kept WSL and Docker running on purpose, to see the keyboard on a busy PC: free RAM 6.9–12.8 GB of 23.2, `vmmemWSL` 0.5–1.2 GB (WSL off from 15:39), CPU 6–27 %.
- **Scope:** the P5 gates (G6 on the test build; G8 with its scroll part, G9 with the outline, G10 with the pad, on the installed build), and the plan's hands-on checks.

## Gates

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | `g6 notepad`, `chrome`, `explorer` | Aim, slow, fast, ratio, third tap, up, Stop, press-stop, sideways, end | Pass | Ratio 3.76 / 3.37 / 3.34; see notes 1 and 2 |
| 2 | `g6 hang`, `g6 unaware` | A hung target gets no pile of wheels; an unaware window scrolls | Pass | Records at 8656554 (run 1): wheels 119 in the hang, 19 after against 18 before |
| 3 | `g8 keyxtend`, `weak`, `arabic`, default settings | Spec §10 budgets, with a scroll and its outline | Partial; owner accepts | Start, idle CPU, frames, key light, typing and size pass; memory over; see notes 3 and 4 |
| 4 | `g9 keyxtend` | Window flags survive; the outline's edges are flagged and lie under the keyboard | Pass | Outline 0 / 4 / 4 / 0 edges idle, hovered, scrolling, stopped; see note 5 |
| 5 | `g10 keyxtend` | Every key named, pressable and found at its centre; the pad's arrows answer | Pass | 76/76 by point; arrows found and pressed, slow, fast and Stop read back. Passed at 7c97c80, and again at 574a7a6 after the Win changes, where the Win step latched and let go with no lone Win |

## Hands-on checks (owner)

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | Notepad: point at the text, move onto the pad | A thin frame around Notepad's text | Pass | |
| 2 | Tap ▼ | A slow glide; one dot under Stop | Pass | |
| 3 | ▼ again, then a third time | Faster with two dots, then it stops | Pass | |
| 4 | ▲, then Stop | Glides up; Stop stops it | Pass | |
| 5 | ▼, pointer off onto Notepad, one click in the text | Badge ↓ slow; the click stops the glide | Pass | |
| 6 | ▼▼ and wait | Stops by itself at line 300 | Pass | |
| 7 | Chrome: point, ▼ three times | Frame round the page; slow, fast, stop | Pass | |
| 8 | Chrome's wide page: ▶, ◀, Stop | Right, left, stop | Pass | |
| 9 | The two speeds | 3 and 10 lines a second feel right | Pass | The owner will tune them in Settings later |
| 10 | Explorer, System32 | The file list glides, speeds up and stops | Pass | See note 6 |
| 11 | ▼, then minimise the keyboard | The glide stops; the bubble brings it back | Pass | |
| 12 | The empty desktop, ▼ | Lit, nothing moves until Stop or the limit | Pass; owner accepts | Windows gives no way to tell the desktop has nothing to scroll |
| 13 | `limit_s = 10`, ▼ once | Stops by itself after about 10 s | Pass | Test setting, removed after |
| 14 | `lang_key = "list"`: open the list, ▼ | One page per tap, a frame round the list | Pass | Test setting, removed after |
| 15 | Arabic layout: the badge | ↓ بطيء, then سريع, joined and right to left | Pass | |
| 16 | Type into Notepad | Letters appear; Notepad keeps its caret | Pass | |
| 17 | Start menu open | The keyboard stays above it | Pass | See note 7 |
| 18 | Right-click on, a short click | The caret only moves | Pass | |
| 19 | Narrator on the arrows and Stop | Each is read by name | Pass | |
| 20 | Settings app, Installed apps: ▼▼ | Glides and stops at the bottom | Pass | |
| 21 | A tooltip as the target, then ▼ | Stops at once | Pass | The tooltip closes, so the target is gone |
| 22 | Test build (no uiAccess) on an admin Notepad | Nothing reaches it: no key, scroll, Right-click or Grab | Pass | The gate-input test build, not plain `cargo run`; Windows blocks input into an elevated window without uiAccess (ADR-0003) |
| 23 | Installed build on the same admin Notepad | Letters appear and the pad scrolls it | Pass | With the owner's yes for the admin window |
| 24 | PowerShell ISE: ▼▼ to the end | Stops at line 300 | Fail; owner accepts | It glides to line 300, then stays lit until Stop or the limit: ISE gives UI Automation no position, so the end is not seen. The rule of check 12 applies |
| 25 | Win, then Win again; then Win, E, Win | Start opens; Explorer opens with no Start | Pass | Built at the owner's request; see note 7 |

## Notes

1. G6's first run (test build at 8656554) failed Notepad's sideways and end cases, Chrome's third tap and Explorer's end, and Explorer's press-stop found no safe spot. The product now reads a percent past 100 as the end and clears the wheel's trip when the `ScrollPattern` steps end where it tripped; the gate settles the percent before it looks, reads Explorer's bottom on the vertical axis, and judges sideways only where content overflows. Runs 2 and 3 passed in all three apps, and run 4 in Notepad and Chrome.
2. Sideways is judged on Chrome's wide page only: Notepad wraps its words and System32's list has no sideways overflow. Explorer's press-stop stays unjudged, as its status bar lies under the keyboard and this view has no column header. G6's Notepad and Chrome press-stop cases and check 5 cover it, as the press stop is our hook's, not the app's.
3. G8 at default settings (product 7c97c80): start 355 / 369 / 367 ms, idle CPU 0.31 / 0 / 0.31 %, no dropped frame, key light p50 15 ms, char p99 16 ms, size 49 MB. Private working set reads 103.0 / 101.5 / 102.4 MB cold (budget 100; P4 read 100.3–100.5), commit 137.2 / 137.8 / 140.5 MB (budget 140), and 110.7 / 107.5 / 133.5 MB after a scroll. The owner accepted the memory for P5 and P13's memory work takes it (2026-10-17).
4. The churn grew 5.8 MB (Arabic 7.6, weak 0.4; limit 3) with a scroll in each round. Run in turn on the same load, it grew 9.5 and 5.1 MB with the scroll and 0.2 and −0.3 MB without, a gap of 0.32–0.47 MB a round by commit slope. With WSL off, two more pairs still left about 0.22–0.25 MB a round between them, one with reduced motion and one with the mouse module off, so neither closes the gap. Repeated scrolls with no theme change level off: the test build grew 6–8 MB at its first scroll, then stayed flat over 7–9 taps. Threads and handles climb alike with and without the scroll, so no leak shows there; P13 records the numbers and a candidate cause.
5. G9's first outline look failed because the keyboard's shadow window, which owns the keyboard, lies between it and the edges; Windows keeps an owner right under its window. The gate now accepts only our own windows between and fails another app's window or an edge over the keyboard.
6. With a small window in front of a big one, the pointer's path to the keyboard crosses the big one, which the pad then picks; a full-size window is picked easily. The owner kept the wheel-like rule (the frame shows the target first), and "aim where the pointer rested" is a P5 idea.
7. The owner asked that a second tap on our Windows key, with nothing pressed in between, open Start as a real key does. It was built in this session, and a key or click while Win is latched still makes a shortcut (check 25). In stay mode a key or click uses Win, so the next click only lets go; in release mode they let it go already.
8. The first G8, G9 and G10 runs were void: with `settings.toml` set aside the product took `settings.previous.toml`, the owner's place at the top of the screen, which left no room above the pad. The gates then ran with both files aside, and the owner's settings were put back after each run. In run 2 the mouse guard stopped G8 at once on a button press, so it gave no data.
9. The Voice Access check was skipped at the owner's choice; Narrator read each arrow and Stop by name (check 19).
10. The drag-off entry check (press a key, drag off the keyboard and let go outside, then enter from another window; the pad must scroll that window) was not run, as the check needs a drag by hand and a tool without uiAccess cannot send input to our windows. The entry watch's tests cover a capture lost off the keyboard.
11. Gate logs, PresentMon captures and the test files are deleted when P5 is recorded; only these numbers are kept.
