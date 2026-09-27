# Security policy

This app runs with Windows **uiAccess**, which lets it send input to other windows, including admin windows. We treat security bugs as the highest priority.

## Reporting a vulnerability
- Please **do not open a public issue**. Use GitHub's private vulnerability reporting (Security tab → "Report a vulnerability").
- We aim to reply within 7 days and to fix confirmed issues before disclosing them publicly. We credit reporters who want credit.

## Scope
**In scope:**
- the `keyxtend` app and `keyxtend-worker`;
- the installer;
- the IPC between them;
- stored data (clipboard history, vault, learned words, transcripts);
- the release pipeline.

**Out of scope:**
- issues that need an attacker who already runs code as the same user with the same rights (unless our app turns that into more rights);
- third-party cloud speech services that the user chose to enable.

## What we promise
- No telemetry. Nothing leaves your PC unless you turn on a cloud feature with your own key.
- Sensitive data is encrypted at rest with a key protected by your OS account.
- Releases come with SHA-256 checksums, an SBOM and GitHub build attestations (`gh attestation verify`).
