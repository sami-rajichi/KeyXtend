# Voice typing — design (P11, part 1: on this PC)

Decided with the owner on 2026-11-09 and 2026-11-10, after two research rounds, a mock-up of the bar ([`design/voice-bar.html`](../../../design/voice-bar.html)), and a trial of ten models on the owner's own voice (numbers in [`docs/items/P11.md`](../../items/P11.md)). This file replaces spec §8.2 of [`2026-09-26-accessible-keyboard-design-modules.md`](2026-09-26-accessible-keyboard-design-modules.md) for everything that runs on the PC, and the roadmap's Phase 11 line. Part 2, the cloud services, is [`2026-11-10-voice-cloud-design.md`](2026-11-10-voice-cloud-design.md); part 1 is built and merged first. Every number below is a `[voice]` setting unless it says otherwise; one design serves Windows, macOS and Linux, and only the adapters differ. Lines marked **(designer)** are choices the owner has not made; the last section gathers them.

## Why

- Each letter is a click for a one-button user, so speech is the fastest way to write a sentence.
- Public scores pointed to Whisper. On the owner's voice, two newer models were far better (0 Arabic words wrong in 57 against 12; 1 French word wrong in 74 against 7 to 12), so the feature must offer many models and let each user find theirs.
- The look follows dictation bars people know: Wispr Flow, ChatGPT's and Gemini's microphone, and Windows' own voice typing.

## Goals, non-goals, anti-goals

- **Goals:**
  - one click starts listening at once, in the keyboard's language; one click stops; the text is typed into the app;
  - the wave follows the real voice, and the text shows only after Stop;
  - ten local models to start with: any model serves any language it knows, each language has a default, and a note names the best one;
  - the processor or a graphics card, by the user's choice, behind checks that never freeze anything;
  - engines and models are plug-in parts: adding, removing or updating one is a table row or one small adapter;
  - Compare models, on the user's own voice, inside the Voice panel;
  - a history of texts, an optional store of recordings, and usage counts for P13.
- **Non-goals in part 1:**
  - cloud services (part 2);
  - exporting kept recordings (F11);
  - voice commands (F2), translation or read aloud (F8);
  - a Settings window (P13 shows these settings; until then the Voice panel carries them);
  - macOS and Linux adapters (P15); the ports are neutral so they fit later.
- **Anti-goals (a failure even if it works):**
  - a bar or panel that takes focus from the app, or a keyboard that waits on the engine;
  - a guessed language, or text written word by word while the user speaks;
  - the microphone open while no bar shows it;
  - a transcript or a recording in a log, or dictation from a password box shown or kept;
  - anything sent to the network that the user did not ask for;
  - a model removed from the list because it is weak for one voice;
  - a value written in code that belongs in a table, a setting, a token or a translation.

## What the user sees

### The key, the tray row and the words

