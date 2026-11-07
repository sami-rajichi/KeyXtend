# 38. Quick-fill is a notebook sealed with the account key and backed up to a locked box

- **Date:** 2026-11-01
- **Status:** Accepted. Supersedes ADR-0010's vault lines: user presence (Windows Hello, PIN), the optional Argon2id master password, and the `age` and CSV export. ADR-0010's data key, cipher and "no own primitives" rule stand.

## Context

- The spec's vault (§8.1) had three fixed tabs, Windows Hello before any password, an idle lock, and an `age` and CSV export.
- The owner (2026-11-01) wants a notebook of any values, filled in one click like the clipboard's Insert, with no unlock step, working the same on Windows, macOS and Linux; Hello is not an option, as a PC may have no Windows password.
- The owner also wants the data to survive a lost, broken or wiped laptop, without trusting a cloud company with readable data, and without remembering where a backup is.
- Browsers keep saved passwords under the account's own key (DPAPI, Keychain, Secret Service) and fill them without a prompt; ADR-0010 already gives every store such a key.
- An authenticator code cannot lock an offline file: with no server to check it, a thief with the file skips the check.

## Decision

Decided with the owner on 2026-11-01 (P9); the design is `docs/superpowers/specs/2026-11-01-quick-fill-design.md`.

1. **A notebook of entries.** Each entry has a name, a kind (Person, Login, Wi-Fi, Card, Note, Other, from a kinds table) and fields: a label, a one-line value and a hidden flag. Row buttons fill at once (User, Password, Both, Insert); a row click does nothing.
2. **No unlock step.** The notebook is sealed under its own data key in `<data>/quickfill/`, wrapped by the OS key (ADR-0010). There is no Hello, PIN, master password or idle lock in P9; a PIN lock is a P13 idea, off by default.
3. **The `Vault` capability.** It gates the notebook store (`VaultStore`), the `Boxes` and `Places` ports, and every Quick-fill event that carries values. The keyboard and the ui hold it as event gates, and prediction holds it for `VaultEmails` only.
4. **The keyboard stays the only typist.** `FillType` carries text and Tab parts; the keyboard lets its latches go, types each part as Unicode, never sends Enter, and tells nothing to prediction. No value goes to the clipboard, and there is no check of the box's kind.
5. **The backup box.** One file, `KeyXtend Backup.kxbox`, in each place the user picked, rewritten after every change. It is locked with the user's own password (8 characters or more), which is never kept: KeyXtend keeps the box's public key (an X25519 `age` recipient) and a sealed copy of the private key locked by the password (scrypt). Memory checks ask for the password now and then; after `check_tries` wrong answers, the user sets a new one. A box made by another computer is never overwritten unasked.
6. **The worker reads and writes the formats.** A `vault` job in `keyxtend-worker` seals and opens boxes and reads KeePass files (KDBX 3.1 and 4), with the media job's rights, so the uiAccess app parses neither format (ADR-0004).
7. **Places by meaning.** The `Places` port suggests the cloud folders and removable drives an adapter's table finds, lists folders page by page, finds boxes by a name that holds `keyxtend` in any case and the mark inside, and writes a box in one step. KeyXtend never goes online: a cloud app syncs its own folder.
8. **Hello dropped.** The spike's `hello.rs` and `fill.rs` are deleted, and gate G23 is aimed at the product's fill instead of Hello.

## Other platforms

- macOS: Keychain wraps the key; the places table lists iCloud Drive and the usual cloud apps' folders and mounted volumes.
- Linux: Secret Service wraps the key; the table lists Nextcloud, Dropbox and pCloud folders and removable media.

## Consequences

- One click fills, but anyone using the signed-in PC can fill too, as in the browsers; the README says so.
- A forgotten backup password costs nothing while the computer works; with the computer gone and the password forgotten, the box cannot open.
- The box holds named parts, so P13 can add settings and learned words to it later.
