# Security note: kx-platform

- **Owner:** product owner
- **Last review:** 2026-11-05 (P9 `kx-security-check`: the host's checks on vault replies, nothing new; P9 Task 23: the neutral worker host, its vault job and the hidden-run rules); 2026-10-31 (P8 security check: the host's checks on a worker treated as compromised, and its command words from `kx-ipc`; P8 task 9: the media worker's host and its review fixes; P8 final fix: the DIB ask)
- **Status:** Reviewed
- **Scope:** `crates/kx-platform`'s neutral worker host (`worker`) and the two ports it serves over an adapter's `Spawner` (ADR-0004): `Pictures` through `pictures` (ADR-0034 decision 5), and `Boxes` through `boxes` (P9, ADR-0038 decision 6). It also holds the hidden-run rules (`mirror`, ADR-0037). The folder rule, the pack and field-watch halves and the UI ports, P8's `Reveal` included, are covered with their adapter in `kx-platform-windows.md`.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None | The `Pictures` port needs none (ADR-0034 decision 3). `BoxesKey` needs `VaultData`, which the kernel checks for the module that asks (ADR-0039). The host only hands bytes to its own worker and checks what comes back. | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Image bytes in a draw or a DIB ask (copied by the user) | Sensitive | Memory: one copy for the session thread, then the pipe | No | Until written to the worker |
| Pixels or a DIB in a reply | Sensitive | Memory, as `Pixels` or bytes behind `Redacted` | No | Handed to the caller |
| A vault ask's password or notebook part | Secret | Memory: wiped copies for the `kx-boxes` thread, then the pipe. Each ask is written unbuffered | No | One call |
| A vault reply's opened parts and KeePass values | Secret | Memory: read through a buffer wiped after each frame, and handed over in wiped buffers without a copy. A custom field's label is the port's plain `Redacted` | No | Handed to the caller |
| A vault ask's padlock, box bytes and KeePass bytes, and a reply's padlock and sealed bytes | Sensitive: already locked | Memory, unwiped | `age` or KDBX | One call |
| Files `mirror` lists or deletes in a hidden copy: paths only, never contents | Sensitive (paths hold the account name) | Memory only, never logged; `Mirrors` has no `Debug` | N/A | One call |

The host logs only why a worker failed or stopped and a fixed note for its exit code, never bytes or sizes of content.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Replies from a worker that decodes hostile images and may be compromised | `kx-ipc` checks each frame. The host takes only a reply whose id and kind are the ask's, pixels only through `Pixels::new` (each side from 1 to the side asked, width × height × 4 bytes), and a DIB only within the cap it asked, never parsed; anything else, a cut or broken frame, or a closed pipe ends the worker | One ask at a time within `deadline_ms`; the reader hands over a frame only when the host waits for one, so it holds one unasked frame at most | Property tests in `kx-ipc`; scripted tests of every bad reply (`a_dib_past_its_cap_late_stray_or_of_another_kind_ends_the_worker`, `a_dib_reply_to_a_draw_ends_the_worker`) |
| Replies from the vault worker (P9) | The same frame and id checks. A reply of another kind ends the worker; a fault that call cannot have becomes `Failed` (`boxes/reply.rs`). Opened parts and entries still pass the module's readers, as a compromised worker can send any | One ask at a time within `[vault] vault_ms` | Property tests in `kx-ipc`; scripted tests (`boxes/tests.rs`) |
| Asks the host writes | `kx-ipc`'s own writer checks run first (an empty image, side 0 or a DIB cap of 0 is `Refused`, one past `MAX_IMAGE_BYTES` is `TooBig`), so the worker never starts for an ask it would refuse; a side past `MAX_SIDE` and a DIB cap past `MAX_IMAGE_BYTES` are cut to them | `MAX_ASK_LEN` | Scripted tests (`an_ask_the_worker_would_refuse_starts_no_worker`, `a_cap_past_the_largest_image_is_clamped_to_it`) |
| The disk under the local app-data folder and the package folders (`mirror`, through the adapter's `MirrorDisk`) | A probe writes one fresh random name, looks for it in each package's mirror and deletes it. `remove` deletes only a copy `copies` lists now, never the data folder itself | One probe at start; lists bounded by the package folders | Unit tests over a fake disk (`mirror/tests.rs`) |

## Threats (STRIDE-lite)
- **Spoofing:** the worker is our own child on inherited anonymous pipes; ids let the host refuse a reply to another draw (`bad_replies_are_refused_and_end_the_worker`).
- **Tampering:** a compromised worker can send any frame; each is checked, and a worker that lies is ended and started afresh only after a backoff.
- **Logging / repudiation:** warnings name the fault (`Late`, `Stray`, `BadReply`, `BadFrame`, `Ended`) and a fixed exit note (`exit_note`), never content.
- **Information disclosure:** content stays in memory and on the two pipes; the worker opens nothing else (`keyxtend-worker.md`).
  - A vault ask writes straight to the pipe with no buffer, and a reply's read buffer is wiped after each frame (`worker/piped.rs`), so the host keeps no copy of a password or an opened part. What the worker's own reader keeps is in `keyxtend-worker.md`.
  - Debug output shows no value, as every ask and reply type hides them (`kx-ipc.md`).
- **Denial of service:**
  - A late worker costs one `deadline_ms` and is ended (`a_late_worker_is_ended_and_is_down_until_its_backoff_ends`). Asks go through a writer thread, so a worker that stops reading cannot block the host.
  - Failures in a row double the wait before the next start, from `restart_ms` up to `restart_max_ms`, and a good answer resets it; draws within the wait, or after a failed start, are `Down` at once with the wait left, so the module asks again then and blames no image (`the_backoff_doubles_up_to_its_cap_and_a_good_answer_resets_it`). A vault call within the wait is `Failed`, so its backup waits.
  - A worker that quit between draws is replaced before the next draw with no backoff, since no draw of ours failed (`a_worker_that_quit_between_asks_is_replaced_with_no_backoff`).
  - An idle worker says goodbye and stops after `idle_s`; dropping the session stops it too.
  - The vault's session thread is `kx-boxes`, its memory cap `VAULT_MEMORY_MB` (`Launch::vault`). A call on it is a blocking wait of up to `vault_ms`, which the module runs on its own `kx-vault-box` thread, never on its actor.
- **Elevation:** the host has no `unsafe` and no OS call; the adapter starts the worker from a restricted token, with no admin group, uiAccess or privilege but change-notify, at Medium (`kx-platform-windows.md`, Elevation (worker)). The worker's command words and exit codes come from `kx-ipc`, the one copy the worker reads too.

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- None.
### Accepted (with reason)
- A draw can take more than `deadline_ms` in rare paths: an idle stop just before it, or a worker that closed its output but has not yet ended, each wait `deadline_ms` at most for the worker to quit.
- A password and a notebook part reach a worker that a crafted file may have compromised, as the same worker opens both boxes and KeePass files. One job at a time, a restart after a fault and the worker's lowered rights narrow it (`keyxtend-worker.md`).
