# 31. Shortcuts are named by meaning, and each platform sends them

- **Date:** 2026-10-23
- **Status:** Accepted.

## Context

- Spec §4.2 asked for a shortcuts layer whose keys map to "the platform's combination", such as Ctrl+C on Windows and ⌘C on macOS. The owner then widened it (2026-10-23): 37 shortcuts on pages the user arranges, chains of steps such as Copy all (Ctrl+A, then Ctrl+C), F1–F12, and a tip that names the keys sent.
- The owner wants every feature built once for Windows, macOS and Linux, so a port to another platform never reworks the keyboard (2026-10-16, again 2026-10-23).
- Letter shortcuts follow the app's layout: Ctrl+A on French AZERTY is the key at the QWERTY Q, so a fixed scan code would send Ctrl+Q. The injector already maps a virtual key to the app's layout for the Keys text route (ADR-0009).
- Some shortcuts are not keys on every platform: Windows reserves Win+L for the session lock, and the power dialog is an OS call on all three (§8.5).
- Modules never call the OS (ADR-0005), and OS mappings live in adapter tables (ARCHITECTURE.md, invariant 9).

## Decision

Decided with the owner's approval of P6's design on 2026-10-23.

1. **By meaning.** The keyboard knows a shortcut only by its id, such as `copy-all` or `f5`. Its icon comes from the theme's `icons.toml`, its name from the translations, and its place from `[keyboard] shortcut_pages`.
2. **A port per platform.** `Shortcuts` in `kx-module-api`, gated by `InjectInput`, lists the ids the platform knows with the steps each one sends, for the tips, and sends one id with the user's latched modifiers held around every step.
3. **A table per platform.** Each adapter reads its own table: a step is a list of neutral key names (`ctrl`, `shift`, `a`, `f5`, `tab`), and an entry may name an OS call instead, such as the session lock. Windows maps each name to a virtual key, then to the scan code in the app's own layout as it sends. macOS and Linux add their tables in P15.
4. **Missing means hidden.** A page lists ids; one the platform does not know is skipped there, so one settings file works on every platform.
5. **Power is its own port.** `PowerDialog`, gated by a new `Power` capability, opens the system's own power dialog; `kx-mod-power` asks it when the key is clicked, so turning that module off dims the key.

## Consequences

- Adding a shortcut means one table line per platform, an icon token and three translations; no keyboard code changes.
- A test checks that every id in the default pages has a Windows entry, an icon or label, and a name in EN, FR and AR.
- The tip's key names are translated per key (`Maj` for Shift in French), so the tip reads in the keyboard's language.
- Users cannot make their own shortcuts; new ones come from the owner in updates (owner, 2026-10-23).
