# 43. One engine runner and a compiled-in models table serve every model

- **Date:** 2026-11-14
- **Status:** Accepted. Supersedes ADR-0007's speech-to-text engine lines: "Default: whisper.cpp (MIT code and weights) through whisper-rs, with models downloaded on demand" with its two models (`large-v3-turbo` q5_0, `small` q5_1), and "Parakeet and Canary (no Arabic)" in its Rejected line. Its benchmark button becomes Compare models. ADR-0007's cloud, Moonshine, prediction, OCR and camera lines stand.

## Context

- ADR-0007 chose whisper.cpp through whisper-rs, and rejected Parakeet and Canary because they know no Arabic.
- On the owner's voice, newer models did far better than Whisper: Parakeet v3 got 1 French word wrong in 74 against 2 to 14, and Cohere Transcribe Arabic 0 Arabic words wrong in 57 against 12 to 23 (`docs/items/P11.md`).
- `transcribe.cpp` (MIT), through its Rust binding `transcribe-cpp`, read all ten trial models and an older whisper.cpp file, and builds with CMake alone. `whisper-rs` 0.16 does not build on Windows without libclang.
- The owner wants engines and models as plug-in parts: adding, removing or updating one is a table row or one small adapter.
- Models weigh 194 MB to 2.41 GB, so none can ship. The owner wants them in one visible folder, never a hidden cache, and a file already there reused.

## Decision

Decided with the owner on 2026-11-10; the design is [`2026-11-10-voice-typing-design.md`](../superpowers/specs/2026-11-10-voice-typing-design.md) (Parts, The models table, The models folder and downloads).

1. **One runner.** The `voice` job (ADR-0042) runs models through one small `Engine` trait, with one adapter per runner. Today one runner, `transcribe.cpp`, serves all ten models. A new runner is one adapter file and its name in the table's `runner` column.
2. **A compiled-in models table.** `kx-mod-voice` holds one row per model: id, shown name, runner, file name, size, SHA-256, source addresses at a fixed revision, licence and credit, each language with the model's own code for it, and the memory needed on the processor and on a card. The worker holds no table; every ask carries the fields it needs.
3. **The first rows.** They are the ten files the trial read, all from `handy-computer` on Hugging Face, with the sizes and fingerprints in `docs/items/P11.md`. None ships: each passes `kx-licence-check` as a user download. Another cut of a model is another row, tested before it may become a default.
4. **Order per language.** One list per language orders the models by the project's own tests, with their date, and the first is the default. Today: Parakeet v3 for English and French, and Cohere Transcribe Arabic for Arabic. Nemotron stays listed but is never a default.
5. **Other languages.** Dictation works in every keyboard language a model knows. A language without its own list takes a general order of the models that know it, marked untested. With no model for it, the bar says so.
6. **The models folder.** Models live in one visible folder the user picks at the first download (`models_dir`). The offer is a `KeyXtend\models` folder on the drive with the most room. A file already there is known by its fingerprint and reused, and files are never moved.
7. **Other files.** A model file no row names is listed as "Other file, not checked". The engine reads its languages from the file, in the same sandbox.
8. **Downloads.** The `fetch` job downloads over HTTPS only, from the table's addresses, into a `.part` file, once `min_free_mb` is free. Size and SHA-256 are checked before the file gets its name, and a wrong fingerprint deletes the part. A redirect to a host the row does not list is refused.
9. **Asked network use only.** Clicking Download is the user asking for that network use. Nothing else in part 1 touches the network.

## Consequences

- A model weak for one voice stays listed, as it may suit another.
- Changing a default is a table edit backed by a dated test.
- Compare models replaces ADR-0007's benchmark button: it counts words wrong on the user's own voice, as well as time and memory.
- The spike's separate whisper server and its local port go; `THIRD_PARTY.md` credits the engine and each model as a user download.
