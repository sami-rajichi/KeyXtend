# Security note: kx-mod-snip

- **Owner:** product owner
- **Last review:** 2026-11-27 (handfeel task 9 and the final review's fixes: Play opens the re-timed copy through `MediaOpen`, which stays in `unsaved/` and goes with the snip; the frame's reach and Esc bound are tested; no new capability or data class); 2026-11-26 (handfeel task 7: a frame's clicks, the timing line, and no frozen pixels held for a video; no new capability or data class); 2026-11-25 (P12 task 21 `kx-security-check`: the P12 rows checked against the code; each finding fixed or accepted below); 2026-11-25 (P12 task 20: the editor's typed text, and the threats under the template's headings, against the code; checked with `kx-security-check` in task 21); 2026-11-25 (P12 task 17 fix round 1: `Microphone` is this module's alone, ADR-0058); 2026-11-24 (P12 task 15 fix round 1: the `Bare` note); 2026-11-24 (P12 task 15 fix: the ui's `Aside` and `Bare`, `aside_ms` and the view's `dialog`); 2026-11-24 (P12 task 13 fix round 1: copy marks and a run never left recording; the data and threats are checked with `kx-security-check` in task 21)
- **Status:** Reviewed
- **Scope:** `crates/kx-mod-snip`: the manifest, the `[snip]` settings, the capture flow, the editor's model and the actor on its `kx-snip` thread. It runs inside the uiAccess `keyxtend` process; recording, the Save and folder dialogs and PNG files run in `keyxtend-worker` (ADR-0056).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `Snip` | An event gate (ADR-0057 decision 5): the Snip key, its light, the views, their clicks, the keyboard's step aside and the editor's line (`the_events_it_uses_need_the_snip_gate`). | The module cannot show the capture or hear its clicks. |
| `ScreenCapture` | It gates `ScreenFreezeKey` (every screen as raw pixels, capped by `shot_cap_mb`), `RecorderKey` (the worker's `record` job) and `SnipFolderKey` (the Snips folder and its `unsaved` part). | The kernel does not start the module. |
| `Microphone` | It gates `MicPassKey`, the app's pass without which the recorder never hears the microphone (Mic). Only this module holds it, and it never reaches `SpeechKey`, which needs `Speech` (ADR-0058). | The kernel does not start the module. |
| `WriteClipboard` | It gates `ClipboardWriterKey`: Copy, written with `write_kept`, so KeyXtend's history keeps the snip and Windows' history keeps a picture, never a video's file (ADR-0056 decision 2, as built). Only this module calls `write_kept` (`only_snip_writes_to_the_clipboard_without_our_markers`). | The kernel does not start the module. |
| `OpenFolder` | It gates `FolderRevealKey` (Open folder) and `MediaOpenKey` (Play, an MP4 of the Snips folder only, through the running Explorer). The voice module holds it too but never asks for `MediaOpen`; the app's tests name both holders (ADR-0058 decision 3). | The kernel does not start the module. |
| `SaveFile` | It gates `FileAskKey`, the system's Save and folder dialogs in the worker's `pick` job; only this module holds it. | The kernel does not start the module. |

- Each capability gates a service the manifest requires or the module's events, and none other is declared (`each_capability_gates_a_service_it_needs_or_its_events`).
- It holds no `InjectInput`, `Network`, `ReadClipboard`, `ReadFocusedText` or `SecretStore` (`it_holds_the_plans_capabilities_and_never_injects_input`).
- It provides no service. It takes each service it needs through its gate at start, and fails without one (`a_missing_service_fails_the_start`); `Pictures` and `Calendar` need no gate.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Screen pixels and pictures | Secret (anything on screen) | Memory (`Shot`, whose `Debug` shows only the size), the clipboard on Copy, and a file the user chose on Save. Only a picture's strip holds a cut (`Held::Picture`); a video's countdown, recording and strip hold no frozen pixels, as its poster is gone | No (Accepted) | Until the snip ends; a copy as long as the clipboard history keeps it |
| Recordings | Secret (anything on screen, and sound) | The Snips folder's `unsaved` part while recorded, a folder for the user and the system only, then a file the user chose | No (Accepted) | Don't save, Cancel and Close delete it, as does quitting with a Copy that never reached the clipboard, unless an earlier Copy put that video there, and a run the actor drops or never hears is cancelled and deleted (`src/actor/tests/quit.rs`). A copied one stays `copied_days` from its copy mark, and the sweep at each start and each new snip deletes every other PNG and MP4 there and retries a refused delete (`src/actor/tests/sweep.rs`) |
| Copy marks | Public (an empty file named after its video) | Beside the copied video in `unsaved/` | No | Deleted with its video, or by a sweep once the video is gone |
| Sound from the microphone and the computer | Secret | Only inside a recording, and only while its switch is on | No | As the recording |
| Text typed into the editor's Text drawings (`SnipLineKey` letters) | Sensitive (typed text) | Memory only, as `Redacted<String>` in the editor's model (`edit/text.rs`) and the view, whose `Debug` never shows it (`typed_text_never_prints`); it reaches pixels only in a flattened picture the user copies or saves | No | Until its drawing is erased or undone, or the snip ends |
| The Save and folder dialogs' answers | Sensitive (a path may name the user) | Memory only; checked as a drive path of the asked type, outside `unsaved/`, before anything is written | No | Until used; `save_dir` keeps a picked folder |
| `[snip]` settings, `save_dir` included | Sensitive (`save_dir` may name the user) | `settings.toml` (only the user's changes) | No | Until the user changes them |

Nothing here is ever logged: no pixel, picture, recording, sound, typed text or path. The actor's warnings carry a fixed line and an error kind only, and its timing line at debug carries milliseconds only.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| The `[snip]` section | Read with `deny_unknown_fields`; `keyboard` takes only its three names, `fps` only `RecordAsk::FPS`, each range is the port's own or the settings' (`src/settings.rs`, `src/settings/jobs.rs`), `editor` only an empty table or a whole rectangle, `editor_full` only as `editor` and only with an `editor`, and `save_dir` is kept as typed and checked when used. A bad value goes back to its default with the settings notice (`tests/snip/settings.rs`) | The kernel's settings file limits; the frames refuse the same ranges again | Property tests over every range and frame rate (`src/settings/tests.rs`) |
| The ui's `Aside` and `Bare` clicks, through the `Snip` gate | `Aside` counts only for the round told with the step aside or a newer one, each freeze is numbered so an older snip's screens are dropped, and a late or second one does nothing (`src/actor/tests/aside.rs`). `Bare` acts on any round: it dims video until the module stops, ends a countdown and stops a recording as Stop does, with the `Bare` note | The freeze goes anyway at `aside_ms` | Property tests over the flow's steps (`src/flow/tests/props.rs`) |
| The hand's Esc (`SnipClick::Esc`, through the `Snip` gate, ADR-0059) | Acts on any round, but only while the screens freeze or show frozen and no system dialog sits in front, then as our keyboard's Esc does, Don't save at the close question there, so a second Esc always ends the snip, or a third with a frame's handle held, as the first drops the hold; at rest, counting down, recording or editing it does nothing (`src/actor/tests/esc.rs`, `src/flow/tests/close.rs`) | One click per press, which the ui sends only while frozen screens show | No |
| The ui's points, strokes, fits, screens, editor places and flattened pictures (`SnipAct`, through the `Snip` gate) | Each through the port's checked types; a key click names its view's round and an older one is dropped; a flattened picture is taken only for the Copy or Save that asked, cut by the ui to the kept part. A frame's clicks are pointer positions and a handle's name only, and the frame rules keep each set within the box round all frozen screens, a video region's one screen, or the picture (`src/frame/tests/props.rs`) | One picture, bounded by the freeze's `shot_cap_mb`; a pen's path keeps its first `stroke_max` points (`src/actor/tests/editor.rs`) | Property tests (`src/flow/tests/props.rs`, `src/edit/tests/props.rs`) |
| The keyboard's letters for a Text drawing (`SnipLineKey`) | Taken only while a Text drawing is open and a Snip window holds the aim after the user's press (ADR-0057); kept as `Redacted<String>` | One key at a time | `src/edit/tests/text.rs` |
| The recorder's notes and the dialogs' answers, through the ports | Already checked by `kx-ipc` and the hosts; a Save answer arrives as a `SaveTo` checked twice with `PickAsk::check`, and one in `unsaved/` shows the `PlaceRefused` note | The hosts' waits | `src/actor/tests/record.rs`, `src/actor/tests/save.rs` |
| The `unsaved/` listing (`SnipFolder::unsaved`) | Plain PNG and MP4 files only, never a link or a folder; deletes go through the folder port, which acts only directly in `unsaved/`, and a file a job still uses waits for a later sweep | One folder read a sweep | `src/actor/tests/sweep.rs` |

## Threats (STRIDE-lite)
- **Spoofing:**
  - Every snip event needs the `Snip` gate, held only by this module, the keyboard and the ui (ADR-0057 decision 5), and the ui sends through a handle limited to its grants (`kx-ui.md`).
  - A key click names its view's round, so one on an older view is dropped. The ui's own answers (`Fit`, `Screens`, `Bare`, `Placed`) the tray's `Open` and the hand's `Esc` act on any round, and `Aside` only on the round told with its step aside or a newer one (`actor/clicks.rs`).
  - The recorder opens only the passes of the app's one `MicSeal`, checked before the platform hears them (`keyxtend.md`).
- **Tampering:**
  - A dialog's answer is checked as a drive path of the asked type in the worker and again in the host (`PickAsk::check`) before anything is written. A Save writes a new `.kxpart` file and renames it over the target only when whole.
  - Deletes, marks and the sweep act only inside `unsaved/`, through the `SnipFolder` port. A copy mark is removed, then made new (`create_new`), so a link planted at its name is never followed.
  - A bad `[snip]` value goes back to its default with the settings notice (Inputs).
- **Logging / repudiation:** no pixel, picture, sound, typed text or path is logged; warnings carry a fixed line and an error kind only, and the timing line milliseconds only. `Shot`'s `Debug` shows only its size, and typed text is `Redacted`.
- **Information disclosure:**
  - Nothing is kept unless the user copies or saves it; the flow's property test checks every path (`no_path_keeps_an_unchosen_file_or_leaves_the_keyboard_aside`). Don't save, Cancel and Close delete a recording, and the sweep deletes every PNG and MP4 of `unsaved/` but a copied video within `copied_days`, and neither the Save dialog nor Change may choose that part.
  - Snip's six capture windows shield themselves, and a capture window the ui could not shield is told as `Bare`, so no video runs with a hidden record bar. The overlay, the editor and the close question are not shielded, as none shows while a freeze or a recording runs; a snip started from the editor or its question freezes only once the ui says they are gone, or at `aside_ms`.
  - With no countdown, the worker's start, hundreds of milliseconds, is the only gap between the overlay's hide and the first frame.
  - Unless `keyboard` is `shown`, the keyboard steps aside and every other window of ours is shielded; in Shown it stays (`SnipAside::Stay` from the editor or its question). The freeze waits for the ui's `Aside`, at most `aside_ms` (Accepted).
  - A copy reaches KeyXtend's history, and a picture Windows' history too (`write_kept`), only on the user's Copy, as the owner chose (ADR-0056 decision 2). A copied picture also reaches Windows' cloud clipboard when the user turned its sync on, as any copy does (ADR-0056, Consequences).
  - Sound is heard only while its switch is on: the recorder never hears the microphone without the app's pass.
  - The module holds no `Network`, and the worker's record and pick jobs hold no network crate (`keyxtend-worker.md`); the Save dialog's own shell may reach the network when the user browses there (`kx-platform-record.md`, Accepted).
- **Denial of service:**
  - A freeze past `shot_cap_mb` is refused before any pixel is grabbed. An `Encode`'s memory cap and deadline grow with its pixels, and a recording's job is capped at `record_mb`.
  - A recording stops by itself at `max_min` or when its drive reaches `min_free_mb`, keeping what it recorded. Copy tries a busy clipboard at most `copy_again` more times, and a pen's path keeps at most `stroke_max` points.
  - Each blocking call runs on a short job thread, so the `kx-snip` actor never waits on the worker or a dialog (`a_slow_freeze_never_holds_the_actor_and_a_late_one_is_dropped`), and a job that dies still answers with a failure (`a_job_that_dies_before_it_answers_still_answers`). The freeze waits for the keyboard at most `aside_ms`, and Undo keeps at most `undo_max` steps.
- **Elevation:**
  - The module holds no `InjectInput`: typed text goes only into its own picture, never to an app.
  - The uiAccess process decodes no file. PNG files, recording, codecs and the Save and folder dialogs run in the worker (ADR-0056), and Play opens only an MP4 of the Snips folder through the running Explorer.

## Findings
### Must fix (blocks Definition of Done)
- None open.
### Should fix (tracked issue)
- None open.
### Accepted (with reason)
- `SnipSettings`'s `Debug` shows `save_dir`, as `[voice] models_dir`'s does: serde needs the plain text, and the module never prints or logs its settings.
- **Recordings kept unencrypted in `unsaved/` (ADR-0010):** Media Foundation writes the MP4 straight to disk in the worker, and a copied video must stay a plain file for the paste and the player, so it cannot be sealed. The Snips folder is made with rights for the user and the system only; Cancel, Close and Don't save delete a recording at once, every start and every snip sweeps the other leftovers, and a copied one goes once its mark is `copied_days` old.
- **Pixels and sound are not wiped:** a snip's pixels pass through GDI, Qt's textures, the clipboard and the media worker's pipe, and its sound through WASAPI and Media Foundation, none of which we can wipe, so our own copies (`Shot`, as `ClipPart::Image`'s pixels) are not wiped either. Each goes when its snip ends, and the user can read their own screen and memory anyway.
- **The freeze waits at most `aside_ms` for the keyboard:** if the ui has not answered by then, the screens freeze anyway, so a stalled Qt thread could leave the keyboard or a panel of ours in that picture. It is the user's own screen, kept only if they copy or save it, and the ui answers within a frame when it runs.
- **A pasted video's name:** a recording in `unsaved/` is named `<unix seconds>.mp4`, counted past every name held, and a pasted copy carries that name, so whoever gets it learns the second it started; the Save dialog proposes the local time too, and no name holds the account or a window's title.
- **Traces outside the folder:** Windows may keep a thumbnail of a video pasted into Explorer or played, and the user's player its recent list, after the sweep deletes the video; those are Windows' and the player's own stores, which the user controls.
