# Work log

Newest first. One entry per implemented feature or fixed issue: a timestamp plus 3–4 sentences at most.

## 2026-09-28 — P1: the Arabic test panel for G7
The Settings key now opens a right-to-left Arabic panel on the Qt face, a stand-in for Settings: a header with a close button, five rows a page with page buttons, and an Arabic text field, in each theme's light and dark look and with system colours in high contrast. It is the only window that takes focus; it opens above the keyboard, lined up with its right edge, and hands focus back to the app when it closes. Tests pass (core 259, harness 116, Qt 8, 12 offline QML checks), and a live check through the accessibility API, with no typing into apps, confirmed the Arabic, the paging, the focus and the caret reader. Three review rounds found about 32 issues, all fixed; the G7 typing run waits for the owner's yes.

## 2026-09-28 — P1: the keyboard moves as in the mock-up
Each mock-up animation the spike has a part for now plays on the Qt face, timed from one new `motion.toml`: key presses, colour fades, a red mic key whose light pulses while recording, legends rising and language names sliding the way the key stepped, the D-pad's Stop popping in, and the pill and caption bar popping in, with a shimmer while words are on their way. Reduced motion, from the keyboard setting or Windows' animation switch, makes every length 0, and no loop runs at 0 ms. 13 new core tests and a scratch QML test of the real files, with full and reduced timings, pass; a live start with theme switches logged no errors, and the microphone was never used. A review of every changed file found about 20 small issues: all are fixed except three kept on purpose, listed in the hand-off notes.

## 2026-09-28 — P1: the Copy pill, voice caption and snip overlay follow the theme
The three small windows beside the keyboard now take each theme's look, light and dark, as in the mock-up: a badge-coloured Copy pill with a shadow, a caption bar with a red recording dot and the Arabic font for Arabic words, and a snip overlay that dims outside the region, with a size tag and a hint bar. Colours the mock-up writes once for all themes live in one shared `[common]` table, with system colours in high contrast, and their sizes in `shape.toml`. A preview of all six looks and live runs (pill on a real selection, overlay open and cancelled with no file left) passed; the microphone was never used. A review found 4 small issues, all fixed.

## 2026-09-28 — P1 stage 2 part B: the full keyboard on Qt
The Qt face now draws the designed keyboard in all three themes, light and dark, with tooltips, hover and press tints, sticky modifiers, the language carousel, click-move-click moving and resizing, and minimise to a bubble in a bottom screen corner. The owner's look check passed after colour fixes; their decisions (no labels under icons, no Esc undo, a remembered state, the settings strip only when asked) are in the spec and roadmap. Dead keys now reach the app as key presses so Windows adds the accent (the owner typed ê), Caps Lock results come from the layout, and a live bug where a move never ended at a screen edge was fixed. Three review rounds found about 45 issues, all fixed, including latches an accent key ignores and arrows that must keep an accent waiting.

## 2026-09-28 — P1 stage 2 part A: look tokens, geometry, size and system look
All three themes, each light and dark, now live in `spike/themes.toml`, and a review checked every value against the mock-up. spike-core gained toolkit-free parts that load and check those tokens, lay out the keyboard at any size, handle −/+, S/M/L and click-move-click resizing, and read Windows' dark mode, accent colour and high contrast. The fonts (IBM Plex, Rubik, Noto Sans Arabic) and 31 Lucide icons were downloaded with their hashes recorded. Eleven text colours in the mock-up are under 4.5:1 (ten in Dolch, one in ET66 light); a test pins that list until the owner decides at the look check.

## 2026-09-28 — P1: voice probe (G22)
A worker process now owns the mic, runs whisper.cpp's server once at start (base model, 0.8 s load) and can send clips to Groq whisper-large-v3 when cloud is turned on; both faces got a Mic button and a click-through caption bar. On both faces the bar stayed on top, Notepad kept focus, the words arrived 0.8–1.3 s after Mic stop, and the worker does not inherit uiAccess. Notepad lost most of a sentence typed in one batch, so dictated text is typed 10 ms per character on its own thread; speed against clip length was local 0.14, weak mode 0.36 and cloud 0.08, and base is weak for French and Arabic. A five-part review found about 35 issues, all fixed but two accepted, among them a loopback-only server address and length-capped worker messages.

## 2026-09-27 — P1: tools row, Copy pill, quick-fill and snip probes (G19 pill, G23, G24)
Both faces got a tools row (Fill user, Fill password, Snip), a Copy pill beside a selection, Windows Hello quick-fill and a full-screen snip overlay, built on shared spike-core code. On both faces the pill sat within 1 px of its place, left the app's focus alone even right after a click on the keyboard, and copied the line. The snip saved the picked region with 0 differing pixels, and with Hello not set up, Fill typed nothing. Slint needed its unstable winit hook to open windows without focus and a retry to lift a new window above the keyboard (Qt did both itself); a five-part review found about 40 issues, all fixed.

## 2026-09-27 — P1: selection, clipboard, password and language probes (G19, G20, G21, G25)
A line selected in Notepad or a Chrome text box (English, French and Arabic) is read back through UI Automation without touching the clipboard, and a pill position is worked out from its screen box. A clipboard listener heard all 25 test copies, skipped the 5 marked as private and pasted an older one back, keeping the owner's clipboard and removing test copies from the Win+V list. Password boxes are recognised in Chrome and in a Win32 box, the language key cycles English, French and Arabic in the app in front, and Ctrl+C/V/Z, Win+V and Alt+Tab work (Win+V only with uiAccess). Three reviews found about 50 issues, all fixed, including arrow-type keys now sent as extended keys and the listener opening the clipboard with its own window.

