---
name: kx-security-check
description: Use when designing or finishing any KeyXtend module, when adding IPC messages, file formats, clipboard/vault/voice handling, or any code that touches input injection, hooks, secrets or the network. Produces or updates docs/security/<module>.md.
---

# Security check for a KeyXtend module

KeyXtend runs with **uiAccess**, so it can drive admin windows. A bug here matters more than in a normal app. Be concrete, and name the file and the line.

## 1. Capabilities and data
- List the module's `capabilities` from its manifest. For each one: why it is needed, and what breaks without it. Remove any that are not strictly needed.
- **Data classes** handled:
  - **Secret:** passwords, vault, API keys.
  - **Sensitive:** typed text, clipboard, transcripts, learned words, screenshots.
  - **Public:** settings, layouts.
- For each data class: where it is stored (must be encrypted, ADR-0010, if Secret or Sensitive), how long it is kept, and who can read it.

## 2. Threats (STRIDE-lite; answer each one)
- **Spoofing:** can another process pretend to be our worker, or us? (Check pipe auth and inherited handles.)
- **Tampering:** can a file we read (settings, vault, models, word lists) be changed to make us misbehave? Is it validated? Are hashes checked?
- **Repudiation / logging:** does any log line contain Sensitive or Secret data? Grep the module for `tracing::`, `println!`, `dbg!`, `{:?}`, and for `Debug` derives on sensitive types.
- **Information disclosure:** clipboard exclusion formats respected? Password fields excluded from learning? Is anything sent over the network, and is it opt-in?
- **Denial of service:** huge inputs (a 200 MB clipboard image, a 10-minute recording, 10k entries), malformed IPC, a hook callback that can block. Every one needs a size limit or timeout.
- **Elevation:** can this code make the uiAccess process parse untrusted data (forbidden, ADR-0004) or inject input the user did not ask for?

## 3. Code checks
- No `unsafe` outside platform crates. In platform crates, every `unsafe` block has a correct `// SAFETY:` comment. Check the lifetimes of HWNDs and buffers.
- No `unwrap()`/`expect()` on data from outside: files, IPC, OS callbacks, clipboard.
- Integer sizes from outside are checked before allocating.
- Secrets are zeroised (`zeroize`) after use and never cloned into `String`s that live long.
- New dependencies: `cargo deny check`, then `cargo audit`. Look at what the crate itself does (build scripts, network, `unsafe`).

## 4. Tests to add
- Fuzz targets (`cargo fuzz`) for every parser and IPC decoder the module owns.
- A property test for every size limit and state machine.

## Output
Write `docs/security/<module>.md` from `docs/security/TEMPLATE.md`. Put each finding under **Must fix** (blocks done), **Should fix** (tracked issue) or **Accepted** (with a reason).
