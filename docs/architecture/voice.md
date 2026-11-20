# Voice typing (P11, ADR-0042 to ADR-0052)

A detail section of [ARCHITECTURE.md](../../ARCHITECTURE.md), which keeps the overview, the code map and the invariants. The designs are [`2026-11-10-voice-typing-design.md`](../superpowers/specs/2026-11-10-voice-typing-design.md) on this PC and [`2026-11-10-voice-cloud-design.md`](../superpowers/specs/2026-11-10-voice-cloud-design.md) for the cloud services (part 2).

- **The keyboard stays the only typist.** `kx-mod-voice` holds no `InjectInput`: a text goes as `TextAsk { round, erase, text, gap_ms }`, and the keyboard erases with its erase key, types by a letter's route in pieces under `text_cap`, keeps every latch and answers `TextDone { round, count, ok }`. Prediction hears one `Break`, never the text.
  - With `insert = "paste"` the module sends `PasteText` instead, and the clipboard history pastes the text as ours and puts the clipboard back (ADR-0048). A no, a late answer or a password box types the text once.
- **Events.** Named `Voice`: `VoiceKey` (keyboard to module: the Dictation key), `VoiceLit { rec }` (module to keyboard: the key's light, amber while preparing), `VoiceOpen` (ui to module: the tray row or the gear), `VoiceAsk` (a late starter asks for the light and the view again), `VoiceView` (module to ui: the bar and the panel, once per change), `VoiceAct` (ui to module: a click with the view's round), `TextAsk` and `TextDone`. Ungated: `VoiceLevel`, the sound level, which holds no content and never wakes the panel.
  - The panel's typed lines add `VoiceLine` (module to keyboard: the line the page shows), `VoiceLineKey` and `VoiceClose` (keyboard to module), all named `Voice` (ADR-0049).
- **Ports.**
  - `Speech` (`Microphone`) opens a `SpeechRun` (stop, cancel, again, keep, close) that tells `SpeechNote`s on any thread; it also lists `Device`s, reads a file's `ModelFacts`, runs a card's check and names the check clips' languages. An ask may name no model: the job then only records, for a cloud dictation, and hands its pieces over at Stop.
  - `CloudSpeech` (`Network`, `CloudSpeechKey`) sends a run's pieces to one service and tells each piece's text in order, or a `CloudFault`; `test` checks a key with no sound. Dropping a run cancels it and never blocks (ADR-0047).
  - `ModelFiles` (`Network`) lists the fixed drives with their room, makes and scans the models folder, fingerprints a file and fetches a table row into a `.part` file.
  - `SessionWatch` (no gate) tells a lock, unlock, sleep or wake, and `FolderReveal` (`OpenFolder`) shows a plain local folder in the file manager (ADR-0046).
- **Who holds what.** `Voice`: the module, the keyboard and the ui, as an event gate; `Microphone`, `Network` and `OpenFolder`: the module alone. The module also holds `SecretStore` (its own shelf, `VoiceShelfKey`), `ReadFocusedText` (the box's kind, so a password box shows dots and keeps nothing), `WriteClipboard` (Copy and Paste at once), `OpenLink` (a card's link, `WebLinkKey`) and `ReadClipboard` (Paste on a typed line, through `ClipTake`); `tests/life/ships.rs` pins each list.
- **The module.** The `kx-voice` actor feeds a pure flow (Idle, Preparing, Listening, Transcribing, Text, Note) on the injected clock, and the Voice panel's state, then carries out their effects: speech and cloud runs and slow reads on short job threads, texts on one typer thread. The language is the keyboard's shown layout (`LayoutChanged`), fixed at Start and never guessed.
  - Preparing lasts from Start until the microphone opens; the bar then shows in Listening. The gear or the tray during a dictation ends it as Cancel does.
  - A cloud pick sends nothing before Stop; a password box always uses the local model, and a failed upload keeps the recording for Try again or "Use the local model".