## 2026-09-27 — P1: Grab, modifier+click and scroll probes (G17, G18, G6)
The owner tried Right-click and Grab with their own mouse: both work. Automatic runs passed in Notepad, Chrome and Explorer: Grab selects text and moves a file (Esc cancels), latched Shift and Ctrl change the next click, and UI Automation, a posted wheel and a SendInput wheel all scroll a window the pointer is not on. Word was not tested: a fresh Word process shows a subscription-check dialog over the document. Two reviews found about 30 issues, all fixed.

## 2026-09-27 — P1: hold-to-right-click engine and G5
The hold engine turns a 1.5-second still press into a right-click, or into a grab in Grab mode, and passes normal clicks and drags through unchanged. It runs in low-level mouse and keyboard hooks on their own thread. G5 replays clicks, drags and holds into the test window, including one run as administrator: both passed 20/20, with the hook taking at most 0.1 ms. Two reviews found about 30 issues, all fixed.

## 2026-09-27 — P1: no-focus, top-band and admin tests
Slint and Qt both passed G2: 1000 clicks each, the target kept focus every time, and keys typed in about 4 ms (13 ms at worst). Both stayed above Start, Search and Task Manager (G3). The new `harness g4` opens Terminal as administrator and clicks 200 keys into it: both typed into it with uiAccess alone, Qt 200/200 and Slint 199/200 (the owner touched the mouse then). Covered keys are now left out of G2 and G4, and the click code they share lives in `clicks.rs`.

## 2026-09-27 — P1: test keyboards and the typing test
Slint and Qt test keyboards run with uiAccess; Tauri was dropped because WebView2 will not start under uiAccess. The harness typed 1000 random English, French, Arabic and AltGr characters into five apps: the test window, Chrome, Terminal and Word received every one (Word's AutoCorrect curls quotes), and Notepad drops a few only in fast bursts. The harness now checks where the keyboard focus really is, saves a screenshot when a pop-up blocks it, and moves on to the next app. A four-part review found about 65 issues in the spike code, all fixed or explained.

## 2026-09-27 — uiAccess dev tools
`cargo xtask dev-cert` makes a test signing certificate and a launcher the owner opens to trust it. `dev-install` signs a build and copies it into Program Files, and `check-uiaccess` explains why uiAccess is missing. Every admin step checks its inputs before the Windows prompt and can only touch `Program Files\KeyXtend-dev`. Two review rounds found about 60 issues, all fixed; 225 tests pass.

## 2026-09-27 — P0 Foundation done
The Rust workspace, its strict lints and the `cargo xtask` checks (tidy, dco, licences, dist stub) are in place, with 97 tests. CI runs fmt, clippy, tests and tidy on Windows and Ubuntu, plus cargo-deny, cargo-audit and a DCO check; all four jobs passed on the first pull request. The public repo protects `main` with a ruleset, blocks pushed secrets and accepts private vulnerability reports.

## 2026-09-27 — Public docs describe the product
The README, spec §1 and the project docs now describe KeyXtend and its target users, not one person. The GitHub username appears only in the repo address and CODEOWNERS. Personal notes for this PC and owner moved to the git-ignored `CLAUDE.local.md`, and `docs/NEXT-SESSION.md` is no longer tracked.

## 2026-09-26 — Toolkit re-check for macOS and Linux
Three research passes re-checked the UI toolkit for all three systems. Qt 6 Quick with a Rust core is now recommended over Slint, and Tauri is rejected on evidence (WebView2 fails under uiAccess, and it uses about 200–320 MB of memory). Phase 1 became a measured test round (ADR-0011): Qt vs Slint, with a quick Tauri check, on the development PC, in weak mode, on GitHub's Mac and Linux machines and in a Linux virtual PC. The owner set a download limit of 80 MB, chose the system accent colour for Native Adaptive, and made the Linux helpers opt-in.

## 2026-09-26 — Project named, moved to D:, rules added
The project is now called KeyXtend ("the eXtended keyboard"): Claude's recommendation, built on the owner's "Extended Keyboard" idea, and the owner can still rename it. The code prefix is `kx-` and the project root is `D:\Projects\KeyXtend`. New rules were added: nothing hardcoded, short texts, small files, review every file, plans without code, and push only with the owner's yes. New skills: `kx-review`, `kx-plan`, `kx-feature`. The future features F1–F12 are listed in `docs/FEATURES.md`.

## 2026-09-26 — Stack and architecture verified
Three research passes verified Rust + Slint (no Windows showstopper), free local AI (whisper.cpp), licences (GPL-3.0-or-later) and free signing (SignPath). The keyboard process itself must hold uiAccess, so untrusted data and the network moved to worker processes. The spec, `ARCHITECTURE.md`, ADRs 0001–0010 and the roadmap (P0–P15) were written.

## 2026-09-26 — Mock-up v2
The mock-up was rebuilt after the owner's review. Right-click now keeps normal clicks, the scroll pad has two speeds (third tap stops), and modifiers work like osk.exe. Clipboard and quick-fill lists are paged five rows at a time and work with the arrow keys, and panels scale with the keyboard. Three themes remain, each with a soft dark version.

## 2026-09-26 — Requirements and mock-up v1
Six rounds of pop-up questions defined the keyboard, the mouse helpers, voice, clipboard, the vault, snip and the window rules. Mock-up v1 compared five keyboard styles built from real references.
