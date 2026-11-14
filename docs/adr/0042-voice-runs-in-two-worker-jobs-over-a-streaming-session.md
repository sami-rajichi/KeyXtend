# 42. Voice runs in two worker jobs over a streaming session

- **Date:** 2026-11-14
- **Status:** Accepted. Adds a streaming session beside ADR-0034 decision 5's one-ask-one-reply host. Supersedes ADR-0004's line that the voice worker may open network connections: the `fetch` job downloads, and the `voice` job has no network.

## Context

- P11 adds voice typing on this PC ([`2026-11-10-voice-typing-design.md`](../superpowers/specs/2026-11-10-voice-typing-design.md), approved by the owner on 2026-11-10).
- The worker host sends one ask and waits for one reply (ADR-0034 decision 5). A dictation needs the worker to speak on its own: sound levels many times a second, each piece's text when ready, and faults.
- ADR-0004 let one voice worker hold both the microphone and the network, the two riskiest powers a job can have.
- The models range from 194 MB to 2.41 GB, so one fixed memory cap would be too tight for some and too loose for others.
- Try again, Compare models and a model change need the same recording in a job that loads another model.
- A dictation must stop and wipe its sound when the PC locks or sleeps.

## Decision

Decided with the owner on 2026-11-10; the design is [`2026-11-10-voice-typing-design.md`](../superpowers/specs/2026-11-10-voice-typing-design.md) (Parts, Memory, Privacy and security).

1. **A streaming session.** The worker host gains a second session kind beside one ask, one reply. The app still sends asks with deadlines, and the worker also sends notes on its own: levels, a piece's text, a state and a fault. Each note is a checked `kx-ipc` frame with its own size range, and an unknown or oversized note ends the session.
2. **Two jobs.** `voice` opens the microphone and runs the voice detector and the engine; it has no network. `fetch` downloads model files; it has no microphone. Both run under the worker's restricted token and job limits.
3. **One program, two kept apart.** Both jobs live in `keyxtend-worker`, so `cargo xtask tidy` keeps the `voice` files from the network crate and the `fetch` files from the audio crate.
4. **One model per job.** A `voice` job loads one model, and its memory cap is that model's row plus `job_mb`. A file no row names counts as its size. A model change ends the job and starts a new one, so no cap ever changes and a model always leaves memory whole.
5. **Listening without a model.** A `voice` job may load no model, capped at `job_mb` alone. It records, cuts and hands over its sound at Stop; part 2's cloud dictation uses this.
6. **The sound hand-over.** A recording a new job needs crosses through the app as secret bytes in wiped buffers, in pieces under the frame limit. The app passes it on and never reads sound. Text and sound cross the pipe as secrets and are never logged.
7. **The `Voice` capability.** A new event gate carries the voice events: the key, its light, the bar's view, its clicks and the text to type. The voice module, the keyboard and the ui hold it. Sound levels carry no content and name no gate.
8. **The module's other powers.** It holds `Microphone` (the speech port), `Network` (the model files port, for downloads only), `SecretStore`, `ReadFocusedText` and `WriteClipboard` (Copy). It holds no `InjectInput`: the keyboard stays the only typist.
9. **A session port.** A new port, `SessionWatch`, tells when the session locks, unlocks, sleeps or wakes, so a dictation stops and wipes. It has no gate, as it carries no content.

## Other platforms

- The ports are neutral. macOS and Linux start the same jobs and fill `SessionWatch` from their own lock and sleep signals (P15).

## Consequences

- The microphone and the network never share a process.
- A model change costs a job start; on the processor every trial model loaded in under 2.5 s (`docs/items/P11.md`).
- A crafted recording or model file meets only the worker; the app relays sound it cannot read.
- The streaming session can serve later jobs, such as part 2's `speech-net`.
- Each new frame needs its property tests and a line in `docs/security/kx-ipc.md`.
