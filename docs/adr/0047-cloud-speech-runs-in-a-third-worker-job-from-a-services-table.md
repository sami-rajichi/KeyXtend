# 47. Cloud speech runs in a third worker job from a compiled-in services table

- **Date:** 2026-11-17
- **Status:** Accepted. Supersedes ADR-0007's cloud lines (Groq, Cloudflare Workers AI and Azure Speech F0; Gemini's free tier with a warning) and its line that keys are stored with the OS secret store; its "never ship an API key" stands. Adds a third job to ADR-0042 decisions 2 and 3, and supersedes its decision 8's "for downloads only". Adds the voice module as a holder of ADR-0041's `OpenLink`.

## Context

- P11 part 2 lets the user pick, per language, a cloud service and model with their own key ([`2026-11-10-voice-cloud-design.md`](../superpowers/specs/2026-11-10-voice-cloud-design.md), approved by the owner on 2026-11-10). The owner's notes of 2026-11-17 are in [`docs/items/P11.md`](../items/P11.md) and amend the plan, [`2026-11-10-phase-11-voice-cloud.md`](../superpowers/plans/2026-11-10-phase-11-voice-cloud.md).
- ADR-0007 named three free services, a Gemini warning and keys in the OS secret store. The research of 2026-11-09 and 2026-11-10 found eleven services the owner wants, reached through six request shapes.
- Prices, free use, training, retention and region change, so they need a date and must never sit in code.
- ADR-0042 keeps the microphone and the network apart. A cloud dictation needs both, at different times.
- The owner settled that nothing leaves the PC before Stop, and that cloud stays off until the user switches a service on.
- The card's link and Paste on the key line need `OpenLink`, which only the selection module holds (ADR-0041), and `ReadClipboard`.

## Decision

Decided with the owner on 2026-11-10 and 2026-11-17; the design is [`2026-11-10-voice-cloud-design.md`](../superpowers/specs/2026-11-10-voice-cloud-design.md) (How a cloud dictation runs, Connections and services, Keys, Privacy and security).

1. **A third job.** `speech-net` in `keyxtend-worker` sends a dictation's pieces to one service and returns their texts. It has the network and no microphone, under the worker's restricted token and job limits, capped at `job_mb`. Tidy keeps its `net` files from the audio crate, as it keeps `fetch`'s.
2. **Nothing before Stop.** A cloud dictation opens part 1's `voice` job with no model (ADR-0042 decision 5), which cuts pieces within the service row's limits. Cancel sends nothing, and dropping a run drops its requests.
3. **The relay at Stop.** The app takes the pieces from `voice` and hands them to `speech-net` as unread secret bytes in wiped buffers, several at a time within the row's limits, with `net_wait_s` and `net_tries`. The frames `NetSend` and `NetTest` (the card's key check, with no sound where the service allows), with the notes `NetText` and `NetFault`, are checked `kx-ipc` frames, and a reply's size is capped in `limits.rs`.
4. **Faults keep the recording.** A fault names a bad key, no credit, the allowance used (with its renewal, if told), the rate, too big, the language, an empty text, a strange reply, the network or a timeout. The recording is kept, the bar offers Try again and "Use the local model", and nothing is typed twice.
5. **Six request shapes.** One small adapter per shape: the OpenAI-style upload form, OpenRouter's JSON, DeepInfra's form, Azure fast transcription, ElevenLabs' form and Gemini. Each builds a request from the row's fields and reads the reply into text or a fault; the worker holds no table. OpenRouter's keep-nothing switch, on to start, asks only for hosts that keep nothing.
6. **HTTPS through `kx-platform-net`.** Its `Http` port gains a POST with headers and a body, through the same WinHTTP adapter, so no new HTTP crate. Every request of the job, the key test included, goes to an HTTPS address on the row's host or the typed Custom or Azure host, and follows no redirect to another host.
7. **A compiled-in services table.** `kx-mod-voice` holds one row per service: id, name, shape, address or "typed", key header, where the key is made, limits, up to ten models with their languages and codes, the facts with their date and source, the warning, the terms number and whether it is offered. Groq comes first; Gemini is built but not offered until six dictations by the owner pass.
8. **Facts as numbers, dated.** A card's sentences are translations filled with the row's numbers, so no price or claim sits in code or twice. The rows hold the facts read on the services' own pages on 2026-11-17; they are read again on the manual check's day and at each release, never over the network. Past `facts_days` the card's date takes the warning tone.
9. **Terms numbers.** A row's terms number rises when its training, retention or region fact changes. `services` keeps the date and number the user confirmed; a newer number asks again before the next upload, and a No turns the service Off and its picks back to the local default.
10. **Keys sealed per service.** A key is the user's own, typed or pasted on the panel line (ADR-0040) and held as a `Secret`. It is sealed as one blob per service in `<data>/voice/keys/`, on the voice shelf with its own key (ADR-0010), and reaches only the module and `speech-net`, in a wiped buffer per run. A hidden run reads and writes no key, and cloud is off there (ADR-0037).
11. **Capabilities.** `Network` now also covers the services the user switched on, beside downloads: it gates the `CloudSpeech` port (`CloudSpeechKey`), the module's only route to `speech-net`. The voice module also holds `OpenLink`, for the card's link to the row's HTTPS page, and `ReadClipboard`, for Paste through the `ClipTake` route Quick-fill uses; it still holds no `InjectInput`.
12. **No cloud by itself.** With `cloud` off no service is offered, nothing is sent, and cloud picks fall back to the local default. A cloud model is never a default, nothing moves from local to cloud on its own, and a password box uses the local model or says none is ready.

## Other platforms

- The `CloudSpeech` port, the frames and the job are neutral. macOS and Linux fill the POST with their own HTTPS stack under the same rules (P15).

## Consequences

- The microphone and the network still never share a process, and the app relays sound it never reads.
- A service that speaks a known shape is one table row and its tests; a new shape is one adapter file.
- The bar's cloud mark and the service's name show whenever the voice will leave the PC.
- A connection with no real-key check ships marked "untested" on its card.
- Paste's take makes the history forget its newest entry; the system clipboard keeps the key until the next copy, as with Quick-fill.
- `Network`, `OpenLink` and `ReadClipboard` gain the voice module, so the app's capability pins and the security notes change with this record.
- Licences checked on 2026-11-17 (`kx-licence-check`): requests and replies are built and read with `serde_json` 1.0.151 (MIT OR Apache-2.0), already built into the app. Base64 uses the `base64` crate 0.23.1 (MIT OR Apache-2.0), the release `keepass` already builds into the worker, so no new crate and no hand-written encoder.
- The key, warning and outside-link icons come from Lucide 1.48.0, the set already bundled (ISC; those three MIT, from Feather), and its cloud icon is in already. Each crate and icon enters the build with the task that uses it.
