# Security note: kx-mod-clipboard

- **Owner:** product owner
- **Last review:** 2026-10-31 (P8 Task 14: the module, the actor, insert and the helper threads; fix round 1: joins, insert notes, the helper queues)
- **Status:** Draft
- **Scope:** `crates/kx-mod-clipboard`: the manifest, the `[clipboard]` settings, the `kx-clip` actor (`actor.rs`, `actor/`), insert, the media and files threads (`media.rs`), the store, the panel and the view. It runs inside the uiAccess `keyxtend` process; pictures decode in `keyxtend-worker` (ADR-0004).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `ReadClipboard` | It gates `ClipboardKey` (the watch and the file checks) and the panel events `ClipKey`, `ClipView` and `ClipAct`. | The module does not start (`each_missing_gate_fails_the_start_and_watches_nothing`). |
| `WriteClipboard` | It gates `ClipboardWriterKey`, which sets the clipboard for each insert step, and `PasteAsk` and `Pasted`. | The module does not start (same test). |
| `SecretStore` | It gates `SealedShelfKey`, which keeps the history sealed (ADR-0034). | The module does not start (same test). |

- It has no `InjectInput`: each insert step asks the keyboard with one `PasteAsk`, and the keyboard sends the paste (ADR-0033).
- `PicturesKey` and `CalendarKey` are ungated; `ClipOpen`, `ClipPanel`, `HistoryWanted`, `HistoryAsk` and `Notice` are ungated too.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Copied text, links, pictures and file paths | Sensitive, and Secret when the user copies a password | Sealed blobs on the `SealedShelf` (kx-crypto); memory in `Redacted` | Yes on disk | `keep_days` (media `media_days`), `max_entries`, `max_mb`; pinned until unpinned |
| Thumbnails | Sensitive | Sealed blobs beside their entry | Yes | As their entry |
| Previews and the shown page | Sensitive | Memory only, in `Redacted` | No | Until the panel changes or closes |
| Picture jobs for the worker | Sensitive | Memory only: one thumbnail drawing, one preview drawing and the newest waiting, each at most `max_entry_mb` | No | Until drawn |
| Each inserted step | Sensitive | The system clipboard, readable by any app | No | Until the next copy; the last entry stays there |
| The folder lists (titles, search text, app names, sizes) | Sensitive | Sealed list blob per folder | Yes | As their entries |
| `[clipboard]` settings, including `skip_apps` | Public | `settings.toml` | No | Until the user changes them |

A file entry keeps paths only, never the files. Copies an app marks as private (`ClipNote::Marked`), and copies from `skip_apps`, are never kept. `Msg`, entries and pages print no content in debug output.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Watch notes (`ClipNote`) | `keep_parts` keeps only known parts; a copy with none makes no row; notes after the stop are dropped | The adapter refuses copies past `max_entry_mb`, which show a note | No; `actor/tests/copies.rs` |
| File facts (`Clipboard::files`) | Asked on `kx-clip-files`, never on the actor, insert checks before the newest row look (older row looks dropped); `Away` keeps the row and skips it at insert, all `Gone` deletes the entry; a files thread gone stops Insert with a note | A share out of reach holds only that thread, which is never joined | No; `actor/tests/panel.rs`, `paste.rs`, `stops.rs`, `media/tests.rs` |
| Panel events (`ClipKey`, `ClipAct`, gated) | A tick on an old round does nothing; rows and pages are clamped to the shown page; a key whose question or preview went is swallowed | One message each | Property tests in `panel/props.rs` |
| `Pasted` (gated) | Only the round asked is taken; a failed or missing answer stops the insert | `paste_wait_ms` | No; `actor/tests/paste.rs` |
| Pictures from the worker | Drawn in the worker within `max_pixels`, `media_ms` and `worker_mb`; a failure leaves the type icon | Thumbnail and preview sides from settings | No; `actor/tests/pictures.rs` |
| Sealed lists and blobs | AEAD first, then the list's version and every length; a damaged list is set aside with `notice.clip-damaged`; a newer version fails the start untouched | The shelf's `MAX_BLOB_BYTES` | Property tests in `store/` |
| `[clipboard]` settings | `deny_unknown_fields`, each value within its named range, pairs ordered; a bad value resets alone | Checked at load | No; `settings/tests.rs`, `tests/clip/settings.rs` |

## Threats (STRIDE-lite)
- **Spoofing:** only modules holding `ReadClipboard` can drive the panel, and only those holding `WriteClipboard` can answer a paste; a forged `ClipOpen` only toggles the panel.
- **Tampering:** a blob changed outside fails its AEAD; a damaged list is set aside once and copies go on. Programs running as this user can unwrap the key, so the decoders treat every blob as untrusted.
- **Logging / repudiation:** log lines hold fixed texts and error kinds only, never copied content, paths or app names.
- **Information disclosure:**
  - Content rests sealed. A `Locked` shelf fails the start, which the kernel tells with `notice.module-failed` and Try again (Ruling 40), so nothing is kept unsealed; a lock found while running says `notice.clip-locked` once.
  - Each insert step puts its content on the system clipboard, where any app can read it, and the last entry stays there.
  - Each paste goes to whichever app has focus at that step; the keyboard sends it, never this module.
  - Follow-the-clear drops what an app clears within `clear_follow_s`, unless pinned.
- **Denial of service:** entries, bytes and days are capped, and the free-space floor refuses copies; the file checks run off the actor; pictures decode in the worker with its own limits, and the actor hands it at most three image copies at once; saves wait `save_ms`.
- **Elevation:** the crate forbids `unsafe`. A panic in the actor closes the panel, frees the chord and shows `notice.module-failed`.

## Findings
### Must fix (blocks Definition of Done)
- None open.
### Should fix (tracked issue)
- None open.
### Accepted (with reason)
- The watch thread can be held by an app that renders its data late; its drop is bounded, and a late note after the stop is dropped.
- Content in memory is not wiped after use; the process runs as the user, who can read it anyway.
- The actor's inbox has no bound; its senders are this module's own threads and gated events.
