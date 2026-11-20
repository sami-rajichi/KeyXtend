# Security note: kx-mod-clipboard

- **Owner:** product owner
- **Last review:** 2026-11-20 (P11 part 2 task 22: Paste at once's skipped-copy rules made plain against `actor/board.rs`, the start-up gap accepted, and ADR-0048's line qualified); 2026-11-06 (P9 Task 27: the search line's text reaches prediction's memory while aimed at); 2026-10-31 (P8 security check: the panel's gate, the insert's file-check deadline and the paths never probed; P8 Task 14: the module, the actor, insert and the helper threads; fix round 1: joins, insert notes, the helper queues; final fix: the DIB an inserted PNG goes with)
- **Status:** Reviewed
- **Scope:** `crates/kx-mod-clipboard`: the manifest, the `[clipboard]` settings, the `kx-clip` actor (`actor.rs`, `actor/`), insert, the media and files threads (`media.rs`), the store, the panel and the view. It runs inside the uiAccess `keyxtend` process; pictures decode in `keyxtend-worker` (ADR-0004).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `ReadClipboard` | It gates `ClipboardKey` (the watch, the file checks and the text read for Quick-fill's take), the panel events `ClipPanel` (ADR-0035), `ClipKey`, `ClipLine`, `ClipView` and `ClipAct`, and the take's `ClipTake` and `ClipTaken`. | The module does not start (`each_missing_gate_fails_the_start_and_watches_nothing`). |
| `WriteClipboard` | It gates `ClipboardWriterKey`, which sets the clipboard for each insert step and each lent paste, and `PasteAsk`, `Pasted`, `PasteText` and `TextPasted`. | The module does not start (same test). |
| `SecretStore` | It gates `SealedShelfKey`, which keeps the history sealed (ADR-0034). | The module does not start (same test). |

- It has no `InjectInput`: each insert step asks the keyboard with one `PasteAsk`, and the keyboard sends the paste (ADR-0033).
- `PicturesKey` and `CalendarKey` are ungated; `ClipOpen`, `HistoryWanted`, `HistoryAsk` and `Notice` are ungated too, as they turn no key and carry nothing.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Copied text, links, pictures and file paths | Sensitive, and Secret when the user copies a password | Sealed blobs on the `SealedShelf` (kx-crypto); memory in `Redacted` | Yes on disk | `keep_days` (media `media_days`), `max_entries`, `max_mb`; pinned until unpinned |
| Thumbnails | Sensitive | Sealed blobs beside their entry | Yes | As their entry |
| Previews and the shown page | Sensitive | Memory only, in `Redacted` | No | Until the panel changes or closes |
| Picture jobs for the worker | Sensitive | Memory only: one thumbnail drawing, one preview drawing and the newest waiting, and an insert's PNG for its DIB, each at most `max_entry_mb` | No | Until drawn or made |
| Each inserted step | Sensitive | The system clipboard, readable by any app | No | Until the next copy; the last entry stays there |
| A lent paste's text (Paste at once, P11) and what it puts back | Sensitive | The system clipboard, marked ours so neither history keeps it; memory in `Redacted`, wiped after each write | No | The text until `paste_gap_ms` after the paste; then what the clipboard held comes back, unless another app changed it meanwhile |
| The folder lists (titles, search text, app names, sizes) | Sensitive | Sealed list blob per folder | Yes | As their entries |
| The search line as told (`ClipLine`, ADR-0040) | Sensitive | One event to modules granted `ReadClipboard`, its text up to the caret, as `Redacted<String>`; the actor keeps the last told. While it is aimed at, the keyboard passes its text to prediction's memory (`AimedLine`), for word chips, and it is never stored, learned or logged there (`kx-mod-predict.md`) | No | Until the search changes or the panel closes |
| `[clipboard]` settings, including `skip_apps` | Public | `settings.toml` | No | Until the user changes them |

A file entry keeps paths only, never the files. Copies an app marks as private (`ClipNote::Marked`), and copies from `skip_apps`, are never kept. `Msg`, entries and pages print no content in debug output.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Watch notes (`ClipNote`) | `keep_parts` keeps only known parts; a copy with none makes no row; notes after the stop are dropped | The adapter refuses copies past `max_entry_mb`, which show a note | No; `actor/tests/copies.rs` |
| File facts (`Clipboard::files`) | Asked on `kx-clip-files`, never on the actor, for every file copy and panel open (`look_rows`) and at Insert, whose check comes before the newest row look (older row looks dropped). Only an absolute drive path (`C:\…`) is probed, through `probe_ok`; every other form, including `\\host\`, `\\?\`, `\??\` and relative paths, is `FileFact::Unchecked`, and so is a path through a symlink or junction, never followed. `Unchecked` shows "Size unknown" and pastes at Insert; `Away` keeps the row and skips it; all `Gone` deletes the entry; a files thread gone stops Insert with a note | Insert's check stops after `check_ms` with `PasteFailed`, and its late answer is dropped; a slow drive holds only that thread, which is never joined | No; `actor/tests/panel.rs`, `paste.rs`, `stops.rs`, `media/tests.rs` |
| Panel events (`ClipKey`, `ClipAct`, gated) | A tick on an old round does nothing; rows and pages are clamped to the shown page; a key whose question or preview went is swallowed | One message each | Property tests in `panel/props.rs` |
| `Pasted` (gated) | Only the round asked is taken; a failed or missing answer stops the insert | `paste_wait_ms` | No; `actor/tests/paste.rs` |
| `PasteText` (gated, from voice) | Refused, with nothing written, while an insert or a paste runs, past the asker's `start_ms`, after a `skip_apps` copy or a change the watch could not read, when a presence marker is on the clipboard (checked live, in the same open as the read, by `Clipboard::unmarked_text`), when the read fails or passes `back_chars`, when the kept entry there cannot be read whole, or when no text is there and the history cannot name what is (`board.rs`); answered once with `TextPasted` | `start_ms`, `paste_wait_ms`, `open_ms`, `back_chars` | Property test in `actor/tests/lend/props.rs`; `lend.rs`, `lend/back.rs`, `lend/start.rs`, `lend/skip.rs` |
| `ClipTake` (gated, from Quick-fill or the voice key page) | The read stops at the take's cap, at most twice the longest field (`ClipTake::cap`), and the text goes back only as a wiped `Secret`, in an answer that names the take's asker (`ClipTake::answer`), so only the module that asked takes it (`an_answer_names_the_module_that_asked_so_only_it_takes_the_text`). The newest entry is forgotten at once, before the vault accepts the text, and only when it is an unpinned text of this copy; a text too long, a busy clipboard or no text forgets nothing, and with no entry to forget the text waits as a wiped `Secret` until the next copy's note, which keeps nothing if it is the same text, as that note can come after the take | The cap; `open_ms` | No; `actor/tests/take.rs` |
| Pictures from the worker | Drawn in the worker within `max_pixels`, `media_ms` and `worker_mb`; a failure on the image leaves the type icon, and a row refused while the worker is down is asked again once it may start | Thumbnail and preview sides from settings | No; `actor/tests/pictures.rs` |
| DIBs from the worker (`Made`) | Only the answer to the DIB ask waited for is taken, within `media_ms`; a DIB past `max_entry_mb`, a failure or no answer sets the PNG alone, with one warning naming only the reason. The bytes are never parsed in this process: the writer sets them beside the PNG | `media_ms`, `max_entry_mb` | No; `actor/tests/dib.rs`, `actor/tests/props.rs` |
| Sealed lists and blobs | AEAD first, then the list's version and every length; a damaged list is set aside with `notice.clip-damaged`; a newer version fails the start untouched | The shelf's `MAX_BLOB_BYTES` | Property tests in `store/` |
| `[clipboard]` settings | `deny_unknown_fields`, each value within its named range, pairs ordered; a bad value resets alone | Checked at load | No; `settings/tests.rs`, `tests/clip/settings.rs` |

## Threats (STRIDE-lite)
- **Spoofing:** only modules holding `ReadClipboard` can drive the panel or turn the keyboard's keys into panel keys (`ClipPanel`), and only those holding `WriteClipboard` can answer a paste; a forged `ClipOpen` only toggles the panel.
- **Tampering:** a blob changed outside fails its AEAD; a damaged list is set aside once and copies go on. Programs running as this user can unwrap the key, so the decoders treat every blob as untrusted.
- **Logging / repudiation:** log lines hold fixed texts and error kinds only, never copied content, paths or app names.
- **Information disclosure:**
  - Content rests sealed. A `Locked` shelf fails the start, which the kernel tells with `notice.module-failed` and Try again, so nothing is kept unsealed; a lock found while running says `notice.clip-locked` once.
  - Each insert step puts its content on the system clipboard, where any app can read it, and the last entry stays there.
  - Each paste goes to whichever app has focus at that step; the keyboard sends it, never this module.
  - A lent paste puts back a kept entry whole, or a text the history cannot name alone, and types when a kept entry cannot be read whole. A cut file comes back as a copy, and a failed put-back is logged and leaves the text.
  - A lent paste never puts back a skipped app's copy: after a `skip_apps` note, or a change the watch could not read, it types until another app's copy, a clear or our own write. Otherwise a password manager's clear timer could fire while our text is on the clipboard and skip its clear, as the password is gone; our put-back would then leave the password on the clipboard with no clear to come.
  - Our own write releases that hold: each Insert step, like a lent paste or a put-back, writes the clipboard as ours (`Board::wrote`), so the skipped copy is no longer there to put back.
  - A lent paste reads the clipboard only through `Clipboard::unmarked_text`, which checks the presence markers in the same open: a copy a password manager marks is never read or written back, and the text is typed instead (ADR-0048). Our writes set only the history and cloud switches to 0, so an unmarked text written back is as private as it was. A copy made during the gap stays.
  - Follow-the-clear drops what an app clears within `clear_follow_s`, unless pinned.
  - Copied file paths are looked at before the user acts, on each file copy and panel open. So only absolute drive paths are probed: a copied share, device or NT path, or a link to one, never makes our process reach another machine.
- **Denial of service:** entries, bytes and days are capped, and the free-space floor refuses copies; at most `queue_copies` copies wait for the actor, and a flood past it is dropped with one warning; the file checks run off the actor; pictures decode in the worker with its own limits, and the actor hands it at most three image copies at once, plus one PNG while an insert waits for its DIB; saves wait `save_ms`.
- **Elevation:** the crate forbids `unsafe`. A panic in the actor closes the panel, frees the chord and shows `notice.module-failed`.

## Findings
### Must fix (blocks Definition of Done)
- None open.
### Should fix (tracked issue)
- None open.
### Accepted (with reason)
- The watch thread can be held by an app that renders its data late; its drop is bounded, and a late note after the stop is dropped.
- Content in memory is not wiped after use; the process runs as the user, who can read it anyway. An image's bytes move from the store to the worker's frame uncopied, so a draw holds at most the read and its frame; an insert's DIB ask copies its PNG, which the step also writes.
- The actor's inbox has no bound; its senders are this module's own threads and gated events.
- One `SecretStore` gates both this shelf and predict's word store, so either module could require the other's service. Each store keeps its own folder and key file, and a gate per store would need a new ADR (builder's call).
- The keyboard and the ui may reach the clipboard services their gates allow, not only the events (ADR-0035 decision 2). Neither asks for one, and the keyboard already sends input.
- The module cannot tell a local path from a remote one: `probe_ok` and the platform's file facts decide what is looked at. A drive letter mapped to a share is still probed (builder's call; `kx-platform-windows.md`).
- The board counts our writes' echoes only to put an entry back whole, as a `Marked` note does not say whose mark it is. A stale count can put back a stale entry's other formats with the clipboard's text, or write "empty" over a picture whose note it swallowed: the current clipboard is lost, but the live check, not the board, keeps private copies from being read.
- A note that names no app is never taken as a `skip_apps` app's, as the history keeps such a copy too, so a skipped app the watch cannot name can still have its copy put back.
- A skipped app's copy already on the clipboard when the keyboard starts can be put back: the history does no first read at start-up, so the board knows nothing of it until the next note (P11 part 2). ADR-0048's consequences say Paste at once types while a skipped app's copy is on the clipboard; that holds from the first note the watch tells, not for a copy made before the start.
