# 37. A run whose saves would be hidden saves nothing

- **Date:** 2026-11-01
- **Status:** Accepted. Adds to ADR-0015.

## Context

- ADR-0015 puts the installed edition's data in `%LocalAppData%\KeyXtend`.
- A KeyXtend started from a shell of a packaged app (the Claude desktop app, Codex) wrote its files into that package's private copy, `%LOCALAPPDATA%\Packages\<package>\LocalCache\Local\KeyXtend`. A KeyXtend started from the Start menu never saw them, and the owner lost a day's history and learned words (2026-11-01, `docs/items/P9.md`).
- Microsoft documents that new files under AppData of a packaged app go to a private per-package place, merged into its view, and that children are created outside the package's identity ([behind the scenes](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes), [flexible virtualization](https://learn.microsoft.com/en-us/windows/msix/desktop/flexible-virtualization)).
- Field reports show the children keep the redirection anyway, so `GetCurrentPackageFullName` reports no package for them, and the breakaway flags do not lift it ([claude-code#94254](https://github.com/anthropics/claude-code/issues/94254), [codex#52698](https://github.com/openai/codex/issues/52698)).
- Relaunching through Explorer escapes it, but Microsoft does not document that use, and the session's tools refused that move as a sandbox bypass.

## Decision

Decided with the owner on 2026-11-01 (P9).

1. **The probe.** Before the log opens and before the kernel reads settings, the platform checks the data folder when it lies under the user's AppData. It writes a small, randomly named probe file there, looks for it in each package's private mirror of that path, and deletes it. Found means a hidden run. The UI port `Hidden` in `kx-platform` carries the probe; the Windows adapter fills it from `hidden.toml`.
2. **A hidden run saves nothing.** It starts fresh: default settings, nothing read from the data folder, no log file, and in-memory stores (`MemoryStore`, `MemoryShelf` in `kx-crypto`), so copies and learned words of that run end with it. The Quick-fill module stays off with its note, and a banner says to start KeyXtend from the Start menu.
3. **No relaunch.** KeyXtend never restarts itself through Explorer or any other route out of a package.
4. **Hidden copies.** A normal run lists `KeyXtend` folders in other packages' private copies (`Hidden::copies`). For each one not in `[kernel] kept_copies`, an asking notice offers to delete it. Yes deletes it at once, with no recycle bin; No adds it to `kept_copies`, so it is never asked again.
5. **Other platforms.** macOS and Linux fill `Hidden` with no hidden run and no copies; P15 checks the macOS App Sandbox container.

## Consequences

- One data folder: a KeyXtend that cannot save where the user will find its data saves nothing.
- Tests on this PC can see a hidden run, since its sessions run inside the Claude package; a live test behind `KX_LIVE_HIDDEN=1` checks it, and CI checks the normal case.
- Notices gain an optional ask (two answers), which ADR-0038's Quick-fill prompts reuse.
- A copy the user keeps stays on disk; deleting it later is the user's own step.
