# KeyXtend — the eXtended keyboard

A small, fast, open-source on-screen keyboard and mouse helper for people who use a computer with **one pointer button**.

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
  - **Grab** does drag and drop and text selection: hold to grab, move, click to drop.
  - A two-speed **scroll pad** scrolls whatever is under the pointer.
- **Modifiers that work with the mouse.** One click holds Shift, Ctrl, Alt or Win, as in Windows' On-Screen Keyboard. The next mouse click is sent with it, so Shift+click and Ctrl+click work.
- **No scrolling, dragging or right-click anywhere in its own UI** (ADR-0008).
  - Lists are paged, five rows at a time, and numbers use −/+ steppers instead of sliders.
  - The keyboard's own arrow keys drive every panel, and the keyboard moves by click, move, click.
- **Any installed language.** English, French and Arabic come first, with correct Arabic: joined letters, harakat, `لا` as one key, and a right-to-left settings window.
- **Private and local.**
  - Voice typing runs free on your PC.
  - Clipboard history, the vault, learned words and transcripts are encrypted.
  - No telemetry; nothing leaves the PC unless you turn that feature on.
- **Light.** Targets: a download of 80 MB or less (without voice models), 80 MB of memory or less when idle, and near-zero CPU while idle.

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

Try the interactive mock-up: [keyboard-style-lab.html](design/keyboard-style-lab.html) (download it and open it in a browser).

## Status

Early development. The foundation (P0), the UI toolkit test round (P1, Qt chosen) and the kernel and settings (P2) are done; the keyboard core (P3) is next. The build list is in [FEATURES.md](docs/FEATURES.md), and progress is in [WORKLOG.md](docs/WORKLOG.md).

## Where your settings live

- **Installed copy:** in `%LocalAppData%\KeyXtend`, in your own user folder.
- **Portable copy:** in a `data` folder beside the program. Make that folder and the program keeps everything there, so you can carry it on a USB stick.
- **Only your changes are saved.** The file `settings.toml` is plain text. A setting you never changed follows the program's newest default.
- **A damaged file is repaired, and you are told.** A bad value goes back to its default on its own. If the whole file is unreadable, it is set aside as `settings.broken.toml` and your last good copy comes back.
- **The log keeps two runs**, this one and the one before, in a `logs` folder inside that settings folder. It is small and never holds what you type, your clipboard or your passwords.
- **Nothing is sent anywhere.** The settings and the log stay on your PC.

## Documents
- Design spec: [accessible-keyboard-design.md](docs/superpowers/specs/2026-09-26-accessible-keyboard-design.md)
- Architecture: [ARCHITECTURE.md](ARCHITECTURE.md) · Decisions: [docs/adr](docs/adr/README.md)
- Roadmap and Definition of Done: [roadmap.md](docs/roadmap.md)
- Security: [SECURITY.md](SECURITY.md) · Contributing: [CONTRIBUTING.md](CONTRIBUTING.md) · Third-party: [THIRD_PARTY.md](THIRD_PARTY.md)
- Changelog: [CHANGELOG.md](CHANGELOG.md) · Code of conduct: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

## Licence
Open source under [GPL-3.0-or-later](LICENSE) (ADR-0006).
