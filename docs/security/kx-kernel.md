# Security note: kernel, settings and log

- **Owner:** product owner
- **Last review:** 2026-09-29
- **Status:** Reviewed
- **Scope:** `kx-module-api`, `kx-kernel` (lifecycle, order, bus, registry, grants, `Kernel`, log), `kx-settings`, `kx-platform`. They run inside the uiAccess `keyxtend` process from P3 on. The test crates (`kx-platform-fake`, `kx-test-support`) and the tools never ship.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None | The kernel enforces capabilities for modules; it needs none itself. | N/A |

- A service is gated by the capability its provider stored, and a provider must itself hold that capability. Only the kernel builds `Grants`.
- Capabilities are policy inside one process, not a sandbox (ADR-0005).

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| `settings.toml`, `settings.previous.toml`, `settings.broken.toml` | Public (no secrets or API keys in P2) | Data folder (ADR-0015) | No | Until the user changes them; one previous and one broken copy |
| `logs/keyxtend.log`, `logs/keyxtend.previous.log` | Public (sensitive fields redacted) | Data folder | No | This run and the previous one, each capped at `log_max_mb` |
| Events and services passed between modules | Up to Sensitive, later (for example typed keys) | Memory only | N/A | Not kept; never logged by the bus or kernel |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `settings.toml` | TOML parse; each module section checked by its own `validate`; unknown keys reset; version checked, with migrations | Read capped at `MAX_FILE_BYTES` (1 MiB); repair work bounded by the defaults' size; notices name at most `MAX_NAMED_KEYS` (10) keys of at most `MAX_KEY_CHARS` (64) characters; nesting capped by the `toml` parser | Property tests only (see Should fix) |
| `[kernel]` section | `KernelSettings` with `deny_unknown_fields`; `log_level` from `LOG_LEVELS`; `log_max_mb` from 1 to 100; no empty ids in `disabled` | As above | Property tests only |
| Module code (validators, migrations, `start`, `stop`, bus handlers) | Each call runs inside `catch_unwind`; a panic fails that module alone | N/A | N/A |

## Threats (STRIDE-lite)
- **Spoofing:** no IPC or network in these crates. Module ids are fixed strings in the binary; a duplicate or reserved id is refused at `Kernel::add`.
- **Tampering:** a hand-edited or damaged settings file.
  - Each value is checked, and only a bad value resets. An unreadable or oversized file is set aside as the broken copy, and the last good copy comes back.
  - Saving writes a temp file and renames it, so a crash never leaves half a file. A file damaged while the app runs is kept as the broken copy, not overwritten.
- **Logging / repudiation:**
  - The redacting formatter prints `‹redacted›` for any field whose name contains a `SENSITIVE` part (`text`, `typed`, `chars`, `clipboard`, `password`, `secret`, `api_key`, `token`, `transcript`), whatever its case or separators.
  - Control characters, line separators and bidi controls (including the Arabic letter mark) are escaped, so no value can fake or reorder a log line.
  - The kernel logs a settings error by its kind only, never the file value; the bus logs only the event name and subscription id.
- **Information disclosure:** nothing is sent anywhere. Logs stay in the data folder.
- **Denial of service:**
  - A huge or deeply nested settings file: size cap, nesting cap, and repair bounded by the defaults (a 5,000-key section needs 5 checks).
  - A runaway log: writing stops at `log_max_mb` with one final line, and write errors never panic.
  - A panicking module: contained; the rest keep running, and a notice offers Try again.
- **Elevation:** the uiAccess process parses the user's own settings file (ADR-0004 keeps media and network parsing out, not the app's own settings).
  - The parser is memory-safe Rust with the limits above, and these crates forbid `unsafe`.

## Findings
### Must fix (blocks Definition of Done)
- Done: repair work on a large section was quadratic in the key count (a start-up hang); it is now bounded by the defaults' size.
- Done: sensitive field names matched only exactly; they now match by name part, ignoring case and separators.
- Done: bidi controls and line separators are escaped in log lines.
- Done: settings errors could quote a file value into the log; only the error kind is logged.

### Should fix (tracked issue)
- A fuzz target for `Store::load` and the repair path (needs a nightly toolchain, so it waits for the owner's yes to install it). Until then, property tests cover repair and ordering.
- Bus events have no capability gate: any module can subscribe to, intercept or publish any event. P3 decides it before any typed-text event exists (`docs/items/P3.md`).
- A settings file that cannot be opened for a reason other than "not found" (for example another program's lock) counts as damaged. P3 treats it as "run on defaults in memory and do not save" (`docs/items/P3.md`).
- The app must install a quiet panic hook, because caught panics still print their message through the default hook (`docs/items/P3.md`).

### Accepted (with reason)
- Settings and logs are not encrypted: in P2 they hold no Secret or Sensitive data. ADR-0010 applies when a module stores such data.
- A same-user program can edit the settings file. It can already act as the user; the file is fully validated, so the worst case is a reset setting and a notice.
- Containment needs unwinding: no build profile may set `panic = "abort"` (ARCHITECTURE.md).