- The side key `mic` becomes a real key with its own role. A click starts a dictation; a click while listening stops it. The key glows red while the microphone is open, from a `rec` flag in the key's state (as P4 did for `dim`), no longer from the paint name.
- The tray menu gets a Dictation row above Quit (owner). It opens the Voice panel, showing the keyboard first if it is hidden, so the settings are reachable without starting a dictation (designer's reading).
- The key and its screen-reader name say the same word (owner): Dictation · Dictée · إملاء. The tray row follows the system's language, the bar and panel follow the keyboard's (ADR-0020).
- Main words, for the owner's check; the builder writes the rest in all three files:

| English | French | Arabic |
|---|---|---|
| Listening… | Écoute… | جارٍ الاستماع… |
| Transcribing… | Transcription… | جارٍ تحويل الكلام إلى نص… |
| Nothing heard | Rien entendu | لم يُسمع شيء |
| Stop · Cancel · Close | Arrêter · Annuler · Fermer | إيقاف · إلغاء · إغلاق |
| Copy · Undo insert · Try again | Copier · Annuler l'insertion · Réessayer | نسخ · تراجع عن الإدراج · إعادة المحاولة |
| Compare models · History | Comparer les modèles · Historique | مقارنة النماذج · السجل |
| Processor · Graphics card | Processeur · Carte graphique | المعالج · بطاقة الرسوميات |

### The bar

- A pill in the keyboard's theme, as drawn in [`design/voice-bar.html`](../../../design/voice-bar.html), the picture the owner saw: Cancel, the red dot and "Listening", the language, the wave, the timer, the gear, Stop. It replaces the style lab's older `.capbar`. Motion comes from `motion.toml` (`caption`, `shimmer`, `dot`), colours from `rec`, `rec_ink` and `rec_dot`.
- **Size (designer):** the bar is about one key row tall and scales with the keyboard; its round buttons are no smaller than the panels' buttons. `shape.toml [caption]` is cut again to the mock-up, `bar_px` included.
- **Mock-up first (designer):** the picture shows only the Listening state and the panel's first page. The plan's first screen task draws the other bar states and each panel page in the same file, in the three themes and in Arabic; the owner sees it before any QML. The owner approved the full mock-up on 2026-11-14: Stop stays dimmed while transcribing, Stop is dark (red stays on the key and the dot), and the panel opens above the bar.
- It is its own window that never takes focus but does take clicks. This changes ADR-0013's line that the caption is click-through; a new ADR records it. It has no owner window, like the notice in `App.qml`, so it stays when the keyboard hides; the guard watches it like a pop-up.
- **Place (designer):** just above the keyboard and centred on it; with no room there, the bottom centre of that screen. Never over the keyboard. `place` also offers the bottom or the top of the screen.
- The order is the same in every language: Cancel at the left end, Stop at the right end, the wave between them (ADR-0020 flips text only).

| State | Shows | Clicks |
|---|---|---|
| Listening | red dot and "Listening", the language, the live wave, a timer, the gear | Cancel, gear, Stop |
| Transcribing | the audio-lines icon, "Transcribing" and a shimmer | Cancel, gear |
| Text | the captions icon and the text typed | Copy, Undo insert, gear, Close |
| Note | one line saying what happened and what to do | the action it names, gear, Close |

- **The wave** shows the real sound level sent by the worker, smoothed over `wave_ms`; it rises with speech and lies flat in silence. With reduced motion it is a plain level meter (designer).
- **Text** stays for `done_ms`, then the bar fades; the pointer over it keeps it open. While the Voice panel is open the bar stays, and `done_ms` starts again when the panel closes (designer). A long text shows its first lines; the whole text is in History.
- **Notes** cover: nothing heard, no microphone, microphone in use or refused by the system, no model yet (with Download), the model does not know this language (with the gear), the model does not fit, and a worker fault. A note never hides the red dot while the microphone is open (P11 item note).

### The gear and the Voice panel

- The gear is on the bar in every state (owner) and opens the Voice panel, a normal keyboard panel (`PanelGlow`, paged lists of 5 rows, no scrolling). The tray's Dictation row opens the same panel.
- It opens on a short list of rows, as in the mock-up; each row shows its present value and opens its page.
- **Models:** the models that know the language in front, each marked Default, Ready, Download (with its size), or Not for this PC (with the reason). A note names the best model of the package for that language, for now Cohere's for Arabic. Tick a row, then: Use for this language, Download, Delete file.
- **Device:** Automatic, Processor, and each graphics card found, with its memory. See Device below.
- **Compare models**, **History**, and **Options** (the switches and steppers of the settings table).
- **Try again:** the recording stays until its bar closes (owner), so while the bar is open the panel offers to transcribe it again with another model or device. The new text replaces the first when Undo insert is still possible (undo, then type); otherwise it stays on the bar with Copy. History keeps the newest text only (designer).

## How a dictation runs

1. **Start.** The key asks the module to toggle; the tray row opens the panel instead. The language is the one the keyboard shows (`LayoutChanged`'s `iso`), fixed for this dictation; nothing is detected or guessed (owner). This also settles the spike's `front_layout` fallback: with no app in front, the keyboard's shown layout decides.
2. **Listening.** The worker opens the microphone and sends levels; the model loads in the background meanwhile, so listening never waits for it. A second click within `debounce_ms` is ignored.
3. **Pieces (designer).** A voice detector marks speech. Once a piece is longer than `piece_s`, it is cut at the next pause and transcribed in the background while the user keeps speaking. Each piece is heard whole, never word by word (owner), and nothing is shown before Stop. A piece's text goes to the module as soon as it is ready, kept secret and unshown, so a worker that dies costs only the pieces not yet written (designer).
4. **Stop.** The last piece is transcribed with the silence around it cut off, with padding (`pad_ms`). The pieces are joined and the text is typed at once (owner).
5. **Typing.** The keyboard stays the only typist: the module asks it to type, paced as the spike's `text_paced` was (`type_gap_ms`), followed by `type_suffix`. Undo insert erases exactly that text while nothing else has been typed since; after that it is dimmed (designer).
6. **Cancel** stops at once and types nothing. The bar says so and offers "Transcribe anyway" until it closes (designer); then the recording is wiped.
7. **Limits.** At `record_max_s` the dictation stops by itself and is transcribed, with a note. With `silence_stop` on, `silence_stop_s` of silence after speech stops it too; it is off to start (owner; this replaces the old spec's "never").
8. **Nothing heard.** After `nothing_s` without speech the bar hints at it and keeps listening; Stop with no speech types nothing.
9. **Password box.** When the field in front is a password (`FieldWatch`), the text is typed but the bar shows dots and offers no Copy, since clipboard history would keep it (designer); neither history nor recordings keep it.
10. **No field (designer).** If no field takes text when the text is ready, it stays on the bar with Copy, and in History.

## Parts

Engines, models and services are plug-in parts (owner): the feature's code names no model.

| Part | Where | What it does |
|---|---|---|
| Module | new `kx-mod-voice` | the dictation state machine on an injected `Clock`, the bar's and panel's views, settings, tables, history, counts; ports only |
| Port | `kx-module-api`, new speech port | a neutral service for a dictation run, model files and devices; it makes real the `SpeechToText` example of `ARCHITECTURE.md` |
| Host | `kx-platform` worker host | a new session kind beside "one ask, one reply": the worker also sends events on its own (levels, states); a new ADR |
| Frames | `kx-ipc` | new frame kinds for voice, each with a size range; text and sound travel as secrets; the crate stays a leaf |
| Worker jobs | `keyxtend-worker` | `voice`: microphone, detector and engine, no network. `fetch`: downloads, no microphone (designer) |
| Engine | inside the `voice` job | one small trait and one adapter per runner; today one runner, `transcribe.cpp` (MIT), for all ten models |
| Network | new `kx-platform-net` | the spike's WinHTTP part, reachable from the worker only |
| Windows | `kx-ui` | the bar window, the Voice panel, hub state and one bridge |
| Keyboard | `kx-mod-keyboard` | the `voice` role for the key, the `rec` flag, typing on request |
| Session | `kx-module-api`, new session port | tells a lock or a sleep, so a dictation stops and wipes (designer) |

- The tables live in `kx-mod-voice` and nowhere else. The worker holds no table: every ask carries the fields it needs (file, language code, device, threads), so a value is never written twice.
- A new model that an existing runner reads is one row. A new runner is one adapter file and its name in the `runner` column.
- **One model per job (designer):** a `voice` job loads one model, and its memory cap comes from that model's row. Changing the model (another language's model, Try again, Compare's next model) ends the job and starts a new one, so no job's cap ever changes and a model always leaves memory whole.
- A recording still needed by the new job crosses through the app as secret bytes in wiped buffers, in pieces under the frame limit. The app passes it on and never reads sound.
- **Listening without a model (designer):** the `voice` job can also record, cut and hand over its sound at Stop with no model loaded. Part 2's cloud dictation uses this.
- **Caps (designer):** a job without a model has a small cap of its own (`job_mb`); a model adds its row's need, and an unchecked file counts as its size.

### The models table

- One row per model: id, shown name, runner, file name, size, SHA-256, source address, licence and credit line, the languages it knows with the model's own code for each (one model writes `fr-FR` where another writes `fr`), memory needed on the processor and on a card, and whether it can write while hearing (recorded, unused).
- One list per language orders the models by the project's own tests, with their date. The first is that language's default and the one the note names. Today: Parakeet v3 for French and English, Cohere Transcribe Arabic for Arabic (owner).
- The rows at the start are exactly the files the trial read, all published by `handy-computer` on Hugging Face. Their sizes in bytes and SHA-256 are in the item file. Another cut of a model is another row, tested before it may become a default.

| Model | File | Size |
|---|---|---|
| Whisper Small | `whisper-small-Q5_K_M.gguf` | 194 MB |
| Whisper Large v3 Turbo | `whisper-large-v3-turbo-Q5_K_M.gguf` | 620 MB |
| Whisper Large v3 | `whisper-large-v3-Q5_K_M.gguf` | 1.16 GB |
| Parakeet TDT 0.6b v3 | `parakeet-tdt-0.6b-v3-Q8_0.gguf` | 740 MB |
| Canary 1b v2 | `canary-1b-v2-Q8_0.gguf` | 1.14 GB |
| Nemotron 3.5 ASR 0.6B | `nemotron-3.5-asr-streaming-0.6b-Q8_0.gguf` | 751 MB |
| Qwen3-ASR 0.6B | `Qwen3-ASR-0.6B-Q8_0.gguf` | 850 MB |
| Qwen3-ASR 1.7B | `Qwen3-ASR-1.7B-Q8_0.gguf` | 2.19 GB |
| Cohere Transcribe | `cohere-transcribe-03-2026-Q8_0.gguf` | 2.41 GB |
| Cohere Transcribe Arabic | `cohere-transcribe-arabic-07-2026-Q8_0.gguf` | 2.41 GB |

- None is shipped; each passes `kx-licence-check` as a user download before its row lands. The small Nemotron stays listed but is never a default (owner).
- **Source (designer):** a row points at its publisher's repository at a fixed revision, read when the row is written, and may list more than one address, so a mirror is one more entry. The SHA-256 is what is trusted.
- **Other languages (owner):** dictation works in every keyboard language a model knows. A language without its own list uses a general order from the same table, limited to the models that know it; the note then says the order is not tested for that language. If no listed model knows the language, the bar says so.
- **Other files (designer):** a model file in the folder that no row names is listed as "Other file, not checked". The engine reads its languages from the file itself, and it runs in the same sandbox. An older whisper.cpp file (`ggml-*.bin`) is such a file; the engine opened one in the trial.

### The models folder and downloads

- Models live in one visible folder the user picks (owner), asked at the first download; the offer is a `KeyXtend\models` folder on the drive with the most room, never a hidden cache (designer). A model file already there is recognised by its fingerprint and reused, not downloaded again.
- A download runs in the `fetch` job: HTTPS only, the address from the table, into a `.part` file, with progress, Pause and Cancel on the row. Size and SHA-256 are checked before the file gets its name; a wrong fingerprint deletes the part and says so. Room on the drive is checked first (`min_free_mb`).
- The Models page shows the folder, with Change and Open; after a change the panel reads the new folder, and files are never moved (designer).
- Clicking Download is the user asking for that network use; nothing else in part 1 touches the network.

### Device

- `device` is Automatic, Processor or one named card (owner: not CUDA alone; Vulkan on Windows and Linux, Metal on a Mac). The engine lists each device with its total and free memory.
- **Checks before a card is used (owner):** the model's card memory plus `card_margin_mb` must fit in the card's free memory at that moment; a first use warms the card up in the background with progress (23 to 36 s measured, once per model family, kept by the driver); a self-test must pass.
- **The self-test (designer):** the model transcribes a short speech clip shipped with the app, on the card and on the processor. It passes when the two texts differ by at most `card_diff` words and the card is at least `card_gain` times faster. There is one clip each in English, French and Arabic, six seconds at most, from a source `kx-licence-check` accepts; a model takes the first clip whose language it knows. The result is kept per card, driver version and model.
- Choosing a card, or one click on Check in the panel, starts the check in the background for the models in use; another model is checked when it is first picked. The processor serves until the check has passed.
- **Automatic (designer)** uses the processor until a card has passed its check; from then on it uses the best checked card that has room.
- A card that reports an error while loading or running falls back to the processor for the same recording, with a note; the check result is dropped so it runs again.
- The worker is built with the engine's loadable parts (`dynamic-backends`). The card part is a module file shipped with the app and loaded only by the worker, so the worker still starts on a PC without Vulkan. Users install nothing: graphics drivers bring Vulkan.
- The same build carries the processor variants, and the engine picks at start the one the PC's processor supports, so a processor without AVX still runs.
- On the processor, threads are capped at the real cores (`threads`, 0 means that cap): 16 threads ran 3 to 8 times slower than 8.

### Memory

- The model loads at the first dictation and stays for `idle_min` minutes after the last one (owner); then the `voice` job ends, and the system takes back all its memory, normal and graphics.
- Each dictation's working memory is freed as soon as its text is delivered (owner). Its sound is wiped when its bar closes, by Close, after `done_ms` or at the next dictation; that short stay is what lets Try again and "Transcribe anyway" work (owner). The job's end wipes any sound still held.
- One model is in memory at a time (designer).
- The job's memory cap comes from the model's row, plus `job_mb`. A model whose need passes the PC's memory is marked Not for this PC, with the reason; with no usable model the panel says so (owner).

## Compare models

- A button in the Voice panel (owner); it replaces the old speed-only benchmark button.
- It shows a set text in the keyboard's language (from the translations), records the user once, then runs each ready model that knows the language on the chosen device, one after another, with "model 3 of 6" and Cancel. Each model runs in a fresh job, so its memory figure is its own and one model's fault does not stop the others.
- Set texts exist for English, French and Arabic. For another language Compare shows each model's text, waiting time and memory, with no count (designer).
- The result is a paged table: model, words wrong out of the total, waiting time, memory. Tick a row, then "Use for this language".
- Scoring ignores case, punctuation and Arabic marks, and treats Arabic letter forms alike; the rules are a small table per language. The numbers are kept, with their date, for the panel and P13; the recording and the texts are not (designer).

## History, recordings and counts

- **History** is on to start (owner): the last `history_max` texts, each with its date and time, language and model. It is a paged, tick-then-act list: Insert (n), Copy, Pin, Delete; only Delete all asks first. Pinned texts do not count towards the limit (designer). A switch turns history off.
- **Keep recordings** is a second switch, off to start (owner): each dictation's sound is kept with its text and date, for the owner's later speech work. It is capped by `voice_max_mb`, oldest unpinned first (designer). A recording is kept as 16 kHz mono sound, the form the models hear, in pieces under the store's limit (designer). Export is F11's job.
- Both are sealed in a new `voice` folder of the data folder, with their own key, as clipboard history is (ADR-0010). A hidden run keeps nothing (ADR-0037).
- **Counts:** per day and per model and device, the number of dictations, seconds heard and seconds waited. Never content.

## Settings `[voice]`

| Setting | Start | Meaning |
|---|---|---|
| `on` | true | the whole feature |
| `models_dir` | empty | the models folder; asked at the first download |
| `picks` | empty | the user's model per language; empty means the table's default |
| `device` | `auto` | `auto`, `cpu` or a card's id |
| `threads` | 0 | processor threads; 0 means the real cores |
| `idle_min` | 5 | minutes a model stays after the last dictation |
| `record_max_s` | 600 | longest dictation |
| `piece_s`, `pad_ms` | 30, 300 | piece length before a cut at a pause; silence kept around speech |
| `nothing_s` | 5 | seconds before the "nothing heard" hint |
| `silence_stop`, `silence_stop_s` | false, 3 | stop after silence |
| `debounce_ms` | 400 | a second click this soon is ignored |
| `type_suffix`, `type_gap_ms` | one space, 10 | what follows the text; pacing of typing |
| `done_ms`, `wave_ms`, `level_hz` | 9000, 300, 30 | how long the text stays; wave smoothing; levels per second |
| `place` | `above` | `above`, `bottom` or `top` |
| `sounds` | true | start and stop sounds |
| `history`, `history_max` | true, 50 | text history |
| `keep_voice`, `voice_max_mb` | false, 1024 | kept recordings |
| `card_margin_mb`, `card_gain`, `card_diff` | 512, 1.5, 1 | room left on a card; how much faster it must be; words its text may differ by |
| `min_free_mb` | 2048 | room left on the models drive |
| `job_mb` | 512 | memory cap of a job without a model; a model adds its row's need |

Worker times (deadline, restart, load and warm-up waits) follow the pattern of `[clipboard]`'s worker keys. Each range is a named constant checked by the settings contract.

## Privacy and security

- The microphone opens only in the `voice` job, which has no network; `fetch` has the network and no microphone. Both run under the worker's restricted token and job limits.
- A recording lives in the worker's memory and is wiped when its bar closes, unless Keep recordings is on. It passes through the app only as unread secret bytes, to be sealed or handed to a new job. Text and sound cross the pipe as secrets and are never logged (`Redacted`).
- Model files are data, read only inside the sandboxed worker; no code is ever downloaded. Addresses come from the table.
- The engine's parts are loaded from the worker's own folder only, never from the models folder. One program holds both jobs, so a check keeps the `voice` job's code from the network part and the `fetch` job's code from the microphone part.
- The module asks for Microphone, Network (downloads only), SecretStore, ReadFocusedText and WriteClipboard (for Copy). `cargo xtask tidy` gains the engine and audio crates in the lists the app may not reach.
- `docs/security/kx-mod-voice.md` is new; the worker's, `kx-ipc`'s and the platform notes gain the voice parts.

## Edge cases for the plan

- No microphone; microphone in use; refused by the system's privacy switch; unplugged while listening.
- A 10-minute dictation; only silence; two quick clicks; Cancel while transcribing; Quit, lock or sleep while listening (stop and wipe).
- The layout changes while listening (the language stays); the app in front closes; a password box; no field.
- No model yet; the model's file vanished; the model does not know the language; a pick for a model no longer in the table.
- A bad fingerprint; a download cut off, paused, resumed or cancelled; a full or missing drive; a read-only folder.
- A keyboard language with no list of its own; a language no model knows.
- A model too big for the PC; a card with too little room now; a card that fails mid-run; the first warm-up; a driver update; a processor without AVX.
- The worker dies while listening or transcribing: never hang, say so, offer the pieces already written on the bar with Copy; the rest is lost.
- Try again when Undo insert is no longer possible; the job ended by `idle_min` while its bar is still open (Try again dims).
- A recording larger than one frame or one sealed file (a 10-minute dictation is).
- The keyboard hides while listening (the bar stays); a hidden run; history off; the limit of kept recordings reached.
- Arabic text on the bar; a text longer than the bar; a text longer than the keyboard's typing limit.

## Risks to prove first

The plan starts with these, before any screen is built:

1. The microphone works inside the worker's sandbox (restricted token, job limits).
2. A graphics card works inside the same sandbox, and the worker starts on a PC without Vulkan.
3. The size of the loadable parts (card module and processor variants) against the 80 MB download budget; the card part added 45 MB in the test. If the budget breaks, the owner decides.
4. The voice detector: a small model file run by the same engine, licence-checked, and its thresholds.
5. Events flowing from the worker without an ask, within the frame rules.
6. The cause of the window guard's z-order finding (P11 item note), before the bar adds a window.
7. G22 re-timed in weak mode, once the job caps each core.
8. The worker starts and transcribes with only the plainest processor variant, as on the roadmap's weak PC, which has no AVX.

- **A proof that fails stops the plan there (designer):** the builder brings the owner the options. No worker limit is loosened and nothing is cut without the owner's yes; a loosened limit also needs an ADR and a line in the security note.
- Risk 6 has a time limit set in the plan. If the cause is not found, the bar goes ahead under the same guard, the hands-on check covers it on the signed build, and the note stays open.

## Moving the spike (move map, P11 rows)

- Moves and is fitted to ports, settings, tokens and translations: `voice.rs` with its tests, `voicecfg.rs`, `voiceworker.rs`, `typer.rs`, `sendkeys`' `text_paced`, `layout.rs`, `screen.rs`' `bottom_centre`, `Caption.qml`, `Shimmer.qml`, the caption parts of `Extras.qml` and its test, the bridge `voice.rs`, `weak.rs`, `voiceproto.rs`, `lines.rs`, and the worker's `args`, `mic`, `resample`, `wav`, `worker`, `main` and `http`.
- Replaced, not moved: the worker's `local.rs` and `engines.rs` drove a separate whisper server over a local port. The engine now runs inside the worker behind the adapter, which is what lets one runner serve ten models.
- `cloud.rs` and `multipart.rs` move in part 2; until then they stay in the spike and keep building.
- Gates `g22`, `g22bench`, `g22type` and `wer` are aimed at the product. The caption's icons are proven in kx-ui's own Qt Quick tests.

## Documents this changes

- New ADRs from 0042: the voice session over the worker pipe and the two jobs; the engine runner, the models table and the defaults (replacing ADR-0007's engine lines); the models folder and downloads; the graphics card module and its checks; the bar that takes clicks (replacing ADR-0013's caption line). The builder may join some.
- `ARCHITECTURE.md` is split first (it is at 399 lines); spec §8.2, the roadmap's Phase 11 and the P11 name in `docs/FEATURES.md` follow this file.
- `THIRD_PARTY.md`: the whisper server leaves; the engine, its graphics parts, the detector, the three self-test clips and each model's licence arrive.

## Tests and checks

- Unit and property tests: the state machine, the piece cutter, the scorer in three languages, the tables (every row valid), settings, the history codec, and every new frame.
- A fake speech service in `kx-platform-fake`, so no test needs a microphone, a model or the network.
- Qt Quick tests for every bar state and panel page, by accessible name, in three languages.
- G22 on the product with the default models, normal and weak; the card path where a card exists.
- The manual Windows check with the owner, and the Definition of Done.

## Left to the builder

- Names of ports, events, frames, files and QML parts; how files are split under the size rule.
- Which Lucide icons to add and the two sound recipes, after `kx-licence-check`.
- The detector's thresholds, typing in pieces under the keyboard's limit, download resume, and how the folder is picked with one button (Quick-fill's `Places` is the model).
- Where check results and compare numbers are kept; behaviour on battery.
- The general model order for languages without a list, from the trial's table.

## The designer's calls

For the owner to check while reading: the bar's place; pieces cut at pauses; Undo insert's limit; Cancel's "Transcribe anyway"; Try again; the text kept on the bar when no field takes it; the two worker jobs; other files in the folder; the folder offer; Automatic's rule; one model in memory; what Compare keeps; pins in History; the recordings limit; the tray row opening the panel, not a dictation; the reduced-motion wave.

Added after the builder's check: the bar's size and the mock-up step before any QML; the bar staying while the panel is open; Try again replacing the first text; each piece's text sent at once; no Copy for a password box; one model per job; the models' source at a fixed revision; the card self-test and its three shipped clips; a failed proof stops the plan.

Added after the plan's check: the session port for lock and sleep; listening without a model; `job_mb` and the cap of an unchecked file; the folder line on the Models page; parts loaded from the worker's folder only.

The owner approved this design on 2026-11-10, and settled two of the check's questions: a recording stays until its bar closes, and dictation works in other keyboard languages, marked untested.