- **Tables.** `models.toml` holds the ten models, each with its file, size, SHA-256, pinned addresses and allowed hosts, memory and language codes, and an order per tested language whose first is the default. `score.toml` holds the scoring rules per language, and `options.toml` the Options page's rows.
  - `services.toml` (table-wide values and upload-form rows), `services-own.toml` (the services with their own shape) and `langs.toml` (language sets) form the services table: one row per service with its shape, host, limits, models, dated facts, terms number and whether it is offered (ADR-0047, ADR-0050).
- **The request shapes.** The worker holds one adapter per shape and no table; the row's fields travel in each ask.

  | Shape | Rows | Request |
  |---|---|---|
  | Upload form | Groq, Lemonfox, Custom, OpenRouter; hidden: OpenAI, Mistral, Together, DeepInfra | one multipart form a piece, with the row's language field and fixed fields |
  | ElevenLabs | ElevenLabs | its own form fields and key header |
  | Gemini | Gemini | one JSON request, the sound inline as Base64, `store` off |
  | AssemblyAI Sync | AssemblyAI | a JSON `config` part first, then the sound; the model in the row's header; no key test (ADR-0051) |

- **The worker.** Three jobs of `keyxtend-worker` over a streaming session (`kx-platform` `worker/stream`): asks with waits, and notes the worker sends on its own, each a checked `kx-ipc` frame. Tidy F5 keeps their files apart, and D3 and D4 keep their crates out of the app.
  - `voice` hears the microphone through `cpal`, resamples to 16 kHz mono, cuts pieces at pauses with the Silero detector and writes text with `transcribe.cpp`; it has no network.
  - `fetch` downloads one model file over HTTPS (`kx-platform-net`) and has no microphone.
  - `speech-net` POSTs a run's pieces over HTTPS (`kx-platform-net`) in the row's shape, paced within the row's limits, with no redirect followed and each reply capped; it has no microphone.
- **The hosts.**
  - `kx-platform` `speech` starts one job per model, capped at the row's memory plus `job_mb` plus room for the longest recording, and ends it after `idle_min`. Devices, a file's facts and a card's check each run in a job of their own, and a model change hands the recording over in wiped pieces.
  - `models` checks the drive's room, then runs one `fetch` job per download, capped at `job_mb`.
  - `cloud` (`WorkerCloud`) relays at Stop: one `speech-net` job per run or key test, capped for its pieces and replies, with the key and sound in wiped buffers. Each piece has a time bound from `net_wait_s` and `net_tries`, and dropping a run cancels it without waiting on the worker.
- **The engine's parts (ADR-0044).** The worker loads the engine's processor parts and its Vulkan card part from its own folder only, never from the models folder; the Windows spawner starts it with only the variables `env.toml` keeps, a system-only `PATH` and its own folder as working folder. A card serves a language only after its check passes on that language's clip, or the first clip the model knows when none ships for it (ADR-0052); Automatic then takes the fastest checked card with room, else the processor. The check the language in front's model lacks runs on the picked card, or under Automatic on each card checked for that model before or that passed another model's check under its present driver, after the devices are read and at each change of that language; checks run one at a time, and one wanted during a check, a dictation or Compare starts when they end.
- **Files.** `<data>/voice/` (`VOICE_DIR`) is a sealed shelf with its own key: `texts`, `sounds` (kept recordings, only with `keep_voice`), `numbers` (counts, card checks and Compare's numbers; the checks' languages in `check-langs`, so older builds still read `checks`) and `keys` (one sealed key per service; a hidden run reads none). Models sit in the folder the user picks (`models_dir`), and the detector and the three check clips ship in the `voice` folder beside the worker (`VOICE_FILES`).
- **UI (`kx-ui`).** The hub turns `VoiceView` into `voice_json` for the `Voice` object, wakes only the meter on a level, and places the bar just above the keyboard, else at the bottom centre, else at the top, never over it (`[voice] place` can try the bottom or the top first). The bar and the Voice panel are ownerless, guarded windows that take clicks but never focus (ADR-0045), and the tray's Dictation row shows the keyboard, then the panel.
  - The panel's Cloud page lists the offered services with their marks; a card shows the row's facts with their date, and the key page its lines. The Models page has a Local | Cloud switch, and the bar shows a cloud mark with the service's name.
  - On the first dictation to a service with a warning, the bar widens and the warning wraps under its row in `warn_ink` (`VoiceWarn`); it never blocks the dictation.
