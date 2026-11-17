# 46. The voice module opens its models folder through a reveal service

- **Date:** 2026-11-15
- **Status:** Accepted.

## Context

- The Models page has an Open button that shows the models folder in the file manager ([`2026-11-10-voice-typing-design.md`](../superpowers/specs/2026-11-10-voice-typing-design.md), The gear and the Voice panel).
- The UI's `Reveal` port (ADR-0034) shows a folder, but it belongs to the UI, has no capability gate and no module reaches it.
- A module service that opens any path could open a file or follow a link to a place the user never picked. The models folder is a plain folder the user chose.
- Translate's `OpenLink` (ADR-0041) is the pattern for a gated service that one module holds.

## Decision

1. **A gated port.** `FolderReveal` in `kx-module-api` shows a folder in the system's file manager. Its key `FolderRevealKey` needs a new capability, `OpenFolder`, which only the voice module holds.
2. **Plain folders only.** The port takes a `FolderPath`: a plain drive path, never a share, a device, a stream, a relative path or a part ending in a dot or a space, and its `Debug` prints no path. The Windows adapter shows it only on a fixed local drive, when it is an existing directory and no folder from the drive down is a reparse point. A file, a link, nothing or another kind of drive is refused before the shell sees it.
3. **Held, then explored.** The adapter holds the folder open without delete sharing until the shell answers, so neither it nor a folder above it can be renamed in between. Like the UI's reveal and the web link, it asks the running Explorer's shell object, here with the `explore` verb, which only folders have. Nothing starts as our child, nothing inherits uiAccess, and a file put in the folder's place is never run.
4. **Registered, not yet called.** The app registers the platform's service now; the voice module's actor calls it when its Open button is wired (P11 task 20).

## Consequences

- The fake records each folder shown, so tests check the module's call without a window; a real open stays a manual check on Windows.
- A swap after the shell answers can still show another folder. Explorer has only the user's own rights there, so this is accepted.
- Other platforms implement the port with their own file manager under the same checks.
- The capability list, the app's grant tests and the security notes name `OpenFolder`.
