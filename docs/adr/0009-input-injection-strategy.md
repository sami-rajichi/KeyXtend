# 9. Characters as Unicode, shortcuts as virtual keys

- **Date:** 2026-09-26
- **Status:** Accepted. The spike verifies it in Notepad, Word, Chrome and Terminal.

## Context
If we send virtual keys for characters, the result depends on the target's keyboard layout. For example, AZERTY versus QWERTY garbles letters, and Arabic breaks. On the other hand, Ctrl+C sent as a Unicode "c" does not work in most apps.

## Decision
- Characters: `KEYEVENTF_UNICODE`, so the result does not depend on the layout.
- Shortcuts, navigation and modifiers: VK plus scan code.
- Per-app fallback list for apps that drop `VK_PACKET`: map the character back through `VkKeyScanEx` in the target's layout.
- Every injected event is tagged in `dwExtraInfo` so our hooks can ignore it.

## Consequences
The Windows CI test target window verifies EN, FR and AR character streams, including `لا`, harakat and AltGr characters.