- **Where the code is.** Each crate's voice parts, which the code map in ARCHITECTURE.md only names.
  - `keyxtend`: `voice.rs` makes voice typing's sealed folder and finds the voice files beside the worker. The `gate-voice` build (`--sound`, `played.rs`) hands a listen-only run's speech on, and its `played/` logs each stage and each cloud dictation's service or fault kind, never content.
  - `keyxtend-worker`: the streamed `voice` job (microphone, Silero, `transcribe.cpp`) and the `fetch` job (model downloads; ADR-0042). The `speech-net` job's `net/` holds one adapter per request shape, the WAV and Base64 writers, pacing and tries (ADR-0047, ADR-0051).
  - `kx-module-api`: `Speech`, `ModelFiles`, `SessionWatch` and the voice events and views (ADR-0042), `FolderReveal` (ADR-0046) and `CloudSpeech`, gated by `Network` (ADR-0047). Also `PasteText` and `TextPasted` (ADR-0048), and `VoiceLine`, `VoiceLineKey` and `VoiceClose` (ADR-0049).
  - `kx-ipc`: the voice and fetch frames, the `[voice]` ranges and the gate's `SOUND_FLAG`. Its `net/` holds `NetSend`, `NetTest`, `NetText` and `NetFault`, with the key and sound as secrets and replies capped, and `gate` the gate's log names.
  - `kx-platform`: the worker's streaming session (`worker/stream`), and `speech` and `models`, the voice and fetch hosts over an adapter's `Hardware` and `Disk`. `cloud` (`WorkerCloud`) is the relay, on its `kx-net` thread.
  - `kx-platform-windows`: the cores, memory, card drivers and models disk (`voice`), the session watch (`WinSessionWatch`, `session.toml`) and the models folder's reveal (`WinFolderReveal`, ADR-0046).
  - `kx-platform-fake`: `FakeSpeech`, `FakeModelFiles`, `FakeSessionWatch`, `FakeFolderReveal` and `FakeCloud`.
  - `kx-ui`: the bar and the panel (`VoiceBar`, `VoicePanel`), the Cloud page and a service's card (`VoiceCloud`, `VoiceCard`, `VoiceKeys`), and the bar's warning line (`VoiceWarn`).
  - `kx-theme`: `[caption]`, `[voice]`, `wave`, `run`, `voice_start` and `voice_stop`. Every theme adds `prep`/`prep_ink` (the amber Preparing light) and `warn`/`warn_ink` (card warnings, old facts) in part 2.
  - `kx-mod-voice`: the `kx-voice` actor runs the models table, device rule, scorer, dictation flow, History, counts, sealed store, models folder, downloads, Compare and the panel's state. Part 2 adds the services table, the cloud pick and its rules (`cloud`), the sealed keys (`keys`) and the typed lines (`line`).
- **Other platforms (P15).** The ports are neutral: macOS and Linux start the same jobs, fill `SessionWatch` from their own signals, use Metal or Vulkan for the card part, and fill the cloud POST with their own HTTPS stack under the same rules. `cpal` is a Windows-only worker dependency for now, so elsewhere the microphone answers `NoMic`.

## Spec §8.2 as built

