# Test report: P6 shortcuts layer and Power key

- **Date:** 2026-10-23, on the development PC.
- **Windows:** Windows 11 Home Single Language 26H2, build 26300.9711.
- **Display:** 1920×1080, 125 % (120 DPI).
- **Apps:** Notepad 11.2607.14.0, Chrome 155.0.8059.86, Explorer and Task Manager of that build; Qt 6.11.2.
- **Builds:**
  - `keyxtend` (a0448f6), release, signed and installed under `Program Files\KeyXtend-dev` with uiAccess (`check-uiaccess`: secure folder, trusted signature, uiAccess in the manifest).
  - `kx-gates` (a0448f6 for G10 and the first two G25 runs; b77bd5e for G25's last run), release, signed and installed beside it with uiAccess.
- **Load:** the owner's usual load (WSL, Docker), left running.
- **Scope:** gates G25 and G10 on the installed build, and the plan's hands-on checks.

## Gates

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | `g25 notepad`, runs 1 and 2 | Layouts, shortcuts, Switch app, clipboard panel and Power all pass | Fail, then fixed | Switch app stayed "not closed"; Copy all missed once in run 1; see notes 1 and 2 |
| 2 | `g25 notepad`, run 3 | As above | Pass | Every part passed; the power dialog was `#32770` of `explorer.exe`, cancelled with Esc while in front |
| 3 | `g10 keyxtend` | Every key named, pressable and found at its centre; the pad's arrows answer | Pass | 76/76 by point; slow, fast and Stop each way; see note 3 |

## Hands-on checks (owner)

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | Click Shortcuts | Icons fill the side block, Shortcuts is lit, the pad hides | Pass | |
| 2 | Next three times | Dots 1 → 2 → 3 → 1; page 3 shows F1–F12 as text | Pass | |
| 3 | Tips on Copy all and Next | "Copy all (Ctrl+A, Ctrl+C)"; "Next page" and "Page 1 of 3" | Pass | |
| 4 | The Next key's look | Dots clear of the chevron; the layer reads well | Pass | |
| 5 | Notepad: Copy all, then Paste at the end | The text twice; Notepad keeps its caret | Pass | A test file, so the owner's notes stayed untouched |
| 6 | Undo, then Redo | The copy goes, then comes back | Pass | |
| 7 | Delete all | The file is empty | Pass | |
| 8 | Shift, then Copy | Shift lets go after the shortcut | Pass | |
| 9 | Chrome: New tab, Close tab | A tab opens, then closes | Pass | |
| 10 | Shift, then New tab | The closed tab comes back (Ctrl+Shift+T) | Pass | |
| 11 | Zoom in ×2, Zoom out ×2, Refresh | The page grows, shrinks back, reloads | Pass | |
| 12 | Show desktop, twice | The desktop shows, then the windows return | Pass | |
| 13 | Switch app | The switcher stays open until an app is picked | Pass | |
| 14 | File Explorer, Task Manager | Both open | Pass | |
| 15 | Emoji in Notepad | The emoji panel opens | Pass | |
| 16 | Power, then Cancel | The Shut Down Windows dialog opens; Cancel closes it | Pass | Never confirmed |
| 17 | French tips | «Copier tout (Ctrl+A, Ctrl+C)»; Power reads «Marche/Arrêt» | Pass | |
| 18 | Arabic tips | The name on the right, «(Ctrl+A، Ctrl+C)» and «(Win+.)» in order on its left | Pass | |
| 19 | Start menu; Right-click on, a short click | The keyboard stays above Start; the click only moves the caret | Pass | |
| 20 | Lock PC | The PC locks; one click on the lock screen comes back | Pass | The owner chose to try it |
| 21 | Narrator on Copy all and Next | "Copy all" with "Ctrl+A, Ctrl+C"; "Next page" with "Page 1 of 3" | Pass | Narrator turned on and off with the keyboard's Ctrl, Win, Enter |

## Notes

1. After Esc closes the switcher that Ctrl+Alt+Tab keeps open, Windows leaves its hidden window (`XamlExplorerHostIslandWindow`) in front, so Notepad never comes back by itself. G25 now brings Notepad back with its front step, which first checks no dialog-class window shows (b77bd5e).
2. Copy all missed once in run 1 and passed in runs 2 and 3, so the miss is timing in the gate, not the shortcut; the owner's check 5 passed.
3. G10's first run, with the owner's place at the top of the screen, broke off with no room above the pad after pressing the side keys, so it left Right-click mode off. As in P5 (note 8), G10 then ran with a test place lower on the screen, and the owner's settings were put back exactly (hash checked), Right-click on.
4. G25 starts Notepad, which also restores the owner's own unsaved tab; G25 closes only its own tab.
5. Whether Windows' `ShutdownWindows` returns at once or only when the dialog closes is still unknown (P6 item note).
6. Gate logs and the test files are deleted when P6 is recorded; only these numbers are kept.
