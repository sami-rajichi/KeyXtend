# 34. The clipboard history seals day folders and decodes images in a worker

- **Date:** 2026-10-30
- **Status:** Accepted.

## Context

- P8 adds clipboard history (spec §7, owner 2026-10-30): text with its formatting, images and file references, a paged panel with search, tick-then-Insert, pins, and limits by day, count and size.
- Copies hold secrets as often as typed text does, and an insert puts content into another app, so both need gates like ADR-0033's.
- The sealed store from P7 (`SealedStore`) is flat and caps a blob at 16 MiB; it cannot list, remove or keep folders, which the owner's day-folder queue needs.
- Copied images come from any app, and the uiAccess process must never decode them (ADR-0004). No worker exists yet, and a child of a uiAccess process inherits its token, uiAccess flag included.
- Search must fold like word prediction (§6), whose folding lives in `kx-mod-predict`; a module may depend only on `kx-module-api` (tidy's `module_allowed`).
- The owner wants every feature built once for Windows, macOS and Linux behind neutral ports.

## Decision

Decided with the approval of P8's plan on 2026-10-30.

1. **Two clipboard capabilities.** `ReadClipboard` gates watching the clipboard and every event that carries clipboard content or search text (`ClipKey`, `ClipView`, `ClipAct`). The new `WriteClipboard` gates setting the clipboard and `PasteAsk`. The clipboard module holds both and `SecretStore`; the keyboard holds both as event gates only; the ui module holds `ReadClipboard`.
2. **The keyboard stays the only typist.** The clipboard module sets the clipboard, then asks the keyboard to send the platform's paste shortcut (`PasteAsk`, answered by `Pasted`). The keyboard lets its latches go first, so one plain paste goes out. Separators between texts are pasted text, never typed Enter keys.
3. **Ports.** `Clipboard` (`ReadClipboard`) watches copies and reports skipped, cleared and too-big ones without reading their content, and tells whether referenced files still exist. `ClipboardWriter` (`WriteClipboard`) sets parts with our "leave out of history" marker and pastes files as a copy. `SealedShelf` (`SecretStore`) keeps sealed blobs in named folders, lists, removes and drops them, and reports the room left on its drive. `Pictures` (no gate) turns image bytes into small RGBA pixels. The UI port `Reveal` shows a folder in the system's file manager.
4. **Day folders.** The history lives under `<data>/clipboard/` with one wrapped key: a folder per calendar day (`yyyy-mm-dd`) and one `pinned` folder, each with a sealed list and one blob per entry, split into parts under `MAX_BLOB_BYTES`. The cipher's extra data names the folder and the blob, so a file moved elsewhere will not open. A new day drops the folders past `keep_days` whole; a delete removes the files at once.
5. **The `media` worker.** `keyxtend-worker media`, beside the program, decodes PNG and DIB images and scales them down. The host starts it on demand with uiAccess cleared and normal integrity, inherits only its two pipe handles, and puts it in a job object with a memory cap that ends it with us. It talks `kx-ipc` frames (version, kind, id, capped length), one request at a time within a deadline, and is stopped when idle and restarted with backoff. Replies are checked: the size asked, and width × height × 4 bytes.
6. **Shared folding.** Prediction's `fold`, `casing` and `fold.toml` move to `kx-module-api::text`, so prediction and clipboard search fold with one table.
7. **The history shortcut by meaning.** The shortcuts table names the platform's clipboard-history chord (Win+V on Windows, Super+V on Linux, none on macOS until P15). While the clipboard module wants it (`HistoryWanted`), the keyboard turns that chord into `ClipOpen`.

## Other platforms

- macOS fills `Clipboard` by polling `NSPasteboard`'s change count and reads `org.nspasteboard.ConcealedType` and `TransientType` as markers; `KeyGuard` uses the Keychain; the worker starts as a plain child process.
- Linux fills it with XFixes on X11 and the data-control protocol on Wayland where the compositor has it, with KDE's `x-kde-passwordManagerHint`; Secret Service guards the key.
- `Reveal` opens Finder or the desktop's file manager; the history chord comes from each adapter's table.

## Consequences

- A module without the gates cannot hear copies, read the panel or make the keyboard paste; contract tests check every gated event.
- The worker host is ready for P11's voice, P12's OCR and P14's update worker.
- The uiAccess app still parses no media: tidy bans `png` from its tree.
