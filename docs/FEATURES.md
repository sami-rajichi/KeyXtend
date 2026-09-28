# KeyXtend — build list

Work happens one item at a time. To start an item, type its command in Claude Code, for example `/kx-feature P3` or `/kx-feature F2`. Claude then asks its questions, writes a plan (no code), builds it test-first, reviews it and records it.

**Status values:** `Ready` · `Waiting` (on the items under Needs) · `In progress` · `Done (vX.Y)`.

## Part 1 — Version 1.0 (Windows), in order

These are the roadmap phases (`docs/roadmap.md`). Each one needs the one before it.

| # | What | Status |
|---|---|---|
| P0 | Foundation: repo, CI, rules, licence files | Done (v0.0.0) |
| P1 | Toolkit test round: Qt vs Slint, quick Tauri check; prove the risky parts (gates G1–G26). Qt chosen (ADR-0013); the tested code moves into P2–P13 (ADR-0014) | Done (2026-09-29) |
| P2 | Kernel and module system; moves in the test tools and helpers, with the settings loader and clock as models | Ready |
| P3 | Keyboard core: typing EN/FR/AR, modifiers, Native theme, window | Waiting |
| P4 | Mouse assist: Right-click hold, Grab, ring and sounds | Waiting |
| P5 | Scroll pad | Waiting |
| P6 | Shortcuts layer and Power key | Waiting |
| P7 | Word prediction and emails | Waiting |
| P8 | Clipboard history | Waiting |
| P9 | Quick-fill vault | Waiting |
| P10 | Selection helper | Waiting |
| P11 | Voice typing (local Whisper) | Waiting |
| P12 | Snip + OCR | Waiting |
| P13 | ET66 and Dolch themes, quick settings strip and settings window, EN/FR/AR interface | Waiting |
| P14 | Release pipeline and v1.0 | Waiting |
| P15 | macOS, then Linux | Waiting |

## Part 2 — New features after v1.0

F1–F12 were chosen on 2026-09-26 and F13–F16 on 2026-09-27, to be added one by one after v1.0 works well. The product owner can reorder them or add F17 and beyond.

| # | Feature | What it does | Needs | Size |
|---|---|---|---|---|
| F1 | **Camera head control** | Move the pointer with your head and click with a face gesture (smile, raised eyebrows); webcam only, free and on your PC. | P4, v1.0 | Large |
| F2 | **AI voice control** | Voice commands such as "open Chrome", "scroll down", "right-click" or "delete last word", using local Whisper plus a command list. Later, an optional small local AI for free-form requests. | P11 | Large |
| F3 | **Zoom-to-click** | Point near a tiny button, a magnified view appears, and you click inside it. | P4 | Medium |
| F4 | **More click types** | Double-click and middle-click hold modes, and a "repeat last action" key. | P4 | Small |
| F5 | **Quick system panel** | Volume, brightness, Wi-Fi, night light, plus window snap, switch and minimise, all in one paged panel. | P6 | Medium |
| F6 | **Snippets and macros** | One click types a saved text (signature, address) or runs a saved key sequence. | P3, P9 | Medium |
| F7 | **Per-app profiles** | Language, size and modes change automatically for each app. | P3 | Medium |
| F8 | **Read aloud + translate** | Reads selected text aloud and translates between EN, FR and AR with free local models (licences checked first). | P10 | Medium |
| F9 | **Emoji and harakat panel** | Paged emoji, symbols and Arabic diacritics, five rows at a time. | P3 | Small |
| F10 | **Mouse grid** | Reach any point on screen by picking grid squares, useful when the pointer is hard to aim. | P4 | Medium |
| F11 | **Voice dataset export** | Export your own voice and text pairs, with your consent, to improve recognition of your voice later. | P11 | Small |
| F12 | **Plugins** | Other developers can add features safely through sandboxed WASM plugins with permissions. | v1.0 | Large |
| F13 | **Hover to type** | A key types when the pointer rests on it for a set time, like osk.exe's hover mode. | P3, v1.0 | Small |
| F14 | **Switch scanning** | A highlight moves through rows and keys; one switch or click picks the highlighted one. | P3, v1.0 | Medium |
| F15 | **Number pad and navigation panel** | An optional panel with a number pad, Home, End, PgUp, PgDn, Insert and F-keys. | P3, v1.0 | Small |
| F16 | **Chinese, Japanese and Korean input** | An IME mode that sends key presses instead of characters, so the Windows IME can build the text. | P3, v1.0 | Medium |

**Adding your own idea:** say "add a feature: …". Claude adds it as the next F-number with a one-line description, its needs and its size.
