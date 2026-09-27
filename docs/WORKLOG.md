# Work log

Newest first. One entry per implemented feature or fixed issue: a timestamp plus 3–4 sentences at most.

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
