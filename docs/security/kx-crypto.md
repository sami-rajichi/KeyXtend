# Security note: kx-crypto

- **Owner:** product owner
- **Last review:** 2026-10-26 (P7 security review: what the tag defeats, rollback and a lost key file); the shelf (P8, 2026-10-31) waits for P8's security check
- **Status:** Reviewed (the shelf: draft)
- **Scope:** `crates/kx-crypto`, a library with no module of its own. The Windows adapter implements its `KeyGuard` with DPAPI (`kx-platform-windows.md`), and the app serves one `SealedFiles` as the `SealedStore` port (`keyxtend.md`). `SealedShelfFiles` backs the `SealedShelf` port (P8, ADR-0034 decision 4), with a `DiskRoom` the platform fills for the drive's free space and name.
- Both stores share one `Sealer`: the key file, the guard, the seal and the file steps, so they cannot drift apart.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None of its own | It seals and opens bytes and reads and writes files under the one root it is given. Modules reach it only through `SealedStore` or `SealedShelf`, which both need `SecretStore`. | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| The data key (`KEY_LEN` random bytes from `getrandom`) | Secret | `sealed.key` in the store's folder (the app's `SEALED_DIR`), wrapped by the OS guard; in memory only as `Zeroizing`, on the heap | Yes, by the guard (DPAPI on Windows) | Until the user deletes the data folder |
| Blobs: learned words, emails (P7) | Sensitive | `<name>.sealed` in the store's folder | XChaCha20-Poly1305, a fresh `NONCE_LEN`-byte nonce per save; the tag covers the magic, the version and the blob's name | Until replaced or deleted |
| Opened bytes | Sensitive | Memory; `open` returns them as `Zeroizing`, but `SealedFiles::load` hands them on as the port's plain `Vec<u8>` | N/A | Handed to the caller (Accepted below) |
| A blob that would not open | Sensitive (still sealed) | `<name>.damaged`, replacing an older one | Yes, as it was | Until the next damaged copy of that name |
| The shelf's data key (P8) | Secret | `sealed.key` in the shelf's root (the clipboard history's folder), one per root, wrapped by the guard | Yes, by the guard | Until the user deletes the root |
| Shelf blobs: clipboard entries and lists (P8) | Sensitive | `<folder>/<name>.sealed` under the root, folders such as `2026-10-31` and `pinned` | As the store's, but the tag covers the folder, `/` and the name | Until `remove` deletes the file, its damaged copy and any temp file, or `drop_folder` deletes the folder |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Sealed files | Size, magic and version byte, then Poly1305 authentication over the magic, version and name as AAD; anything else is refused and set aside | `MAX_BLOB_BYTES` plus the header and tag; never reads more than that and one byte | Property tests (proptest): any bytes, bytes after a good header, a random flipped bit; plus every bit and every truncation of one file; the shelf's `load` over random file bytes. No `cargo fuzz` target (see Accepted) |
| The key file | The guard unwraps it; anything that is not a `KEY_LEN`-byte key is `Locked`, and the file is never deleted or replaced | `MAX_KEY_FILE_BYTES`; a larger file is `Locked` unread | No; tests cover a refused, a short and an oversized key file |
| Blob names | `BlobName` (a-z, 0-9, `-`, `MIN_BLOB_LEN` to `MAX_BLOB_LEN` bytes, and no device name from the port's `RESERVED_NAMES`) before any file is touched; `SealedFiles` and the fake store take only a `BlobName`, so both refuse the same names | `MAX_BLOB_LEN` bytes | Property test of the name form (`kx-module-api`); a test that the fake and the real store keep the same names |
| Shelf folder names and listings (P8) | Folders are `BlobName`s too, so no `..`, separator or drive reaches a path. `folders` and `names` list only folders and `.sealed` files named by a `BlobName`, never following links, and leave anything else out | None (see Accepted) | Tests of stray, upper-case, dotted and over-long entries; a test that the fake and the real shelf agree |
| The drive's free space and name (P8) | From the platform's `DiskRoom`, asked about the nearest existing folder of the root; only shown, never used as a path | N/A | A scripted test drive, failing too |

## Threats (STRIDE-lite)
- **Spoofing:** a blob copied or renamed to another blob's name fails authentication, since the name is in the AAD.
  - On the shelf the AAD is `folder/name`, so a file moved to another folder or name fails too. No `BlobName` holds `/`, so each pair gives other bytes, and a store blob (AAD without `/`) never opens on the shelf or the reverse.
- **Tampering:** any changed bit fails the tag, and the tag covers the magic and version too, so a later format version cannot be swapped in under the same key. The blob is then renamed to `<name>.damaged` and reported once as `Damaged`.
  - The tag defeats other users, other PCs and damage, not programs running as this user: they can unwrap the key (`kx-platform-windows.md`) and seal blobs that pass. An older copy put back also passes (see Accepted).
  - A save writes a temp file in the same folder, syncs it, then renames it over the old file, so a crash never leaves half a file.
- **Logging / repudiation:** the crate never logs. Its errors name only the kind, and `SealedFiles`' `Debug` shows neither the folder (it holds the account name) nor the key.
- **Information disclosure:** a copy of the folder on another PC or account cannot be opened, since only the guard unwraps the key (ADR-0010). The key, the cipher's copy of it and every buffer that holds plain bytes inside the crate are wiped on drop; `load` hands opened bytes to the caller unwiped (Accepted below).
  - Shelf deletes skip the recycle bin, and `remove` also deletes the blob's damaged copy and any temp file, so a deleted entry leaves no sealed copy behind. `SealedShelfFiles`' `Debug` shows neither the root nor the key.
- **Denial of service:** reads are capped by size. A lost key file sets the old blobs aside and the next save makes a new key, dropping the key still held in memory; a key file the guard refuses keeps the store `Locked` rather than losing the data.
- **Elevation:** none; `#![forbid(unsafe_code)]`, and no primitives of our own (RustCrypto's `chacha20poly1305`).

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- None open.
### Accepted (with reason)
- A key file that the guard always refuses, for example one copied from another account, keeps every save `Locked` until the user clears the data folder. Replacing it would destroy data the user may still unlock, which the design forbids.
- The port has no "too large" error, so a save over `MAX_BLOB_BYTES` is `Io`; the learner's caps keep its blobs far below the limit.
- Stack copies of the key may outlive a call (Rust can move values); the key itself lives on the heap and is wiped, which is the `zeroize` crate's documented limit.
- Opened bytes leave `Zeroizing` at the port boundary, since `SealedStore::load` returns a plain `Vec<u8>`. Wiping them is then the caller's job; changing the port's type is a decision for its users.
- Rollback: an older sealed copy put back, by a backup restore or a program running as the user, passes the tag; the format has no version counter.
- A key file deleted while the app runs makes the next save drop the held key and make a new one, so blobs sealed under the old key are set aside when next loaded. It needs a manual or antivirus deletion.
- No `cargo fuzz` target: property tests stand in until the owner allows the nightly toolchain, as in `kx-kernel.md`.
- One `SealedFiles` per folder: the key file is written by a replacing rename, so two stores over one folder could race on the first save and leave blobs under a lost key. The app runs one instance, which owns the folder.
  - The same holds for one `SealedShelfFiles` per root, and a store and a shelf never share a folder (`<data>/words` and the clipboard's own root).
- One `SecretStore` capability gates both ports (ADR-0034), so a module holding it could require either store. Each keeps its own folder and key file, and a gate per store would need a new ADR (Ruling 46).
- A shelf delete is not a secure wipe: the drive may keep the freed blocks. They hold only sealed bytes, which a program running as the user could still open with the key.
- Listings have no count cap: only a program running as the user could fill the shelf's folders, and the history's own caps keep them small.
- The room's `used` counts only `.sealed` blobs, not damaged copies or stray files, so it matches what the history can delete.
