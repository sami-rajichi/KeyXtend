# 44. The engine loads its card and processor parts from the worker's folder only

- **Date:** 2026-11-14
- **Status:** Accepted. Adds to ADR-0042's `voice` job and ADR-0043's runner how a device is picked and how the engine's parts load.

## Context

- On a laptop graphics card, every trial model answered a 30 s clip in 0.4 to 1.3 s, against 2.9 to 29 s on the processor, with the same mistakes (`docs/items/P11.md`).
- A statically linked Vulkan build imports the Vulkan library, so the worker would not start on a PC without Vulkan; it also added 45 MB.
- The roadmap's weak PC has no AVX, and a build made for a newer processor may not run there.
- The first use of a model family on a card compiles its programs for 23 to 36 s. A card with too little free memory, or a driver that computes wrongly, must never freeze anything or change the text.
- The engine reads four environment variables (found 2026-11-14): `GGML_BACKEND_PATH` loads an outside library, `TRANSCRIBE_DUMP_DIR` dumps working data to disk, and `TRANSCRIBE_BACKENDS` and `GGML_DISABLE_VULKAN` narrow the devices.
- The owner wants the processor or a card, not through CUDA alone, and users to install nothing.

## Decision

Decided with the owner on 2026-11-10; the design is [`2026-11-10-voice-typing-design.md`](../superpowers/specs/2026-11-10-voice-typing-design.md) (Device, Memory, Privacy and security).

1. **Loadable parts.** The worker is built with the engine's loadable parts (`dynamic-backends`). The graphics-card part and the processor variants ship as files beside the worker. At start the engine picks the processor variant the PC supports, so a processor without AVX still runs.
2. **The worker's own folder.** The engine loads its parts through an explicit directory, the worker's own folder, before any device list or model load. It never loads them from the models folder.
3. **A clean start.** The host starts the worker with every `GGML_*` and `TRANSCRIBE_*` variable removed, and with its own folder as the working folder. No variable can then load an outside library, write working data to disk or narrow the devices.
4. **Users install nothing.** The worker starts on a PC without Vulkan, and then offers no card. Graphics drivers bring Vulkan; the Vulkan SDK is a build tool only.
5. **The fit check.** Before a card is used, the model's card memory plus `card_margin_mb` must fit in the card's free memory at that moment.
6. **The warm-up.** A model family's first use on a card warms it up in the background, with progress. The driver keeps the result.
7. **The self-test.** The model transcribes a shipped clip on the card and on the processor: one clip each in English, French and Arabic, six seconds at most, the first whose language the model knows. It passes when the texts differ by at most `card_diff` words and the card is at least `card_gain` times faster. The result is kept per card, driver version and model.
8. **Automatic.** `device` is `auto`, `cpu` or a card's id. Automatic uses the processor until a card has passed its check, then the best checked card with room. The processor serves while a check runs.
9. **Fallback.** A card that reports an error while loading or running hands the same recording to the processor, with a note. Its check result is dropped, so the check runs again.
10. **Threads.** On the processor, `threads` is capped at the real cores, and 0 means that cap: 16 threads ran 3 to 8 times slower than 8.

## Other platforms

- The card part is Vulkan on Windows and Linux, and Metal on a Mac (P15).

## Consequences

- A library planted in the models folder, or named by a variable, is never loaded.
- The card part and the processor variants count against the 80 MB download budget: they pack to 18.7 MB, so the download stays near 63 MB (`docs/items/P11.md`, 2026-11-14).
- With the card part missing or damaged, the worker starts on the processor; with no processor part it cannot start, so the stage check requires one.
- A driver update or a new model file runs the check again.
- A card speeds up dictation only after its check; until then the processor serves, so nothing waits on the card.