Moved from the [modules spec](../superpowers/specs/2026-09-26-accessible-keyboard-design-modules.md#82-voice-typing) to keep it short: §8.2's first plan, kept as written and replaced by the 2026-11-10 designs, then where P11 settled or changed it. Section numbers are the spec's.

- **Mic key:** click to start, click to stop. There is no voice-activity auto-stop, so users who speak slowly or pause are never cut off.
- Distinct start and stop sounds, and a live waveform in a **caption bar at the bottom centre**, never over the keyboard.
- **Flow:** Listening, then Transcribing (shimmer), then the text is inserted into the focused field and shown in the caption with **Copy** and **Undo insert**.
- **Forced language setting:** Auto, English, French or Arabic. Auto can mis-detect Arabic, so forcing is recommended for Arabic.
- **Engines:**
  - Default: local **whisper.cpp** via `whisper-rs` (MIT).
    - Model `large-v3-turbo` q5_0 (574 MB), or `small` q5_1 (190 MB) for weak PCs.
    - Models are downloaded on first use and verified by SHA-256; they are not bundled.
    - A **benchmark button** tells the user which model their PC can handle.
  - Optional cloud, **bring your own free key**, off by default:
    - Groq (free tier: 8 audio hours/day);
    - Cloudflare Workers AI;
    - Azure Speech F0 (5 h/month; 18 Arabic locales, including Maghrebi dialects).
    - Gemini's free tier is offered only after a warning that the data may be used for training.
- Silence trimming before sending, to reduce Whisper's hallucinations on silence.
- **Transcript history:** kept locally and encrypted, 1 day by default. It can be exported later to build a personal dataset; this is opt-in.
- Runs in the **voice worker process**, the only process that may use the microphone and the network (§11).
- **As built (P11 part 1, 2026-11-16):** where the build settles or differs from the text above; the 2026-11-10 design holds the full rules.
  - The key reads Dictation · Dictée · إملاء. Click to start and click to stop stays; a `silence_stop` setting, off by default, can also stop after a silence (owner).
  - The bar sits just above the keyboard, else at the bottom centre, else at the top, never over the keyboard, and it takes clicks but never focus (ADR-0045). It offers Copy and Undo insert, as above.
  - There is no forced-language setting: the language is the keyboard's shown layout, fixed at Start and never guessed (ADR-0042, owner).
  - One engine, transcribe.cpp, runs ten models from one table, in place of whisper-rs. The defaults are Parakeet v3 for English and French and Cohere Transcribe Arabic for Arabic (ADR-0043, owner).
  - Models download on demand into a folder the user picks, each checked by its size and SHA-256; none is bundled (ADR-0043).
  - Models run on the processor, or on a graphics card through Vulkan once the card passes its check; Automatic takes the fastest checked card with room (ADR-0044). Under Automatic, a card that passed one model's check is checked by itself for each newly used model; a first check stays one click on Check or picking the card.
  - The benchmark button is Compare models: the user reads a set text once, each ready model writes it, and only the numbers are kept.
  - Silence trimming is the Silero voice detector, which cuts pieces at pauses and keeps `pad_ms` (300 ms) of silence around speech.
  - History keeps the last 50 texts (`history_max`, pins aside), sealed on disk with no time limit, not one day. Recordings are kept only with `keep_voice`, off by default.
  - The worker runs two jobs: `voice` hears the microphone and has no network, and `fetch` downloads models and has no microphone (ADR-0042).
  - The cloud engines are part 2 ([`2026-11-10-voice-cloud-design.md`](../superpowers/specs/2026-11-10-voice-cloud-design.md), ADR-0047).
- **As built (P11 part 2, 2026-11-20):** the cloud, in place of the "Optional cloud" lines above.
  - Cloud stays off until the user switches a service on and confirms its card; a cloud model is never a default, and nothing moves from local to cloud by itself (ADR-0047).
  - Offered: Groq, Lemonfox, Custom, OpenRouter, ElevenLabs, Gemini and AssemblyAI; OpenAI, Mistral, Together and DeepInfra are built but hidden (ADR-0050).
  - Not used: Cloudflare left with ADR-0047, OVHcloud by the owner's decision of 2026-11-17 (the part 2 plan's amendment), and Azure with ADR-0050.
  - Each service brings the user's own key, sealed on the voice shelf; KeyXtend ships none and makes no account.
  - Nothing leaves the PC before Stop. Then the recording goes, in pieces cut at pauses, to the one service named on the bar.
  - The worker runs three jobs: `voice` and `fetch` as above, and `speech-net`, which has the network and no microphone and sends the pieces over HTTPS in one of four request shapes (ADR-0047, ADR-0051).
  - Each card shows the service's facts with the day they were read; ElevenLabs and Gemini carry a training warning on the card and again at the first dictation.
  - A password box always uses the local model, and a failed upload keeps the recording, with Try again and "Use the local model".
  - A dictated text can also be pasted at once through the clipboard module (`insert`, ADR-0048); the panel's lines take typed text (ADR-0049).
  - A card serves a language only after its check passes on that language's clip (ADR-0052).
