# Phase 1 — Toolkit test round (plan)

## Goal
Choose the UI toolkit by measurement (Qt 6 Quick + cxx-qt vs Slint 1.18, with a quick Tauri check) and prove the risky part of every planned feature.
Source: `docs/roadmap.md` Phase 1, ADR-0011, item P1 in `docs/FEATURES.md`.

## Scope
**In (decided with the owner, 2026-09-27):**
- Stages 0–5 below, each ending with a Pass/Fail checkpoint pop-up; a toolkit that fails a must-pass gate stops there.
- Gates G1–G16, with three changes: G11 uses one screen, G13 is Native Adaptive plus an ET66/Dolch sampler row, G14 lasts 4 hours.
- New feature probes G17–G26 (table below); G6 covers four scroll directions.
- Weak mode imitates a cheap 4 GB laptop (Celeron N4020 class).
- Kept for good: `cargo xtask dev-cert`, `dev-install`, `check-uiaccess`, this plan, ADR-0013, doc updates, the test report.
- F13–F16 added to `docs/FEATURES.md`: hover-to-type, switch scanning, number pad + navigation panel, CJK IME mode.

**Out:** real product crates (P2+), full ET66/Dolch themes (P13), the two-screen DPI move (P3), overnight soak.

## New and changed gates (full pass marks go into `docs/roadmap.md`)
| Gate | Pass mark (short) | Runs |
|---|---|---|
| G6 Scroll | Up, down, left, right in Notepad, Chrome, Explorer, Word | core |
| G11 DPI | 100/150/200 % on one screen keep size and sharpness | per toolkit |
| G13 Themes | Native light/dark live (accent, high contrast, frosted glass tries) + sampler row | per toolkit |
| G14 Soak | 4 h of automatic typing, both keyboards together: flat memory, no crash | per toolkit |
| G17 Grab | Click-move-click drags a file and selects text; Esc cancels; button never stuck | core |
| G18 Modifier+click | Shift+click extends a selection; Ctrl+click picks several files | core |
| G19 Selection | Selection seen in Notepad, Word, Chrome without copying; no-focus pill's Copy works | core + pill |
| G20 Clipboard | Every copy seen; "exclude from history" copies skipped; one click pastes an old entry | core |
| G21 Password | Password boxes flagged in Chrome and a Win32 field; normal fields not flagged | core |
| G22 Voice | Mic records in a worker; caption bar takes no focus; Whisper base speed on normal + weak | core + bar |
| G23 Quick-fill | Hello opens from the no-focus keyboard; Yes fills test User/Password; Cancel types nothing | core + owner window |
| G24 Snip | Overlay above all windows; click-move-click region; image matches at 125 % | per toolkit |
| G25 Language + shortcuts | Target app switches language with us; Ctrl+C/V/Z, Win+V, Alt+Tab work | core |
| G26 Other languages | GitHub Windows: de, ru, he, fa labels and text intact; Japanese IME gives 日本 | core, CI |

- **Must-pass per toolkit:** G2, G3, G4, G7–G16, and the window parts of G19, G22, G23, G24.
- A failed core gate is a design finding for ADR-0013; it never eliminates a toolkit.
- Known OS walls (GNOME Wayland: no on-top, no-focus overlay) are recorded, not counted as a toolkit fail.

## Design notes
- **Branch:** all work on `spike/toolkits`. Kept files get their own commits, never mixed with spike files.
- **Spike (throwaway, light rules):** folder `spike/` with its own Cargo workspace, outside tidy's scan dirs and CI.
  - `spike/spike.toml`: every number and path (counts, timings, weak-mode limits, soak length).
  - `core`: input injection, window guard, mouse hook, scroll, UIA, clipboard, Hello, capture, layout labels, measuring, weak-mode job.
  - `harness`: one subcommand per gate; prints the numbers. `target-window`: records every character with a timestamp.
  - `worker`: mic recording and whisper.cpp. Faces: `slint-kb`, `qt-kb` (cxx-qt + QML), `tauri-kb` (G2, G3, G8 memory only).
