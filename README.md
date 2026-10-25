# KeyXtend — the eXtended keyboard

A small, fast, source-available on-screen keyboard and mouse helper for people who use a computer with **one pointer button**.

Created and owned by [sami-rajichi](https://github.com/sami-rajichi).

## Who it is for

- People who use a computer with a single pointer button: for example a finger on a mouse or touchpad, a head or eye pointer with a dwell click, or a mouth stick.
- People who cannot use a physical keyboard, a right-click, a scroll wheel or dragging, and rely on an on-screen keyboard to type.
- Anyone else who prefers an on-screen keyboard can use it too.

The goal: with one button, do *everything* a keyboard-and-mouse user can do.

## Why another on-screen keyboard

- Windows' On-Screen Keyboard stays on top of everything, but it looks dated, has no themes, can render keys tiny, and has no clipboard history, voice typing or mouse helpers.
- The Windows 11 touch keyboard is built for touch screens.
- macOS Dwell and OptiKey solve parts of the mouse problem, but neither covers Arabic or one-button use on Windows.

## What makes it different

KeyXtend is in early development; this is what it is being built to do.

- **Always on top, never in the way.** It stays above every window, including the Start menu, Search and Task Manager, and types into admin windows. Clicking a key never takes focus from the app you type into (ADR-0003).
- **Full mouse help with one button.**
  - Hold the button still for 1.5 s to right-click; short clicks and drags keep working.
  - **Grab** does drag and drop and text selection: click to pick up, move, click to drop.
  - A two-speed **scroll pad** scrolls what you last pointed at.
- **Modifiers that work with the mouse.** One click holds Shift, Ctrl, Alt or Win, as in Windows' On-Screen Keyboard. The next mouse click is sent with it, so Shift+click and Ctrl+click work.
- **No scrolling, dragging or right-click anywhere in its own UI** (ADR-0008).
  - Lists are paged, five rows at a time, and numbers use −/+ steppers instead of sliders.
  - The keyboard's own arrow keys drive every panel, and the keyboard moves by click, move, click.
- **Any installed language.** English, French and Arabic come first, with correct Arabic: joined letters, harakat, `لا` as one key, and Arabic text that reads right to left in a settings window that keeps its places.
- **Private and local.**
  - Voice typing runs free on your PC.
  - Clipboard history, the vault, learned words and transcripts are encrypted.
  - No telemetry; nothing leaves the PC unless you turn that feature on.
- **Light.** Targets: a download of 80 MB or less (without voice models; measured deflated, a conservative stand-in for the installer's LZMA2), 80 MB of memory or less when idle, and near-zero CPU while idle.

## Features (v1.0, Windows)

- **Typing:** the keyboard for the active language, a one-click shortcuts layer (copy, paste, undo, switch app…), and word prediction with email suggestions.
- **Mouse:** Right-click hold, Grab and the scroll pad, with a ring and soft sounds that show when a hold fires.
- **Voice typing:** local Whisper, click to start and click to stop. A cloud engine with your own free key is optional and off by default.
- **Clipboard history:** text, links, images and files, newest first, with pins and "Paste together".
- **Quick-fill vault:** saved emails, names and passwords. Passwords unlock with Windows Hello or a PIN.
- **Selection helper:** a Copy button appears next to any text you select.
- **Snip:** capture a region without dragging, annotate it, and copy its text with OCR.
- **Power key:** opens the system's own shut-down dialog.
- **Themes:** Native Adaptive, ET66 and Modern Dolch, each in light and soft dark, in sizes from small to large.

macOS and Linux follow v1.0, from the same code. Later features, such as camera head control, voice commands and zoom-to-click, are listed in [FEATURES.md](docs/FEATURES.md).

The early design mock-up, kept as a record, is [keyboard-style-lab.html](design/keyboard-style-lab.html); its licence notes predate ADR-0030.

## Status

Early development. Done so far:

- P0: the foundation.
- P1: the UI toolkit test round; Qt was chosen.
- P2: the kernel and the settings.
- P3: the keyboard core, which types English, French and Arabic.
- P4: mouse assist, which right-clicks and drags with clicks only.
- P5: the scroll pad, which scrolls the window last pointed at with clicks only.
- P6: the shortcuts layer and the Power key: one click sends a common shortcut, such as Copy all or Switch app, or opens the system's power dialog.

The build list is in [FEATURES.md](docs/FEATURES.md), and progress is in [WORKLOG.md](docs/WORKLOG.md).

## Using the keyboard

- **Type** by clicking keys: the text goes to the app you were typing in, which keeps the focus.
- **Shift, Ctrl, Alt, AltGr and Win** stay held for the next key or click, then let go; a setting keeps them held until you click them again.
- **Right-click and Grab** (the side keys): with Right-click on, press and hold the button still on something (1.5 s by default) to right-click it, while short clicks and drags work as usual, and a ring shows the hold. With Grab on, one click picks up what is under the pointer and the next drops it, to move files or select text without dragging; a badge at the pointer shows which mode is on, and a soft sound plays at each step.
- **The scroll pad** (the round pad): point at what you want to scroll, come back to the keyboard and tap an arrow.
  - One tap scrolls slowly that way, a second tap on the same arrow scrolls fast, and a third stops. Another arrow turns that way, slowly.
  - **Stop** shows in the pad's middle while it scrolls, with one dot lit for slow and two for fast.
  - It also stops by itself at the end of the page, at a click outside KeyXtend's windows (with mouse assist on), two minutes after your last tap on the pad, when the window closes, and when you close or minimise the keyboard. If there is nothing to scroll there, or it is already at the end, a short notice says so.
  - A thin outline marks what will scroll while the pointer is on the pad and all the time it scrolls, and, with mouse assist on, the badge at the pointer shows the arrow and "slow" or "fast".
  - Pointed at one of KeyXtend's own lists, such as the language list, each tap turns one page: ◀ or ▲ back, ▶ or ▼ on.
  - Its settings are in the `[scroll]` section: `slow = 3` and `fast = 10` lines a second (1 to 60, fast above slow), `limit_s = 120` seconds (0 for no limit, else 10 to 1800), and `stop_at_end` and `stop_on_press`, both `true`. Four more tune how it finds, reads and moves what it scrolls; see [kx-mod-scroll's README](crates/kx-mod-scroll/README.md).
- **The shortcuts layer** (the top-right side key, Shortcuts):
  - Click Shortcuts and the side keys turn into shortcuts; the key lights up like a held Shift, and a second click brings the side keys back. The layer stays open after each shortcut, and KeyXtend always starts on the usual side keys.
  - Each shortcut shows its icon (F1 to F12 show their names), and its tip names it and the keys it sends, such as "Copy all (Ctrl+A, Ctrl+C)".
  - **Next**, in the bottom-right corner, turns the page, from the last back to the first. Its dots show which page is open, and its tip and screen readers say "Page 1 of 3".
  - A Shift, Ctrl, Alt, AltGr or Win clicked before a shortcut joins it (Lock PC goes alone), then lets go as after any key: Shift, then New tab, sends Ctrl+Shift+T.
  - Opening the layer stops the scroll pad, which hides until the layer closes; Right-click and Grab stay as they were.
  - The `[keyboard] shortcut_pages` setting lists the pages by shortcut id: editing first, then Windows, tabs and zoom, then F1 to F12. A page holds 13 shortcuts, or 14 when it is the only one, with at most 6 pages, and an id Windows does not know is skipped; the Settings window will arrange the pages (P13).
- **The Power key** opens Windows' own power dialog, where you choose what to do or cancel; KeyXtend never shuts down by itself. It dims, with a tip, when the Power key is switched off, and a notice says so when the dialog cannot open.
- **The language key** switches the keyboard and the app in front to another installed language. Keys and tips follow the keyboard's language.
- **The top bar:** the sun or moon switches light and dark, the palette changes the theme, and the lens buttons and S, M, L change the size. Click the grip, move, then click to place the keyboard; the corner resizes it the same way.
- **− and ×:** − shrinks the keyboard to a bubble, and × hides it. The tray icon's menu shows it again or quits; Windows 11 may keep a new tray icon under the ^ arrow until you pin it.
- **Starting KeyXtend again** shows the running keyboard, and `keyxtend --quit` ends it.

## Where your settings live

- **Installed copy:** in `%LocalAppData%\KeyXtend`, in your own user folder.
- **Portable copy:** in a `data` folder beside the program. Make that folder and the program keeps everything there, so you can carry it on a USB stick.
- **Only your changes are saved.** The file `settings.toml` is plain text. A setting you never changed follows the program's newest default.
- **A damaged file is repaired, and you are told.** A bad value goes back to its default on its own. If the whole file is unreadable, it is set aside as `settings.broken.toml` and your last good copy comes back.
- **Comments you type into the file are not kept.** The program writes the whole file again each time it saves.
- **The log keeps two runs**, this one and the one before, in a `logs` folder inside that settings folder. It is small and never holds what you type, your clipboard or your passwords.
- **Nothing is sent anywhere.** The settings and the log stay on your PC.

## Documents
- Design spec: [accessible-keyboard-design.md](docs/superpowers/specs/2026-09-26-accessible-keyboard-design.md)
- Architecture: [ARCHITECTURE.md](ARCHITECTURE.md) · Decisions: [docs/adr](docs/adr/README.md)
- Roadmap and Definition of Done: [roadmap.md](docs/roadmap.md)
- Security: [SECURITY.md](SECURITY.md) · Contributing: [CONTRIBUTING.md](CONTRIBUTING.md) · Third-party: [THIRD_PARTY.md](THIRD_PARTY.md)
- Changelog: [CHANGELOG.md](CHANGELOG.md) · Code of conduct: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

## Licence
Source-available, all rights reserved: you may read the code and keep a copy to read it, but not use, change or share it without permission ([LICENSE](LICENSE), ADR-0030). Earlier versions keep the licence they were published under.
