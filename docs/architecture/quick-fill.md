# Quick-fill (P9, ADR-0037, ADR-0038, ADR-0039)

Moved from [ARCHITECTURE.md](../../ARCHITECTURE.md), which keeps the overview, the code map and the invariants.

- **The keyboard stays the only typist.** `kx-mod-vault` holds no `InjectInput`: a click sends `FillType`, and the keyboard types each text part by a letter's route and each Tab as its own key. After a Tab the rest waits `[vault] tab_gap_ms`, but never goes more than `[keyboard] fill_late_ms` past due, or to another app. Nothing reaches the clipboard, and prediction hears one `Break`.
- **Events.**
  - Ungated: `FillOpen` and `FillClose` (keyboard to vault), and `FillAsk` (any module asks the vault to say its state again), as they carry no value.
  - Named `Vault`: `FillLive` (is it running, which lights or dims the fill key) and `FillPanel` (vault to keyboard and ui), `FillView` (vault to ui), `FillKey` (keyboard to vault), `FillLine` (vault to keyboard: the line that shows, ADR-0040), `FillAct` and `FillNames` (ui to vault), `FillType` and `FillTyped` (vault and keyboard), and `VaultEmails` (vault to predict).
  - Paste uses `ClipTake` and `ClipTaken`, which name `ReadClipboard`.
- **Ports.** `Boxes` starts a `BoxRun` on the vault job, and `Places` lists, reads, writes, finds and dates boxes in folders (`Places::modified`). Each call can block, so all run on the module's `kx-vault-box` thread. `Calendar` and `Random` have no gate.
- **Who holds what.** See the list under [Word prediction](prediction.md): `Vault` gates events alone, and `VaultData`, which only the vault holds, gates the store, the boxes and the places (ADR-0039). `ReadClipboard` also goes to the vault, for Paste; it never reads a copy.
- **Files.**
  - `<data>/quickfill/` (`VAULT_DIR`) holds its key file and the sealed blobs `notebook` and `backup`, with their `-aside` copies when one opened but did not read.
  - Each place holds one `KeyXtend Backup.kxbox`: mark, version, the password-locked identity, then one `age` stream. A box another computer wrote, in a place the user picks, asks before any write.
  - The data folder, above or below it, is never a place, as a format would wipe it (`PlaceState::Refused`).
- **The hold.** A notebook that did not read writes no box until Bring back, or Back up now and its Yes, even after a restart, as a hold kept only in memory would lose the box one restart later. Import still adds to it and writes no box.
- **The hidden run (ADR-0037).** A copy started from inside a packaged app, whose writes Windows redirects to that package's private folder, reads and writes nothing in the data folder. The kernel starts fresh with Quick-fill off, the stores are in memory, the log goes to standard error and a banner says so. The next normal run asks about each copy left behind.
- **The worker.** The vault job is the same program as the media job, under its own memory cap (`VAULT_MEMORY_MB`), on the same host, spawner and restricted token ([code map](../../ARCHITECTURE.md)). A crafted file meets only the worker.
