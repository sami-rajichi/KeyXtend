# 62. Paths follow each system's own absolute form

- **Date:** 2026-11-27
- **Status:** Accepted (2026-11-27), from the owner's ask that code run on every system. Supersedes "a plain drive path" in ADR-0046 decision 2 and "a drive path of the asked type" in ADR-0056 decision 6, off Windows only; on Windows both stand word for word.

## Context

- The shared path rules (`probe_ok`, `path_ok`, `FolderPath`, `SnipFile` and the pick job's answer check) took only drive paths such as `C:\…`, on every system.
- On Linux and macOS no real path has that form: the Snips folder, a models folder or a temp folder is `/home/…` or `/tmp/…`, so each was refused. Worse, `C:\x` is a relative name there.
- The Linux CI job of PR #28 failed on that: 16 Snip tests on real temp folders got `BadPath`.
- The owner asked for code that runs on every system wherever it can, so P15 needs less work, rather than tests kept to Windows.

## Decision

1. **One rule, two styles.** kx-ipc's `PathRoot` names how an absolute path starts: `Drive` (an ASCII letter, `:` and a separator) or `Slash` (one `/`). `PathRoot::NATIVE` is `Drive` on Windows and `Slash` elsewhere, and every shared rule checks the native style.
2. **The same strictness.** A `Slash` path keeps every other refusal: control characters, device names, `.` and `..` parts, parts ending in a dot or a space, and any `:`. It also refuses a path starting with `//`, which may name another machine, and any `\`, so a path's parts read the same to every check.
3. **Case.** `SnipFile::lies_in` and the voice module's `drive_of` ignore case only on Windows; `Slash` compares every part exactly.
4. **Both styles tested everywhere.** Each rule has a crate-private twin that takes the style, so Windows and Linux CI both test both styles. Tests that pass example paths through the rules write them for Windows and wrap them in `kx_test_support::paths::native`, which spells them as Git Bash does off Windows (`C:\x` as `/c/x`), so drives stay apart.
5. **Linux checked on Windows.** `--cfg kx_slash_paths` makes `NATIVE` the `Slash` style on Windows, so the Linux path rules can be tested on a Windows PC before a push. It cannot change Windows itself: a temp folder is still a drive path and `Path::join` still writes `\`, so tests built on those fail there and pass on Linux; no product code works around it.

## Consequences

- On Windows nothing changes: the same paths pass and fail.
- On Linux and macOS the Snips folder, the models folder and a dialog's answer can be real paths, so P15's adapters only need their own file manager, dialogs and drives.
- Off Windows, Windows' own refusals still apply: a part named after a device (`aux`, `nul`), ending in a dot or a space, or holding `\` or `:`; and in a dialog's answer, `<>"|?*` too. No folder KeyXtend makes holds one.
- A network mount on Linux or macOS is a plain `/` path that passes the root rule, as a mapped `Z:\` does on Windows; the adapters must check the mount, as the Windows ones check the drive (`docs/items/P15.md`).
