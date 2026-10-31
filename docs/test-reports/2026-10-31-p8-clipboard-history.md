# Test report: P8 clipboard history

- **Date:** 2026-10-31, on the development PC.
- **Windows:** Windows 11 Home Single Language 26H2, build 26300.9711.
- **Display:** 1920×1080, 125 % (120 DPI).
- **Apps:** Notepad 11.2607.14.0, Paint 11.2605.81.0, Snipping Tool 11.2608.41.0, Explorer, Narrator, Chrome 156.0.8078.74; Qt 6.11.2.
- **Builds:**
  - `keyxtend` and `keyxtend-worker` (aca46e85), release, staged, signed and installed under `Program Files\KeyXtend-dev` with uiAccess (`check-uiaccess`: secure folder, trusted signature, uiAccess in the manifest).
  - The gate test build (`--features gate-input`, no uiAccess) and its worker at aca46e85, driven by the plain `kx-gates`; again at 526da0a7 for G20's last run.
  - `keyxtend` at 2f8ca072 for the final install, and the test build at 2f8ca072 for the Open folder recheck (note 4).
- **Load:** the owner's usual load (WSL, Docker), left running.
- **Scope:** G20 on the product, G20's worker check, G8's clipboard part, and the plan's hands-on checks.

## Gates

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | `g20 keyxtend`, run 1 | Rows, search, ticks, Insert (2), Win and V | Void | A mouse press stopped it at once, by design |
| 2 | `g20 keyxtend`, run 2 | As above | Fail, then gate fixed | First copy all passed; the history-off copy needs uiAccess (note 1) |
| 3 | `g20 worker` on the dev install | Every worker without uiAccess, at most Medium | Pass | Worker 8192 (Medium), no uiAccess; product 12288 with uiAccess (note 2) |
| 4 | `g8 clip`, run 1 | `[g8]` budgets with 1,000 entries; rounds grow ≤ 3 MB | Fail | Empty 105.1 / 142.3 MB, as P7; open panel +45.5 / +68.2 MB; rounds +3.4 / +5.9 MB (note 3) |
| 5 | `g8 clip`, run 2, 3 warm-up and 10 judged rounds | As above | Fail; owner accepts | Rounds +2.9 MB working (pass) and +7.3 MB commit (note 3) |
| 6 | `g20 keyxtend`, run 3, with the gate fix | As run 2, the history-off copy skipped without uiAccess | Pass | Every checked part passed |

## Hands-on checks (owner)

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | Copy EN, FR and AR text, a link, a browser picture, a Snip, a Paint picture and two files; open the panel with the clip key | Newest first, each kind's icon, real thumbnails, Arabic right to left | Pass | |
| 2 | Search without accents and harakat; tick in two searches; Insert (2) into the owner's own Notepad tab or a browser box | Both found, both pasted in tick order with a blank line | Pass | |
| 3 | Insert the Snip into Paint, and the files into an Explorer folder | The whole picture; the files copied | Pass | The PNG went with the worker's `CF_DIB` |
| 4 | Win, then V, on our keyboard | Our panel, not Windows' | Pass | |
| 5 | Pin, Delete, Delete all | Delete all asks with the count; pins stay | Pass | |
| 6 | Long text and picture previews; the storage line | Pages turn by the preview's buttons; the line shows | Pass | |
| 7 | Open folder | Explorer opens the history's folder | Fail, then Pass | "Cannot find" on the installed build, from how it was started (note 4); Pass on the test build |
| 8 | Light, dark and high contrast; the keyboard in Arabic | The panel follows each look; Arabic right to left | Pass | |
| 9 | Narrator on, a 40 MB copy while the panel shows | The too-big note shows and Narrator reads it by itself | Pass | The text was made by the controller with `clip.exe` |
| 10 | Regressions: keys never take focus; above the Start menu; Right-click mode keeps short clicks normal | As before | Pass | |

## Notes

1. An injected Win+V reaches Windows only from a uiAccess app (as P6 found), and the test build never has uiAccess. G20 and G25 now report that part as skipped with a reason (650084c7, 3aca4a01); it is checked by hand on the dev install once Settings can turn `history_key` off (P13).
2. On an administrator account Windows runs a uiAccess app at High, and the product starts its worker from a restricted token at Medium with no uiAccess, as designed.
3. The empty product is over `[g8]` as in P7, which the owner accepted with P13 owning memory; 1,000 entries add only 3.4 MB. A harness over the real QML (no mouse) showed the commit a close keeps settles near 50 opens, about +69 MB, so the owner accepted the readings, and the two savings found (a give-back after close, lighter title text) are in `docs/items/P13.md`.
4. The installed build was started from a packaged (MSIX) app's shell, so it wrote into that app's private copy of AppData, where Explorer cannot see the history folder; a change to start `explorer.exe` instead (db2dfb80) chased that and was reverted (2f8ca072). On the test build, whose data folder is on D:, Open folder opened `target\gate-input\release\data\clipboard`.
