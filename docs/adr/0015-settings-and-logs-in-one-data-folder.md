# 15. Settings and logs live in one data folder

- **Date:** 2026-09-29
- **Status:** Accepted

## Context

The kernel (P2) needs one safe place for the user's settings and logs. The place must survive a hand edit, a crash while saving, and a change of version.

- The keyboard needs uiAccess, and uiAccess needs an install under Program Files (ADR-0003). Program Files is write-protected for normal users, so the app cannot keep its data beside the program there.
- Some users want a copy on a USB stick that leaves nothing behind on the PC.
- Every module owns its own settings (ADR-0005), and the file must not break when one module changes.

## Decision

Decided by the owner on 2026-09-29.

- **Where:**
  - the portable edition keeps its data in a `data` folder beside the program;
  - the installed edition uses `%LocalAppData%\KeyXtend`;
  - `AppDirs::data_dir` in `kx-platform` picks between them: portable when a `data` folder sits beside the program;
  - a new user starts clean, with no settings file.
- **What:**
  - one plain TOML file, `settings.toml`;
  - one section per module, named by its id, with its own `version`;
  - the kernel has its own section, `kernel`, for the switched-off modules and the log limits;
  - only the user's changes are stored, and the defaults live in each module's embedded `defaults.toml`;
  - sections are flat: a nested table is compared and reset as one value.
- **Repair:**
  - a bad value resets alone, and values that are valid only together are kept;
  - a damaged file (also one over 1 MiB) is moved to `settings.broken.toml`, and the last good copy, `settings.previous.toml`, comes back;
  - with no good copy, the defaults are used;
  - a section from a newer version, and a section no module owns, are kept as they are;
  - a module whose settings check panics fails alone, and the rest start;
  - a notice names what changed, by a translation key (the texts come with the UI in P3);
  - saving writes a temp file and then renames it, so a crash never leaves a half-written file.
- **Changes:** changing a module's settings restarts that module, and the modules that need it, until P3 adds a live settings handle.
- **Logs:**
  - the `logs` folder sits inside the data folder: `logs/keyxtend.log` holds this run and `logs/keyxtend.previous.log` the run before, as NVDA does;
  - each file is capped at `log_max_mb`, and later lines are dropped once it is full;
  - fields with a sensitive name are printed as `‹redacted›`;
  - typed text, clipboard content, secrets and transcripts are never logged, and the log is never sent anywhere.

## Consequences

- No module knows a file path: the kernel reads a module's section, migrates and repairs it, and hands it over.
- A settings file that only holds changes stays small, and a new default reaches users who never changed that value.
- Repair has a cost: a value the user set wrongly goes back to its default, and the notice says which one.
- The Windows adapter (P3) reports the two folders, and the app picks the data folder from them.
- What the uninstaller does with the data folder is decided when the installer is built, in the release phase (P14, `docs/roadmap.md`).

## Alternatives

- **Roaming AppData:** the settings would follow the user across PCs, but the logs and any machine-bound data would follow too; refused.
- **Every setting in the file:** easy to read, but a new default never reaches a user who has an old file; refused.
- **A database:** atomic, but not readable or repairable by hand; refused for a small, human-scale file.
- **Resetting a whole feature on one bad value:** simple, but it throws away good settings; refused.
