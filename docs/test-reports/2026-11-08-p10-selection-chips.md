# Test report: P10 selection helper

- **Date:** 2026-11-08 and 2026-11-09, on the development PC.
- **Windows:** Windows 11 Home Single Language 26H2, build 26300.9711.
- **Display:** 1920×1080, 125 % (120 DPI), one screen.
- **Apps:** Notepad 11.2607.14.0, Chrome 157.0.8103.38, Zen 1.24.1b (the default browser), Windows Terminal 1.24.12741.0 and the console (the default terminal is "Let Windows decide"), VS Code 1.145.2, Explorer; Qt 6.11.2.
- **Builds:**
  - `keyxtend`, release, staged, signed and installed under `Program Files\KeyXtend-dev` with uiAccess, at bfc02627, then 8504cb8d, and last at 2d690636 (`check-uiaccess`: secure folder, trusted signature, uiAccess in the manifest).
  - The gate test build (`--features gate-input`, no uiAccess), with its own data folder, driven by the plain `kx-gates`.
- **Load:** the owner's usual load (WSL, Docker), left running.
- **Scope:** G19 on the test build, the uiAccess inheritance check, and the plan's hands-on checks.

## Gates

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | `g19 core` | Notepad and Chrome lines read back through UI Automation, the clipboard untouched | Pass | After the gate paced its keys (note 1) |
| 2 | `g19 keyxtend`, strip cases | Each case shows the right chips, keeps focus and is placed clear of the keyboard | Pass | 21 cases, 16 placed, Notepad, Chrome, our test boxes, Windows Terminal, the console, a running ping; last run at 2d690636 |
| 3 | `g19 keyxtend`, edits | Copy, Cut and Paste land; Chrome gets Ctrl+C, never Ctrl+Shift+C; a latched Shift stays latched | Pass | |
| 4 | `g19 keyxtend`, caret Paste, hides, Translate's question | As designed; Not now opens nothing | Pass | |
| 5 | `g19 keyxtend`, near the bottom | The strip goes above the line | Pass | On a target-window the gate moved; the second screen is not applicable (one screen) |
| 6 | `g19 keyxtend`, a hung app | No strip for the hung app; other apps keep working | Pass | The hung app's own cases passed in every run; the gate's Esc click after them failed in 3 of 7 runs on a covered key, not in the last (note 4) |
| 7 | `g19 terminals` | Windows Terminal, the console and a running ping show the strip; Copy and Paste with Num Lock on and off | Pass at 6363e719, and inside `g19 keyxtend` at 2d690636 | Failed before the fix in note 2 |
| 8 | `g19 translate-read` | The owner's Translate and Yes open the page with the whole line | Pass by eye | Zen hides its address bar until clicked, so the gate cannot read it (note 3) |
| 9 | uiAccess inheritance beside the installed keyboard | The browser gets no uiAccess and no raised level | Pass | Note 3 |

## Hands-on checks (owner, installed keyboard)

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | A word selected in Notepad | The keycap strip, as tall as a key, with Copy, Cut, Paste, Translate | Pass | |
| 2 | Copy, Cut and Paste | Each acts and shows its done word | Pass | |
| 3 | A click after a line, with something copied | Paste alone, gone after 5 s or on a key | Pass | |
| 4 | Shift latched and Right five times | The strip follows calmly | Pass | No jitter |
| 5 | A selection made with Grab | The strip shows | Pass | |
| 6 | The keyboard minimised to its bubble | The strip still shows | Pass | |
| 7 | The keyboard hidden to the tray | No strip | Pass | |
| 8 | Translate in French and Arabic | Google Translate opens with the whole text, correct | Pass | |
| 9 | Static text on web pages and in an Electron app | Copy and Translate | Pass at 2d690636 | Failed before the fix in note 5; rechecked in Zen and the Claude app after the last fixes, with a page's text box still showing all four |
| 10 | Three themes, light and dark | The strip follows each look | Pass | |
| 11 | Sizes S and L | The strip stays as tall as a key, its gap grows | Pass | |
| 12 | The fade in and out | No clipped edge or shadow | Pass | |
| 13 | Windows Terminal: `dir`, a word, Copy, Paste at the prompt | Both work | Pass | |
| 14 | VS Code's terminal | No strip, nothing else happens | Pass | Accepted: it does not tell its selection |
| 15 | The scroll pad on a long page | The strip follows the word or goes | Pass | |
| 16 | The Arabic layout | The strip's Arabic words are correct | Pass | |
| 17 | Narrator on the strip | It says each button's name | Pass | |
| 18 | Regressions: focus, above the Start menu, Right-click mode | As before | Pass | |

## Notes

1. The gate's own tools needed fixes found live, none in the product: paced keys in Notepad, a Cut drag that took the line break, press points clear of our windows, a rich-edit target-window, the console started with no standard handles, Zen added to the browser lists, and Don't save for the gate's own tab.
2. Windows Terminal refuses to compare a selection's start with its own end, so the strip never showed there. When that check is refused, the selection's line boxes now tell whether it holds text (6363e719).
3. The plain gate cannot see a uiAccess keyboard, so the inheritance check was done by hand: Zen closed, the owner's Translate and Yes, then a read-only token read. The new Zen ran at medium level without uiAccess, started by a launcher that had exited; the keyboard ran with uiAccess.
4. In 3 of 7 runs, after the hung case, the test keyboard (no uiAccess) was reported below windows that are not topmost, such as Notepad, IME windows and WinUI popup bridges, so the gate's Esc click hit another window. The gate now prints the z-order when it happens; this window-guard finding is in P11's notes, and the installed keyboard stayed above all in the hands-on check.
5. Many pages and Electron apps focus a plain group or text item while the page's document holds the selection. The look now walks up to the nearest ancestor that holds the text, and stops at a password box, a hiding or opaque window, or our own; after a walk up, the strip offers Copy and Translate only, as the keys go to the focused element (3847d9c4, fbfa0961, 0382aa49).
6. All test data was deleted: the gates' data folders, their Notepad tabs and the screenshots, each at once.
