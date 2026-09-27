# Roadmap

We build **one module at a time**. A module is finished only when it meets the Definition of Done below.

- Each phase starts with its own implementation plan in `docs/superpowers/plans/`, written with the `kx-plan` skill (steps only, no code).
- Phase N is item **PN** in `docs/FEATURES.md`. Start one with `/kx-feature PN`.

Versions are pre-1.0 until the Windows feature set is complete.

## Definition of Done (every module, every phase)

1. **Spec:** the module's section in the design spec is current. Any change of direction gets an ADR.
2. **Tests:**
   - unit tests for all logic;
   - property tests for every state machine or parser;
   - Slint UI tests for every view (by accessible label);
   - edge cases listed in the plan are all covered.
3. **Security:** `docs/security/<module>.md` is filled in using the `kx-security-check` skill: threat list, capabilities, data classes, what is logged, IPC inputs. Findings are fixed or tracked.
4. **Quality gates green:**
   - `cargo fmt --check`;
   - `cargo clippy --all-targets -- -D warnings`;
   - `cargo test --workspace`;
   - `cargo deny check`;
   - `cargo audit`;
   - `cargo xtask tidy`.
5. **Accessibility:**
   - every action works with short left clicks only (ADR-0008);
   - every control has an accessible role and label;
   - contrast is ≥ 4.5:1 in light and dark.
6. **Manual Windows check:** the module's checklist has been run on a real Windows 11 PC and saved in `docs/test-reports/`.
7. **Review:** `kx-review` passes on every changed file (no hardcoded values, short texts, small files). `/code-review` and `/security-review` have run, and findings are resolved.
8. **Docs:** the user-facing README section and `CHANGELOG.md` are updated. There is a `docs/WORKLOG.md` entry (timestamp + 3–4 sentences).

---

## Phase 0 — Foundation (repo, CI, rules)

**Output:** an empty but strict workspace.