- **Stage 1 keyboard:** the 5-row block with labels read from the installed layouts, no theme.
- **Stage 2 keyboard:** plus Shift/AltGr/Caps, language key, Right-click toggle, S/M/L, one mirrored Arabic panel with a text field, pointer ring, Native light/dark, sampler row.
- **Weak mode:** the face runs in a Windows job: 2 cores, about 30 % speed, 4 GB, software drawing.
- **Measuring:** memory from Windows counters; frames from PresentMon; click-to-character from `target-window`; size as a Deflate zip of the app folder (an upper bound for Inno Setup's LZMA2).
- **Kept dev tools** (full rules), in `xtask/src/devtools/`: `config.rs`, `sdk.rs`, `cert.rs`, `install.rs`, `check.rs`.
  - Settings in `[workspace.metadata.devtools]`: cert subject `CN=KeyXtend Dev Test`, cert days `90`, output dir `target/dev-cert`, install dir name `KeyXtend-dev`, SDK bin root, SDK arch `x64`, tool names `signtool.exe` and `mt.exe`, digest `SHA256`.
  - `dev-cert`: creates a code-signing certificate with a non-exportable key in the user store, exports the public part, and writes the launcher the owner opens. `--remove` deletes it and writes the "untrust" launcher.
  - `dev-install <folder>`: signs every `.exe` and copies the folder to `Program Files\KeyXtend-dev\<name>` (Windows asks Yes). `--remove` deletes it.
  - `check-uiaccess <exe>`: checks the three Windows conditions: under Program Files, trusted signature, `uiAccess="true"` in the manifest.
  - The running-process token check lives in the spike core now and in `kx-platform-windows` from P3.
- No product ports, settings schema keys, theme tokens or translation keys: nothing in the spike is kept.

## Tasks
**Stage 0 — setup**
1. **Docs.** *Files:* `docs/roadmap.md` (Phase 1), `docs/FEATURES.md` (P1 In progress, F13–F16), this plan. *Verify:* the gate table matches this plan.
2. **Dev-tools config and SDK finder.** *Files:* `Cargo.toml`, `xtask/src/devtools/config.rs`, `sdk.rs`. *Test first:* a bad config is rejected, and the newest SDK folder with the tool wins. *Implement:* load and validate the config; pick the newest SDK version folder. *Verify:* `cargo test -p xtask` green.
3. **dev-cert.** *Files:* `devtools/cert.rs`, `cli.rs`, `main.rs`. *Test first:* the create script sets NonExportable, code signing and the configured days; the launcher imports only the exported file. *Implement:* run the scripts through PowerShell; print the thumbprint. *Verify:* `cargo xtask dev-cert` creates the cert and launcher.
4. **dev-install.** *Files:* `devtools/install.rs`. *Test first:* the target is `Program Files\KeyXtend-dev\<name>`; names with separators or `..` are rejected. *Implement:* sign each exe, then run the elevated copy or removal. *Verify:* unit tests green; a real install happens in task 9.
5. **check-uiaccess.** *Files:* `devtools/check.rs`. *Test first:* a lookalike folder such as `Program Files Evil` fails, and a manifest with `uiAccess="false"` or no flag fails. *Implement:* path check, `signtool verify /pa`, `mt.exe` manifest read; one line per condition. *Verify:* run on the installed exe in task 9.
6. **Review kept files.** Run `kx-review` and `kx-security-check` (new `docs/security/dev-tools.md`); fix findings; commit. *Verify:* the six gate commands in `CLAUDE.md` are green.
7. **Installs** (a yes for each download; C:/D: space before and after; `D:\dev\INSTALLED.md`; `kx-licence-check`): Qt 6 through aqtinstall, PresentMon, NVDA portable, CMake if VS has none. *Verify:* each tool prints its version.
8. **Certificate (owner).** I open File Explorer on the launcher; the owner opens it and clicks Yes. *Verify:* a read-only listing shows the thumbprint in LocalMachine\Root.

**Stage 1 — deal-breakers** (checkpoint pop-up at the end)
9. **Spike skeleton.** `spike/` workspace, `spike.toml`, `target-window`, the three faces showing the stage-1 keyboard, uiAccess manifests. *Verify:* `dev-install` then `check-uiaccess` pass for each face.
10. **G1 typing** (core): 1,000 random EN/FR/AR characters with `لا`, harakat and AltGr into Notepad, Word, Chrome, Terminal and `target-window`; read back and compare.
11. **G2 no focus, G3 top band, G4 admin** per face (Tauri: G2, G3). G2: 1,000 clicks, checking the foreground window and caret each time. G3: screenshots over Start, Search, Task Manager. G4: typing into an admin PowerShell.
12. **G5 hold, G17 Grab, G18 modifier+click** (core): hook p99 time, 6 px vs 8 px moves, Esc, admin target.
13. **G6 scroll** (core): four directions in Notepad, Chrome, Explorer, Word.

**Stage 1b — feature probes** (checkpoint)
14. **G19, G20, G21, G25** (core; the G19 pill per face).
15. **G22 voice:** worker records the owner's EN/FR/AR sentences; Whisper base (downloaded after a yes, checksum checked) timed in normal and weak mode; caption bar per face. Recordings deleted.
16. **G23 Hello** (the owner confirms with their own PIN or face; only test values are typed) and **G24 snip** per face.

**Stage 2 — looks, speed, accessibility** (checkpoint)
17. **Full test keyboard** per face; **G7 Arabic, G12 ring, G13 themes** side by side with `design/keyboard-style-lab.html`.
18. **G8 budgets** (normal + weak; Tauri memory), **G9 flags**, **G10** Narrator and NVDA (UIA names checked by script, spot-listened by the owner), **G11** (the owner changes the scale in Settings).

**Stage 3 — G14 soak.** Both faces take turns typing into Notepad for 4 hours; memory is sampled every minute and the trend is fitted. The PC stays awake without changing settings. (checkpoint)

**Stage 4 — other systems** (checkpoint)
19. **G15 + G26:** `spike/ci` workflow for macOS, Ubuntu and Windows; push only after the owner's yes; the remote branch is deleted after.
20. **G16:** VirtualBox (a yes; its drivers go on C:); distro picked by pop-up then; GNOME and KDE, Wayland and X11.

**Stage 5 — results and cleanup**
21. **ADR-0013** (numbers, winner, redesign findings); a pointer in ADR-0011 Results; ADR-0002 status if superseded; update `ARCHITECTURE.md`, spec §3.5–3.6 and §11, `THIRD_PARTY.md`, and the `kx-slint-theme` and `kx-windows-manual-test` skills.
22. **Test report** `docs/test-reports/2026-MM-DD-p1-toolkits.md` (numbers only).
23. **Cleanup:** delete `spike/`, the VM, the Program Files install and the loser's tools. The owner opens the untrust launcher. Delete screenshots and logs. *Verify:* read-only checks show all gone.
24. **Record and PR:** `kx-module-done`; WORKLOG, FEATURES (P1 Done, P2 Ready), CHANGELOG; branch `feat/P1-results` from `main` with kept commits only; pull request after the owner's yes.

## Edge cases (kept dev tools; each maps to a test)
1. Config: zero cert days, empty names or a missing SDK root are rejected.
2. No SDK folder contains the tool: a clear error naming the tool.
3. Several SDK versions: the newest one containing the tool is chosen.
4. Thumbprint output that is not 40 hex characters is rejected.
5. An install name with `\`, `/` or `..` is rejected.
6. A path under a lookalike folder (`Program Files Evil`) fails the check; the comparison ignores case.
7. A manifest with `uiAccess="false"`, or with no flag, fails the check.
8. A cert that already exists is reused, not duplicated.
9. The owner cancels the Windows prompt: exit code "check failed" with one line of explanation.

## Security
- Kept tools: key theft (non-exportable key, user store, code-signing only, 90 days, removed at the end); launcher tampering (it imports one file by thumbprint); elevated copy (fixed source and target, no user text in the command).
- Spike: the faces have no network; the worker downloads nothing (model fetched by hand, checksum checked); only random or test text is typed; recordings deleted.

## Manual Windows check (owner)
Open the trust launcher, Yes on each install prompt, look at the G3 screenshots, G7 Arabic, G10 listening, G11 scale changes, G13 side by side, G22 speaking, G23 Hello, G24 snip region, soak start.
Results go into the test report through Pass/Fail pop-ups.

## Done when
- Definition of Done in `docs/roadmap.md` for the kept files; for the spike, ADR-0013 holds every gate's number and the winner.
- No leftovers: no spike code, VM, test install, certificate, screenshots or logs remain.
