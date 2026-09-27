# Architecture decision records

Never rewrite an accepted ADR. Supersede it with a new one.

| # | Decision | Status |
|---|---|---|
| 0001 | [Record architecture decisions](0001-record-architecture-decisions.md) | Accepted |
| 0002 | [Rust for all code, Slint for all windows](0002-rust-and-slint.md) | Accepted; under review by ADR-0011 (Phase 1 test round). |
| 0003 | [The keyboard process itself holds uiAccess](0003-uiaccess-keyboard-process.md) | Accepted |
| 0004 | [Untrusted data and network access live in worker processes](0004-worker-processes.md) | Accepted |
| 0005 | [Micro-kernel with independent "Lego" modules](0005-micro-kernel-modules.md) | Accepted |
| 0006 | [Licence: GPL-3.0-or-later for the app](0006-licence-gpl3.md) | Accepted for now (2026-09-26). Revisit before v1.0. |
| 0007 | [AI features are local-first and free](0007-local-first-free-ai.md) | Accepted |
| 0008 | [Every UI works without scrolling, dragging or right-click](0008-paged-no-scroll-ui.md) | Accepted |
| 0009 | [Characters as Unicode, shortcuts as virtual keys](0009-input-injection-strategy.md) | Accepted. The spike verifies it in Notepad, Word, Chrome and Terminal. |
| 0010 | [Sensitive data is encrypted at rest with an OS-protected key](0010-encryption-at-rest.md) | Accepted |
| 0011 | [Choose the UI toolkit by a measured test round](0011-toolkit-test-round.md) | Accepted (process). Result pending. |
| 0012 | [Installer: Inno Setup 7](0012-installer-inno-setup.md) | Accepted |

ADR-0011 names "ADR-0012" for the toolkit result; that number went to the installer decision, so the toolkit result will be ADR-0013.
