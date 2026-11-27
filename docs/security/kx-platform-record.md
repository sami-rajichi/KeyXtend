# Security note: kx-platform-record

- **Owner:** product owner
- **Last review:** 2026-11-25 (P12 task 21 `kx-security-check`: the P12 rows checked against the code; each finding fixed or accepted below); 2026-11-25 (P12 task 20: written from the code and the task 1, 6, 7 and 8b reports with `kx-security-check`'s template; checked with `kx-security-check` in task 21)
- **Status:** Reviewed
- **Scope:** `crates/kx-platform-record`, a library linked only into `keyxtend-worker`, never into the uiAccess app (tidy D4, `banned_media`). It holds the `record` job's recorder, mixer and speed change (`record`, `speed`), the Windows studio over Graphics Capture, `cpal` and Media Foundation (`win/`), the `pick` job's Save and folder dialogs (`pick`) and the encoder table (`record.toml`, read by `table`); off Windows, `NoStudio` answers `NoVideo` and `NoDialogs` shows none (ADR-0056).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None held | A library run by the worker's jobs, which hold no capability. The app checks the gates before any ask: `RecorderKey` needs `ScreenCapture`, `MicPassKey` `Microphone` and `FileAskKey` `SaveFile`, all held by the snip module alone (ADR-0056; ADR-0058). | N/A |
| The screen (record job) | Windows Graphics Capture films the part the ask names, pointer on and border off. | Video region and Video full screen |
| The microphone and the computer's sound (record job) | `cpal` opens the default input, and the default output as a loopback, only while that switch is on. | A video's sound; it records silence |
| The graphics card (record job) | Each frame goes to Media Foundation's encoder through its DXGI device manager, never through the processor (ADR-0056 decision 4). | Smooth video at the full work area |
| The system's file dialogs (pick job) | `IFileSaveDialog` and `IFileOpenDialog` with folder picking, owned by a hidden topmost window of the worker. | Save and Change folder |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Captured frames | Secret (anything on screen) | Graphics card memory in the capture's pool (`record.toml` `buffers`), then the encoder | No | Until encoded |
| A copy's decoded frames | Secret | Processor memory, one frame at a time | No | Until encoded |
| Sound from the microphone and the computer | Secret | A queue per source of at most `queue_ms` (`record.toml`), then the mixer and the encoder | No | Until encoded; older sound is dropped |
| The video or copy being written | Secret | The one path the ask names, in the Snips folder's `unsaved/` part | No | The host's rules (`kx-mod-snip.md`); Cancel, a fault or a broken file deletes it |
| A dialog's answer | Sensitive (a path may name the user) | Memory only; told only after `PickAsk::check` passes it | No | One reply |
| `record.toml` | Public | Compiled in, never read from disk | N/A | The run |

Nothing here logs. A fault leaves only a fixed kind or a system code; a dialog's failure shows its `HRESULT` and its answer's length, never the path (`an_answer_shows_its_paths_length_never_the_path`).

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| A recording's plan, from a checked `RecordAsk` (`kx-ipc.md`) | The part made even for H.264; the start refuses an empty part, a file already at the path and a drive under `min_free_mb` before any device opens (`an_existing_file_is_never_written_over`, `each_start_fault_ends_the_start_and_leaves_no_file`) | `max_min`, every time and length capped at `MAX_VIDEO_MS`; the free-space floor checked each second (`the_floor_reached_mid_run_ends_it_and_keeps_the_video`) | Property tests on the frames; unit tests over the scripted studio (`fake`) |
| A re-timed copy's source | Must be a plain file other than the target, and the target must not exist (`speed.rs`); the host checks it is a listed video of `unsaved/` first. Media Foundation's Source Reader parses it inside the worker | The copy's own length; the host's copy wait | A real copy of a small written video (`win/tests/copy.rs`) |
| Graphics Capture frames | Sizes from the system; pointer on and border off are read back, and a start without them is refused | The pool's buffers; `first_frame_ms` (`record.toml`) for the first frame | Live tests behind `KX_LIVE_SNIP=1` |
| `cpal` sound in any channel count and format | Mixed to stereo by the table's downmix; a count it does not list sends its first channel left, its second right and the rest to both | `queue_ms`; `cushion_ms` and `drift_ms` place each packet; a lost source records silence and is told once (`a_source_lost_mid_run_is_told_and_gives_silence_while_the_run_goes_on`) | Unit tests (`record/tests/sound.rs`, `win/ears/tests.rs`) |
| A dialog's chosen path | Read from the shell item, then `PickAsk::check`: a drive path (on Windows, ADR-0062) of the asked type within `MAX_PATH_BYTES`, no UNC or device path, no part ending in a dot or a space | One dialog at a time | Unit tests in `kx-ipc` and `pick/tests.rs` |
| `record.toml` | Parsed once into checked structs; a table that does not parse refuses every recording and copy, and a build with no `media_dlls` fails | N/A | `table/tests.rs` |

## Threats (STRIDE-lite)
- **Spoofing:** the crate talks only to its job's loop, which reads the worker's own anonymous pipe (`keyxtend-worker.md`). A dialog's answer comes from the shell inside the worker's own process.
- **Tampering:**
  - A video another program of the user swaps into `unsaved/` before a copy meets Media Foundation's parser only inside the restricted worker; the uiAccess app never decodes it (ADR-0056).
  - A file already at a video's or copy's path is never written over (`an_existing_file_is_never_written_over`).
  - `record.toml` is compiled in, so no file on disk changes the encoder's values.
- **Logging / repudiation:** no log line, no `println!` and no stderr. Faults are fixed kinds (`RecordFault`, `PickReply`) with at most a system code.
- **Information disclosure:**
  - No network crate: tidy F5 keeps `kx-platform-net` and the engine out of the worker's `src/record` and `src/pick`, `cpal` out of `src/pick`, and this crate out of `src/voice`, `src/fetch` and `src/net`.
  - A window of ours shielded with `WDA_EXCLUDEFROMCAPTURE` is absent from the video (task 1, 6 live).
  - Sound is opened only while its switch is on; a switched-off source records silence; its device stays open `cushion_ms` (`record.toml`) while the sound before the switch lands, and what it hears after the switch is dropped (`a_switched_off_source_gives_silence_until_switched_on`, `a_sound_left_off_is_never_opened`).
  - Both dialogs set `FOS_DONTADDTORECENT`, `FOS_FORCEFILESYSTEM` and `FOS_NOCHANGEDIR`, so no recent list changes (task 1). Windows still keeps the dialog's size for the worker (see Accepted).
- **Denial of service:**
  - The job object caps the record job at `record_mb` and the pick job at `[snip] pick_mb` (`kx-platform` `snip/pick.rs`); a job past its cap ends as `Gone`, and the host deletes its video.
  - A run that falls behind skips to the current slot, so it never queues frames (`a_run_that_falls_behind_skips_slots_and_keeps_its_sound_whole_and_in_step`).
  - After a stall each sound write holds one source's `queue_ms` at most: a lane drops a gap of a queue or more, and a re-timed copy pads its silence a queue at a time (`a_stall_of_minutes_writes_its_sound_a_queue_at_a_time_and_keeps_it_whole`, `a_gap_far_past_the_queue_never_grows_the_lane_and_its_sound_still_lands`, `a_long_video_without_sound_pads_its_copy_a_queue_at_a_time`).
  - `max_min` and the free-space floor end a run by themselves and keep the video.
  - A dialog may stay open until the host's `[snip] dialog_min`; the job cannot read goodbye meanwhile, so the host relies on the job object to end it.
- **Elevation:**
  - Both jobs run under the worker's restricted token at Medium integrity, with every UI limit kept (`JOBS_UI`, ADR-0056 decision 7).
  - `mfplat.dll`, `mfreadwrite.dll`, `d3d11.dll` and `propsys.dll` are delay-loaded and loaded only from System32 (`LOAD_LIBRARY_SEARCH_SYSTEM32`) when a video job asks (ADR-0063). The worker's `tests/imports.rs` checks its start-up imports name none, so a system without one still runs every other job.
  - Media Foundation starts with `MFSTARTUP_LITE`, which opens no sockets, and an empty media buffer is refused before its lock (`an_empty_buffer_is_refused_before_any_copy`).
  - The crate root has `#![deny(unsafe_code)]`; each Windows call sits in a scoped allow with its `// SAFETY:` comment, over COM objects and buffers the call owns.

## Findings
### Must fix (blocks Definition of Done)
- None open.
### Should fix (tracked issue)
- None open.
### Accepted (with reason)
- **The dialog's remembered size:** Windows writes one value, `ComDlg32\CIDSizeMRU`, for `keyxtend-worker.exe` (task 1). It holds a size, never a path.
- **A mapped network drive:** `PickAsk::check` is lexical, so `Z:\` on a share passes; the user chose that place (task 7 review).
- **Shell extensions in the pick job:** the dialog loads the user's shell extensions into the worker, where they and the dialog's Network and cloud places may reach the network. That is why it runs there, under the restricted token and job, never in the uiAccess app (ADR-0056 decision 6), and an answer off a drive is never told, so the app never sees one.
- **Frames and sound are not wiped:** our own buffers (each source's queue, the mixer's chunk, the frame kept for a skipped slot and a copy's decoded frame) are not wiped, nor are the capture pool's, WASAPI's, `cpal`'s and Media Foundation's. They are reused slot by slot and go as the job ends, and the worker runs as the user.
- **A copy's sound plays faster and higher:** frames are dropped and the sound sped up with them, as fast-forward does; no time-stretch crate is added.
- **No `cargo fuzz` target:** the frames have property tests in `kx-ipc`, and the studio's parsers are the system's.
