# 10. Sensitive data is encrypted at rest with an OS-protected key

- **Date:** 2026-09-26
- **Status:** Accepted

## Context
The clipboard history, vault, learned words and voice transcripts can contain secrets.

## Decision
- XChaCha20-Poly1305 (RustCrypto).
- A random data key per store, wrapped by DPAPI (Windows), Keychain (macOS) or Secret Service (Linux).
- The vault also requires user-presence verification (Windows Hello/PIN) before it decrypts, and has an optional Argon2id master password.
- Export format: `age` (encrypted), plus an optional plain CSV behind a warning.
- We never write cryptographic primitives ourselves.

## Consequences
Data cannot be read by copying the files to another PC or user account. The user can delete all data from Settings → Privacy.
