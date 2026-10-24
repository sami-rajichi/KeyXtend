# Security note: kx-crypto

- **Owner:** product owner
- **Last review:** 2026-10-24 (P7 Task 5: the crate is built)
- **Status:** Draft
- **Scope:** `crates/kx-crypto`, a library with no module of its own. The Windows adapter implements its `KeyGuard` with DPAPI (P7 Task 6a), and the app serves `SealedFiles` as the `SealedStore` port (Task 12).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None of its own | It seals and opens bytes and reads and writes files in the one folder it is given. Modules reach it only through `SealedStore`, which needs `SecretStore`. | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| The data key (32 random bytes from `getrandom`) | Secret | `sealed.key` in the data folder, wrapped by the OS guard; in memory only as `Zeroizing`, on the heap | Yes, by the guard (DPAPI on Windows) | Until the user deletes the data folder |
| Blobs: learned words, emails (P7) | Sensitive | `<name>.sealed` in the data folder | XChaCha20-Poly1305, a fresh 24-byte nonce per save, the blob's name as AAD | Until replaced or deleted |
| Opened bytes | Sensitive | Memory; `open` returns them as `Zeroizing` | N/A | Handed to the caller |
| A blob that would not open | Sensitive (still sealed) | `<name>.damaged`, replacing an older one | Yes, as it was | Until the next damaged copy of that name |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Sealed files | Size, magic and version byte, then Poly1305 authentication with the name as AAD; anything else is refused and set aside | `MAX_BLOB_BYTES` (16 MiB) plus the header and tag; never reads more than that and one byte | Property tests (proptest): any bytes, bytes after a good header, a random flipped bit; plus every bit and every truncation of one file |
| The key file | The guard unwraps it; anything that is not a 32-byte key is `Locked`, and the file is never deleted or replaced | `MAX_KEY_FILE_BYTES` (64 KiB) | No; tests cover a refused and a short key |
| Blob names | `BlobName` (a-z, 0-9, `-`, 1–32 bytes), and `RESERVED_NAMES` refuses Windows device names before any file is touched | 32 bytes | No; tests cover every device name |

## Threats (STRIDE-lite)
- **Spoofing:** a blob copied or renamed to another blob's name fails authentication, since the name is the AAD.
- **Tampering:** any changed bit fails the tag; the blob is renamed to `<name>.damaged` and reported once as `Damaged`. A save writes a temp file in the same folder, syncs it, then renames it over the old file, so a crash never leaves half a file.
- **Logging / repudiation:** the crate never logs. Its errors name only the kind, and `SealedFiles`' `Debug` shows neither the folder (it holds the account name) nor the key.
- **Information disclosure:** a copy of the folder on another PC or account cannot be opened, since only the guard unwraps the key (ADR-0010). The key, the cipher's copy of it and opened bytes are wiped on drop; the buffer that briefly holds plain bytes while sealing is wiped too.
- **Denial of service:** reads are capped by size. A lost key file sets the old blobs aside and the next save makes a new key; a key file the guard refuses keeps the store `Locked` rather than losing the data.
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
