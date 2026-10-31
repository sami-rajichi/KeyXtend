# kx-mod-clipboard

Clipboard history (spec §7, ADR-0034): the module that keeps what the user copies, sealed on disk, and the panel that pastes it back through the keyboard (ADR-0033). The parts below it are pure and tested alone.

- **Entries** (`entry`, `hash`, `order`, `find`): an entry per copy, its kind, title and search text; the history in pinned-then-newest order; folded search.
- **Limits** (`keep`, `follow`): days, entry and byte caps, the free-space floor, and follow-the-clear, which drops what an app clears unless pinned.
- **Store** (`store`): day folders and a pinned folder on the `SealedShelf`, each with a versioned list; blobs, thumbnails, moves and a damaged list set aside.
- **Panel** (`panel`, `view`): keys and clicks to `Do`, ticks in order, paged rows, the preview and the Delete-all question; `view` builds the `ClipPage` the ui draws.
- **Insert** (`insert`): the ticked entries in tick order, a join between texts pasted as text (never Enter), files still there, and those out of reach skipped.
- **Media** (`media`): thumbnails and previews drawn by the `Pictures` worker on `kx-clip-media`; file checks on `kx-clip-files`, as a share out of reach can block.

## The module

`ClipboardModule` loads the history, watches the clipboard and runs the `kx-clip` actor. A locked shelf, or a list from a newer version, fails the start with nothing changed; the kernel tells it and offers Try again. The actor:

- keeps each copy within the limits, moves a twin up, and shows a note for one too big, out of disk room, or past a full pinned list;
- toggles the panel on `ClipOpen`, waits for the ui's `ClipAct::Rows`, then draws one `ClipView` per change;
- inserts with one `PasteAsk` per step, waits for `Pasted`, then `paste_gap_ms`, and writes a join only once the entry after it is read; a failed write, a paste that never comes or nothing left to paste stops with a note, and the panel closes when all went in;
- saves `save_ms` after a change, sweeps at local midnight, and on stop closes the panel and frees the chord (`HistoryWanted { on: false }`).

## Services and capabilities

| Service | Use |
|---|---|
| `Clipboard` | Watches copies and checks the files of file entries. |
| `ClipboardWriter` | Sets the clipboard for each insert step. |
| `SealedShelf` | Keeps the history sealed, and tells the drive's free space. |
| `Pictures` | Draws thumbnails and previews in the worker. |
| `Calendar` | The local day, for day folders and the midnight sweep. |

| Capability | Why |
|---|---|
| `ReadClipboard` | It gates `Clipboard` and the panel events (`ClipKey`, `ClipView`, `ClipAct`). |
| `WriteClipboard` | It gates `ClipboardWriter`, `PasteAsk` and `Pasted`. |
| `SecretStore` | It gates the `SealedShelf`. |

It provides no service.

## Settings (`[clipboard]`, version 1)

The defaults live only in `defaults.toml`, each with its one-line meaning; the ranges are named constants in `src/settings.rs`.

| Group | Keys |
|---|---|
| Limits | `keep_days`, `media_days`, `max_entries`, `max_mb`, `min_free_mb`, `max_entry_mb` |
| Follow and skip | `clear_follow_s`, `skip_apps` |
| Insert | `join_breaks`, `join_rule`, `paste_gap_ms`, `paste_wait_ms`, `history_key` |
| Text | `search_chars`, `title_chars`, `preview_chars` |
| Pictures | `thumb_px`, `preview_px`, `max_pixels`, `media_ms`, `worker_idle_s`, `worker_mb`, `restart_ms`, `restart_max_ms` |
| Platform | `open_ms`, `poll_ms`, `save_ms` |

## Public API

`ClipboardModule`, `ID`, `MANIFEST`, `ClipSettings` (with `from_defaults` and each part's config) and its `SPEC`; the parts above are public for tests and tools.

## Tests

`cargo test -p kx-mod-clipboard`: unit tests drive the actor without a thread on the fakes, and `tests/clip` runs the module on the fake host.

## Security

See `docs/security/kx-mod-clipboard.md`.
