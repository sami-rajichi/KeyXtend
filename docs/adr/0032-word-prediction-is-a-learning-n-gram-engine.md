# 32. Word prediction is a learning n-gram engine of our own

- **Date:** 2026-10-24
- **Status:** Accepted.

## Context

- Spec §6 asked for our own small engine with unigram and bigram counts, seeded from wordfreq. The owner then asked for guesses from the last N words, set in the settings, for every layout language, at a quality users can rely on (2026-10-24).
- A research round compared today's keyboards (the report on keyboard prediction engines, kept under `target/research/`):
  - Gboard still ran an n-gram finite-state decoder in 2024, with small LSTMs of 1.4M–6.4M parameters ranking only the next word. Its LSTM raised top-3 next-word recall from 22.1% to 27.0% over a 1.25M n-gram model; replacing the whole decoder cut corrections by about 1% and added 17–28% latency.
  - A word n-gram with 5 suggestions saves 42–60% of key presses in English and French simulations; learning from the user adds 3 to 9 points in French, levelling off after about 25,000 words.
  - No study reports savings for Arabic, and Google and Apple both leave Arabic out of their newest prediction.
  - Windows' `TextPredictionGenerator` is documented as Japanese-only.
- The app is fully local, at the edge of its memory budget (ADR-0021), with about 36 MB of its 80 MB download left.
- A read-only memory map of a 64 MiB file, every page read, raised private memory by 0.1 MB on the development PC.

## Decision

Decided with the owner's approval of P7's design on 2026-10-24.

1. **An n-gram engine in Rust.** Per language, a lexicon finishes the word and word-chain counts guess the next one from at most `context_words` previous words (2 by default, 1–4), backing off to fewer. No neural model and no OS prediction API in v1.
2. **Packs read from disk.** Each language's seed is one pack file: a lexicon (`fst` crate) from folded keys to correctly spelled words, and pruned n-gram tables of word ids with 8-bit scores. The app maps it read-only and loads only the language in front.
3. **Open, credited seeds.** Words from wordfreq, kept where an open spelling dictionary knows them and given its capitals; chains from Tatoeba and Wikipedia. Packs ship as separate data files with their credits (CC BY-SA 4.0, CC BY 2.0 FR), built once by `cargo xtask words` and kept in the repository.
4. **A capped learner on the device**, after AOSP LatinIME's design: counts with last-used days for words and chains, a two-use gate, caps of 10,000 words and 30,000 chains per length, clean-up of the oldest at 1.2 × a cap, a purge after 300 days. Its file is sealed by `kx-crypto` (ADR-0010).
5. **Matching folds, typing never does.** French drops accents and Arabic drops harakat and tatweel and folds the alef forms and ى for matching; the chip always types the stored spelling.
6. **Measured, not assumed.** A meter replays held-back sentences: English and French must save at least 40% of presses with 5 chips; Arabic's number is reported.

## Consequences

- Prediction runs the same on Windows, macOS and Linux; only the file map and the field reader are per platform.
- A neural re-ranker or a model distilled into the packs stays possible later, where the meter shows the chains fall short.
- `kx-crypto` moves from P8 to P7, its first user.
- The packs add about 20 MB to the download, checked by the stage's size limit.
