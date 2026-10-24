# Security note: kx-mod-predict

- **Owner:** product owner
- **Last review:** 2026-10-24 (P7 stub: folding and the pack format; the module, the learner and its threats come later in P7)
- **Status:** Draft
- **Scope:** `crates/kx-mod-predict`: folding (`fold`, tables in `fold.toml`) and the word-pack format (`pack`: the reader, and the writer behind the build-only feature `build`). It will run inside the uiAccess `keyxtend` process; the writer runs only in `cargo xtask words`, and `cargo xtask tidy` (D7) refuses `build` in the app's tree.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|

- None yet: there is no module, manifest or settings; they are added with the module.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| A word pack's seed words, frequencies, capitals and chains | Public (wordfreq, Tatoeba and Wikipedia data) | The shipped `<iso>.kxw` files, read in place from a read-only map | No | As shipped |
| A folded word (`fold`) | As the word it folds: typed text stays in `Redacted<T>` with the caller | Memory only, never logged | No | Not kept |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Word pack bytes (`Pack::parse`) | Magic, version and every header field in range, the language code one the port's `fits_iso` takes and its padding zero; each section inside the file, after the table, known and once; the required sections present and sized to the word count; every spelling's offsets in order and its UTF-8 valid; the lexicon's own CRC-32C (`fst` `verify`). Each later read checks its offsets, ids and lengths again with checked slicing, as a mapped file could change under the view | The adapter maps 1 byte to `MAX_PACK_BYTES` (64 MiB), and the writer makes nothing larger; a pack names at most 2^24 words, spellings of 128 bytes; a prefix stream reads at most as many keys as words | Property tests: random bytes, every truncation, every single-byte change, several changes, and changes after parse outside the lexicon (`pack/tests/damage.rs`) |
| `fold.toml` (compiled in) | Parsed once; refused for a key listed twice, a key not lower case, a key mapping to itself, or a replacement that folds again; a broken table folds case only | Compiled in, never read from disk | Unit tests; property tests that folding twice changes nothing and that a Greek word folds to a prefix of its longer forms |

## Threats (STRIDE-lite)
- **Spoofing:** to be filled in with the module.
- **Tampering:** a damaged pack is refused at parse or reads as empty, and never panics our code. `fst` reads its lexicon's nodes with ordinary indexing, which panics on a lexicon changed after its checksum passed at parse, though it never reads out of bounds (see Accepted).
- **Logging / repudiation:** the crate logs nothing.
- **Information disclosure:** packs hold no user data.
- **Denial of service:** a prefix stream reads at most as many keys as the pack has words, and a context search halves its range each step.
- **Elevation:** the crate forbids `unsafe` (`lib.rs`).

## Findings
### Must fix (blocks Definition of Done)
- None open.
### Should fix (tracked issue)
- None open.
### Accepted (with reason)
- A lexicon changed under the view after its checksum passed could make `fst` panic: the Windows adapter opens packs with read sharing only, so no program can write a mapped pack, and the kernel will contain the module's panic.