- `git init` in `D:\Projects\KeyXtend`. Create the GitHub repo and remote only after the owner's yes; then protect `main` and add a DCO check.
- Workspace `Cargo.toml`, `rust-toolchain.toml` (pinned stable), `deny.toml`, `rustfmt.toml`, `clippy.toml`.
- `xtask`: `tidy` (enforces the ARCHITECTURE.md invariants), `licences`, `dist` (stub).
- CI (GitHub Actions on `windows-latest` and `ubuntu-latest`): fmt, clippy, tests, cargo-deny, cargo-audit, tidy, Dependabot for Cargo and Actions, secret scanning.
- Files: `LICENSE` (after decision D2), `THIRD_PARTY.md`, `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `.github/CODEOWNERS`, and issue/PR templates.
- Choose the installer tool after a licence check (spec D8).

## Phase 1 — Toolkit test round (throwaway branch `spike/toolkits`)

**Purpose:** prove the risky parts and choose the UI toolkit by measurement (ADR-0011).

- **Candidates.**
  - Qt 6 Quick with a Rust core (cxx-qt) and Slint 1.18 build the same small test keyboard.
  - Tauri runs only G2, G3 and the memory part of G8.
- **Run once.** Gates marked "core" test the shared Rust core, so they run once. Every other gate runs per toolkit.
- **Stages** (decided 2026-09-27), each ending with a Pass/Fail checkpoint:
  - 0: tools and the dev certificate;
  - 1: deal-breakers G1–G6, G17, G18;
  - 1b: feature probes G19–G25;
  - 2: looks, speed and accessibility G7–G13;
  - 3: soak G14;
  - 4: other systems G15, G16, G26;
  - 5: results (ADR-0013) and cleanup.
- **Early stop.** A toolkit that fails a must-pass gate stops there: G2–G4, G7–G16, and the window parts of G19, G22, G23 and G24.
  - A failed core gate is a design finding for both toolkits; it never eliminates one.
  - Known OS walls (GNOME Wayland) are recorded, not counted as a fail.
- **Machines.**
  - The development PC, normally and in **weak mode**: a cheap 4 GB laptop (Celeron N4020 class), imitated as 2 cores at about 30 % speed, 4 GB and software rendering.
  - GitHub's macOS, Ubuntu and Windows machines.
  - A Linux virtual PC on D:.

| Gate | Pass condition |
|---|---|
| G1 Typing (core) | 1,000 random EN/FR/AR characters, including `لا`, harakat and AltGr symbols, arrive intact in Notepad, Word, Chrome, Windows Terminal, and our CI test window |
| G2 No focus | 1,000 clicks on keys; the target app keeps focus and its caret every time |
| G3 Top band | The signed, Program Files, uiAccess build, started by a standard user, stays above the Start menu, Search and Task Manager |
| G4 Admin windows | Types into an elevated window (for example an admin PowerShell). Record whether this needs anything beyond uiAccess |
| G5 Hold engine (core) | With Right-click on, over normal and admin windows: short clicks and drags are unaffected, a 1.5 s still hold becomes a right-click, and the hook never times out |
| G6 Scroll (core) | Scrolls up, down, left and right in a long Notepad text, a Chrome page, an Explorer list and a Word document, using the last point outside the keyboard as the target |
| G7 Arabic UI | Joined legends, `لا`, harakat on ◌ (Noto Sans Arabic for Dolch), a mirrored right-to-left panel, and an Arabic text field whose caret moves correctly |
| G8 Budgets | Spec §10 targets on the development PC and in weak mode. Frame times measured with PresentMon; a pressed key lights up on the next frame |
| G9 Window flags survive | No-focus, tool-window and topmost flags stay set after hide/show, a topmost toggle, a click-through toggle and a resize |
| G10 Screen readers | Narrator and NVDA read and activate every key |
| G11 DPI | 100/150/200 % on the development PC's one screen keep the size and sharpness. The move between mixed-DPI monitors is checked in P3 |
| G12 Overlay | The pointer ring runs at 60 Hz without dropping clicks underneath it |
| G13 Themes | Native Adaptive in light and dark matches the mock-up side by side and follows the system accent, dark mode and high contrast live. Two frosted-glass tricks are tried on the inactive window; otherwise the plate is solid. A sampler row of ET66 and Dolch keys (colour marks, LED, pressed inset) matches the mock-up |
| G14 Soak | 4 hours of automatic typing, both keyboards in the same session: memory does not grow and nothing crashes |
| G15 macOS and Linux, automatic | On GitHub's macOS and Ubuntu machines the test keyboard builds, stays on top and never takes focus |
| G16 Linux, hands-on | In the Linux virtual PC (GNOME and KDE, Wayland and X11): stays on top without focus, types, and scales at 125/150 % |
| G17 Grab (core) | Click, move, click drags a file in Explorer and selects text in Notepad, Word and Chrome. Esc cancels, and the button is never left pressed |
| G18 Modifier + click (core) | Shift+click extends a selection in Notepad and Chrome; Ctrl+click selects several files in Explorer |
| G19 Selection helper | A selection in Notepad, Word and Chrome is noticed without copying it (core). The pill appears without taking focus, and its Copy button copies |
| G20 Clipboard (core) | Every copy is seen; copies marked "exclude from history" are skipped; one click pastes an older entry into Notepad and Word |
| G21 Password fields (core) | Focus in a password box is flagged in Chrome and in a Win32 password field; normal fields are not flagged |
| G22 Voice | The mic records in a separate worker process (core); the caption bar never takes focus. Local Whisper (base model) transcribes EN, FR and AR; its speed is recorded normally and in weak mode |
| G23 Quick-fill | Windows Hello opens from the no-focus keyboard; after Yes, test User and Password values are typed into a login page; after Cancel nothing is typed |
| G24 Snip | The capture overlay covers every window; a region is picked by click, move, click; the image matches the region at 125 % |
| G25 Language and shortcuts (core) | The language key also switches the target app's input language; Ctrl+C/V/Z, Win+V and Alt+Tab work from the keyboard |
| G26 Other languages (core, GitHub Windows) | German, Russian, Hebrew and Persian labels match their layouts and their text arrives intact; a Japanese IME fed key presses produces 日本 |

**Decision rule:** ADR-0011. **Kept:** `cargo xtask dev-cert`, `dev-install` and `check-uiaccess`, merged by a normal pull request. **No leftovers:** when the round ends, the spike code, screenshots, logs, the test install, the certificate and the virtual PC are deleted. Only the numbers are kept, in ADR-0013.

## Phase 2 — Kernel and module API — v0.0.x

- Crates: `kx-module-api`, `kx-kernel`, `kx-settings`, `kx-platform`, `kx-platform-fake`, `kx-test-support`.
- Covers: lifecycle state machine, event bus (notify and intercept), service registry, capability grants, settings migrations, `Redacted<T>`, tracing with a redaction layer.
- **Edge cases:**
  - a module fails during start;
  - dependency cycle;
  - a missing required service;
  - a capability that was not granted;
  - a settings file that is corrupt or from a newer version;
  - a stop order in which dependents stop first.

## Phase 3 — Keyboard core — v0.1 "it types"

- `kx-mod-layouts`: reads installed layouts, label tables for EN/FR/AR, `ToUnicodeEx` with flag 0x4.
- `kx-mod-keyboard`: key model (§4.1), modifiers kx-style (§4.4), Caps, layers, language key (carousel, cycle or list), panel key routing.
- `kx-platform-windows`: `SendInput` Unicode + VK, `dwExtraInfo` tag, the no-activate topmost window, per-monitor DPI v2.
- `kx-ui`: the Native Adaptive theme (light + soft dark), keyboard view, top bar, move (click, move, click), minimise to bubble, size S/M/L plus steps.
- `kx-crypto` (needed later, but it is small and foundational).
- **Edge cases:**
  - modifiers + AltGr on AZERTY;
  - Arabic shifted harakat;
  - a layout added or removed while running;
  - a target window that drops `VK_PACKET`;
  - multi-monitor with mixed DPI;
  - moving the keyboard off-screen;
  - a key click while a modifier is held in "Stay held" mode.

## Phase 4 — Mouse assist — v0.2

- `kx-mod-mouse`: the hold engine (§5.1) with a fake clock; Right-click toggle; Grab for drag-drop and text selection; modifier+click.
- Overlay window: ring, burst, badge. Sounds (CC0, generated).
- **Health monitor:** detects when Windows removes the hook and reinstalls it.
- **Edge cases:**
  - press on our own window;
  - press → move 6 px vs 8 px;
  - release exactly at T;
  - a second button pressed during Pending;
  - the app crashing while a Grab has the button latched;
  - a hold over an admin window;
  - the target app hanging;
  - Esc during a carry;
  - switching mode during Pending.

## Phase 5 — Scroll pad — v0.3

- `kx-mod-scroll`: the two-speed controller, target tracking and outline; UIA ScrollPattern → posted wheel → SendInput.
- **Edge cases:**
  - the target closes while scrolling;
  - the target is not scrollable;
  - the end of the content is reached;
  - direction switches;
  - scrolling our own paged lists.

## Phase 6 — Shortcuts layer and Power — v0.3

- 15 shortcuts mapped per OS; Win+V routing (setting); the Power dialog.

## Phase 7 — Word prediction — v0.4

- `kx-mod-predict`: trie + n-gram, wordfreq seed data (CC-BY-SA, separate files), the learner, `@` domains, saved emails, hold-@ and hold-P.
- Password-field detection with UIA `IsPassword` (intercept event).
- **Edge cases:** mixed scripts in one word, an RTL caret, a very long token, learning off in password fields, "forget word".

## Phase 8 — Clipboard history — v0.5

- `kx-mod-clipboard` plus the `media` worker (thumbnails).
- The encrypted store, retention, exclusion formats, the paged panel, multi-paste, and pasting files and images.
- **Edge cases:**
  - a 200 MB image;
  - 10,000 entries;
  - a password-manager copy (must be skipped);
  - a file reference whose file was later deleted;
  - two paste targets.

## Phase 9 — Quick-fill vault — v0.6

- `kx-mod-vault`: the encrypted file, Windows Hello unlock, idle lock, tabs, User/Password fill, `age` export/import, and CSV with a warning.
- **Edge cases:**
  - Hello unavailable (fall back to the master password);
  - a corrupt file;
  - a wrong import format;
  - a locked vault during hold-P;
  - filling while the focus changes.

## Phase 10 — Selection helper — v0.6

- UIA TextPattern selection detection, with the gesture fallback; the non-activating pill; no Ctrl+C probing.

## Phase 11 — Voice typing — v0.7

- `kx-mod-voice` plus the `voice` worker.
- Recording, local whisper.cpp, model download with SHA-256 check, benchmark button, forced language, caption bar, cloud with the user's own key, transcript history.
- **Edge cases:**
  - no microphone;
  - the microphone is in use;
  - a 10-minute dictation;
  - only silence;
  - a network failure mid-upload;
  - a model file that fails the hash check;
  - the user cancels while transcribing.

## Phase 12 — Snip + OCR — v0.8

- `kx-mod-snip` plus the `ocr` worker.
- Capture modes (Region, Window, Full screen, Scrolling, timer), the compact editor, OCR fallback chain, pin on screen.

## Phase 13 — Themes, settings, i18n — v0.9

- ET66 and Modern Dolch (light + soft dark).
- The complete settings window (paged, steppers).
- UI translations in EN/FR/AR, with an RTL settings window.

## Phase 14 — Release pipeline — v1.0 (Windows)

- A tag triggers CI:
  - deny, audit;
  - `cargo auditable build --release`;
  - CycloneDX SBOM;
  - the installer;
  - `actions/attest`;
  - SHA-256 checksums;
  - a GitHub Release.
- Apply to the SignPath Foundation once v0.x releases exist. Until then: unsigned releases with a clear README.
- winget manifest; Scoop Extras.

## Phase 15+ — macOS, then Linux

- macOS: `kx-platform-macos` (NSPanel, CGEventTap, CGEventPost, Keychain).
- Linux: `kx-platform-linux` (X11 first; Wayland through the RemoteDesktop portal and layer-shell; GNOME's own secondary click).

## After v1.0

New features F1–F16 are listed and numbered in `docs/FEATURES.md`. Start one with `/kx-feature F<n>`.
