# Security note: kx-platform

- **Owner:** product owner
- **Last review:** 2026-11-16 (P11 task 28: checked against the code; the capabilities row names the speech and model files ports' gates); 2026-11-15 (P11 task 22: the shipped voice files' names, `speech::shipped`, and `Assets::beside` and `within`, which only join paths); 2026-11-15 (P11 task 14 part b: the processor's cores and memory and the built-in mark in `Device`, the clips' languages; nothing new crosses the pipe); 2026-11-15 (P11 task 14: the speech and model files hosts; its review: a keep answered by any ending fault and capped, a check's callback closed on return, no callback under the job's lock, no progress while paused, the folder made guarded from the start); 2026-11-05 (P9 `kx-security-check`: the host's checks on vault replies, nothing new; P9 Task 23: the neutral worker host, its vault job and the hidden-run rules); 2026-10-31 (P8 security check: the host's checks on a worker treated as compromised, and its command words from `kx-ipc`; P8 task 9: the media worker's host and its review fixes; P8 final fix: the DIB ask)
- **Status:** Reviewed
- **Scope:** `crates/kx-platform`'s neutral worker host (`worker`) and the two ports it serves over an adapter's `Spawner` (ADR-0004): `Pictures` through `pictures` (ADR-0034 decision 5), and `Boxes` through `boxes` (P9, ADR-0038 decision 6). P11 adds `Speech` through `speech` and `ModelFiles` through `models`, over the worker's `voice` and `fetch` jobs (ADR-0042, ADR-0043, ADR-0044). It also holds the hidden-run rules (`mirror`, ADR-0037). The folder rule, the pack, field-watch and selection-watch halves (`selection`, P10) and the UI ports, P8's `Reveal` included, are covered with their adapter in `kx-platform-windows.md`.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None | The `Pictures` port needs none (ADR-0034 decision 3). `BoxesKey` needs `VaultData`, which the kernel checks for the module that asks (ADR-0039). `SpeechKey` needs `Microphone` and `ModelFilesKey` needs `Network`, checked the same way (ADR-0042). The host only hands bytes to its own worker and checks what comes back. | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Image bytes in a draw or a DIB ask (copied by the user) | Sensitive | Memory: one copy for the session thread, then the pipe | No | Until written to the worker |
| Pixels or a DIB in a reply | Sensitive | Memory, as `Pixels` or bytes behind `Redacted` | No | Handed to the caller |
| A vault ask's password or notebook part | Secret | Memory: wiped copies for the `kx-boxes` thread, then the pipe. Each ask is written unbuffered | No | One call |
| A vault reply's opened parts and KeePass values | Secret | Memory: read through a buffer wiped after each frame, and handed over in wiped buffers without a copy. A custom field's label is the port's plain `Redacted` | No | Handed to the caller |
| A vault ask's padlock, box bytes and KeePass bytes, and a reply's padlock and sealed bytes | Sensitive: already locked | Memory, unwiped | `age` or KDBX | One call |
| Files `mirror` lists or deletes in a hidden copy: paths only, never contents | Sensitive (paths hold the account name) | Memory only, never logged; `Mirrors` has no `Debug` | N/A | One call |
| A dictation's recording, kept or handed to a new job after a model change (P11) | Secret | Memory: each piece in a wiped buffer (`SecretBytes`), sent as `Wiped::Kept`; a recording a new job could not take stays parked in the run until it closes | No | Until the run closes or its job ends after `idle_min` |
| Dictated text pieces and a check's texts (P11) | Secret | Memory, as the port's `Redacted` secrets, told to the run's callback | No | Handed to the caller |
| Model, detector and clip paths, folder scans and download fields (P11) | Sensitive (paths hold the account name) | Memory only, never logged; `Assets` has no `Debug` | N/A | One call or run |
| The PC's total and free memory, the processor's real cores and the clips' languages the speech port tells (P11 task 14 part b) | Public (numbers and ISO codes) | Memory, in each `Device` and `Speech::clips` | No | One call; a built-in card is marked by the worker's own kind, never guessed |

The host logs only why a worker failed or stopped and a fixed note for its exit code, never bytes or sizes of content.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Replies from a worker that decodes hostile images and may be compromised | `kx-ipc` checks each frame. The host takes only a reply whose id and kind are the ask's, pixels only through `Pixels::new` (each side from 1 to the side asked, width × height × 4 bytes), and a DIB only within the cap it asked, never parsed; anything else, a cut or broken frame, or a closed pipe ends the worker | One ask at a time within `deadline_ms`; the reader hands over a frame only when the host waits for one, so it holds one unasked frame at most | Property tests in `kx-ipc`; scripted tests of every bad reply (`a_dib_past_its_cap_late_stray_or_of_another_kind_ends_the_worker`, `a_dib_reply_to_a_draw_ends_the_worker`) |
| Replies from the vault worker (P9) | The same frame and id checks. A reply of another kind ends the worker; a fault that call cannot have becomes `Failed` (`boxes/reply.rs`). Opened parts and entries still pass the module's readers, as a compromised worker can send any | One ask at a time within `[vault] vault_ms` | Property tests in `kx-ipc`; scripted tests (`boxes/tests.rs`) |
| Notes a streamed worker sends on its own (P11: the voice and fetch jobs, ADR-0042 decision 1) | Each note is a `kx-ipc` frame of its job's kinds within its size range, told in order; an unknown kind, an oversized or broken frame, a note on an id never asked of that worker, or a closed pipe ends the worker, told as `Lost`. An ask the frames refuse is never sent, and an ask on a run whose worker ended starts none | A watched id must hear a note within its `Wait`, judged by when the reader took it, or the worker is ended (`Late`), then the same doubling backoff. A worker's reader waits once `IN_FLIGHT` (2, `worker/stream.rs`) of its frames wait for the stream, and the pipe reader holds one more, so a flood stays in the worker's pipe and the app holds 3 of its frames at most (about 3 MiB of sound pieces) | Scripted tests (`worker/stream/flow/tests.rs`, `worker/stream/tests.rs`), a flood test (`worker/stream/tests/load.rs`) and a property test (`worker/stream/flow/tests/props.rs`) |
| The voice job's notes to the speech host (P11) | A note goes only to the run whose id it carries (`speech/router.rs`); a stray id is dropped. Sound pieces reach only a waiting `keep`, and any fault that ends the id reaches it too, so a keep never waits on; a `NoSound` fault dims Keep and Try again. A worker death or idle end is told once to each run, which then answers `NoSound` or `Gone`, never a hang. A check's callback is closed before `check` returns, so nothing is told after it | Each ask under its `Wait` from `[voice]`: the ask wait while listening, then load + warm + piece once the microphone closes (`Wait::Shift`); a check's wait covers two loads and reads. A keep takes at most `MAX_SOUND_PIECES` pieces and `MAX_RECORD_BYTES`, else `Internal`, so a broken job cannot fill the app's memory | Scripted tests (`speech/tests/*.rs`, `a_keep_the_job_answers_with_another_fault_returns_it_and_never_hangs`, `a_keep_past_the_longest_recording_fails_and_keeps_no_more`) |
| The fetch job's notes to the models host (P11) | Each maps to a fixed `FetchNote`; no progress while paused, as the fake; nothing after the end is told, and a dropped run's callback goes first | The Get's first note within `ask_ms`; the job's own `stall_s` ends a stalled download | Scripted tests (`models/tests/fetch.rs`) |
| A models folder's entries (P11) | Links, junctions and hidden or system entries are skipped, and a name that is not UTF-8 is left out; every path must pass `path_ok` first. A make creates the missing folders top first, the first one this host makes guarded from the start through the adapter, so a folder another program made keeps its rights; a failure removes only the empty folders this make made | One folder per scan | Tests on a real folder (`models/tests/folder.rs`) |
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
- **P11 hosts:**
  - Each model runs in its own job, capped at its row's memory plus `job_mb` and the recording's room; a check and a file's facts run in jobs of their own, so a hostile model file never shares a job with another model.
  - A callback is never called with the router or the dictation job locked, and a run may close from inside its own callback (`ear.rs`), so a module cannot stall the host.
  - A download is checked for room before its job starts; a cancel its job can no longer take deletes only that download's part file.
- **Elevation:** the host has no `unsafe` and no OS call; the adapter starts the worker from a restricted token, with no admin group, uiAccess or privilege but change-notify, at Medium (`kx-platform-windows.md`, Elevation (worker)). The worker's command words and exit codes come from `kx-ipc`, the one copy the worker reads too.

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- None.
### Accepted (with reason)
- A draw can take more than `deadline_ms` in rare paths: an idle stop just before it, or a worker that closed its output but has not yet ended, each wait `deadline_ms` at most for the worker to quit.
- A worker death tells `Fault(Gone)` to every open run of that job, even one only holding its recording, as the recording went with the job (P11).
- A paused download keeps its fetch job running until the run resumes, cancels or drops, as the job holds the open download (P11). The job sits idle then, and `job_mb` is only its ceiling, so it costs a few MB.
- A refused Get is `Host`: `Link` and `FetchAsk::check` already hold the frames' rules, so it should never come. A read-only folder is not probed first; the job, under its own restricted token, tells it at once as a `Disk` fault (`Io`), which is the check that counts (P11).
- A broken worker that keeps talking on a keep's id without its last piece holds that keep, and the dictation job's lock, while it talks; it cannot grow the app's memory past the keep's cap (P11).
- A password and a notebook part reach a worker that a crafted file may have compromised, as the same worker opens both boxes and KeePass files. One job at a time, a restart after a fault and the worker's lowered rights narrow it (`keyxtend-worker.md`).
