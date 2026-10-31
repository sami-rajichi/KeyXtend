# Security note: kx-platform

- **Owner:** product owner
- **Last review:** 2026-10-31 (P8 security check: the host's checks on a worker treated as compromised, and its command words from `kx-ipc`; P8 task 9: the media worker's host and its review fixes; P8 final fix: the DIB ask)
- **Status:** Reviewed
- **Scope:** `crates/kx-platform`'s worker host (`pictures`), which serves the `Pictures` port over an adapter's `Spawner` (ADR-0004, ADR-0034 decision 5). The folder rule, the pack and field-watch halves and the UI ports, P8's `Reveal` included, are covered with their adapter in `kx-platform-windows.md`.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None | The `Pictures` port needs none (ADR-0034 decision 3); the host only hands image bytes to its own worker and checks what comes back. | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Image bytes in a draw or a DIB ask (copied by the user) | Sensitive | Memory: one copy for the session thread, then the pipe | No | Until written to the worker |
| Pixels or a DIB in a reply | Sensitive | Memory, as `Pixels` or bytes behind `Redacted` | No | Handed to the caller |

The host logs only why a worker failed or stopped and a fixed note for its exit code, never bytes or sizes of content.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Replies from a worker that decodes hostile images and may be compromised | `kx-ipc` checks each frame. The host takes only a reply whose id and kind are the ask's, pixels only through `Pixels::new` (each side from 1 to the side asked, width × height × 4 bytes), and a DIB only within the cap it asked, never parsed; anything else, a cut or broken frame, or a closed pipe ends the worker | One ask at a time within `deadline_ms`; the reader hands over a frame only when the host waits for one, so it holds one unasked frame at most | Property tests in `kx-ipc`; scripted tests of every bad reply (`a_dib_past_its_cap_late_stray_or_of_another_kind_ends_the_worker`, `a_dib_reply_to_a_draw_ends_the_worker`) |
| Asks the host writes | `kx-ipc`'s own writer checks run first (an empty image, side 0 or a DIB cap of 0 is `Refused`, one past `MAX_IMAGE_BYTES` is `TooBig`), so the worker never starts for an ask it would refuse; a side past `MAX_SIDE` and a DIB cap past `MAX_IMAGE_BYTES` are cut to them | `MAX_ASK_LEN` | Scripted tests (`an_ask_the_worker_would_refuse_starts_no_worker`, `a_cap_past_the_largest_image_is_clamped_to_it`) |

## Threats (STRIDE-lite)
- **Spoofing:** the worker is our own child on inherited anonymous pipes; ids let the host refuse a reply to another draw (`bad_replies_are_refused_and_end_the_worker`).
- **Tampering:** a compromised worker can send any frame; each is checked, and a worker that lies is ended and started afresh only after a backoff.
- **Logging / repudiation:** warnings name the fault (`Late`, `Stray`, `BadPixels`, `BadFrame`, `Ended`) and a fixed exit note (`exit_note`), never content.
- **Information disclosure:** content stays in memory and on the two pipes; the worker opens nothing else (`keyxtend-worker.md`).
- **Denial of service:**
  - A late worker costs one `deadline_ms` and is ended (`a_late_worker_is_ended_and_is_down_until_its_backoff_ends`). Asks go through a writer thread, so a worker that stops reading cannot block the host.
  - Failures in a row double the wait before the next start, from `restart_ms` up to `restart_max_ms`, and a good answer resets it; draws within the wait, or after a failed start, are `Down` at once with the wait left, so the module asks again then and blames no image (`the_backoff_doubles_up_to_its_cap_and_a_good_draw_resets_it`).
  - A worker that quit between draws is replaced before the next draw with no backoff, since no draw of ours failed (`a_worker_that_quit_between_draws_is_replaced_with_no_backoff`).
  - An idle worker says goodbye and stops after `idle_s`; dropping the session stops it too.
- **Elevation:** the host has no `unsafe` and no OS call; the adapter starts the worker from a restricted token, with no admin group, uiAccess or privilege but change-notify, at Medium (`kx-platform-windows.md`, Elevation (worker)). The worker's command words and exit codes come from `kx-ipc`, the one copy the worker reads too.

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- None.
### Accepted (with reason)
- A draw can take more than `deadline_ms` in rare paths: an idle stop just before it, or a worker that closed its output but has not yet ended, each wait `deadline_ms` at most for the worker to quit.
