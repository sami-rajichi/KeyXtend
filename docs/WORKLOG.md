# Work log

Newest first. One entry per implemented feature or fixed issue: a timestamp plus 3–4 sentences at most.

Older entries: [P8 to P10](worklog/p8-p10.md), [P4 to P7](worklog/p4-p7.md) and [P0 to P3](worklog/p0-p3.md).

## 2026-11-20 21:30 — P11: part 2's documents as built
The plan, the spec, the architecture pages, the security notes and the item notes now describe part 2 as built: seven offered services, all untested until the owner's check, four request shapes, sealed keys and the card check by language. ADR-0047 is marked superseded in part by ADR-0050 and ADR-0051, and the voice detail moved from `ARCHITECTURE.md` into its own architecture page. The cloud mock-up now shows AssemblyAI and no Azure, split into its data and its drawing.

## 2026-11-20 19:10 — P11: the spike's worker is gone
The P1 prototype's worker crate is deleted, with its cloud settings, now that the upload form, its multipart body and the WinHTTP upload all live in the product. The prototype workspace keeps only its core crate, without the five Windows features nothing used any more. The move map records each part's home.

## 2026-11-20 17:30 — P11: the card check reads the language in front
A graphics card is now checked on the clip of the language being dictated, the first time it serves that language, so a card that is wrong only in Arabic falls back to the processor. Each result counts for its card, driver, model and language alone. The results file keeps the form older builds read, with the languages in a file beside it, and an older build always lands on a failure if any language failed. ADR-0052 records this in place of ADR-0044's seventh decision.

## 2026-11-20 14:40 — P11: G22's cloud line and timing
The automatic voice test can now send its three public clips to every offered cloud service with a key on this PC. It borrows the sealed keys from the daily data folder, never opens or shows them, and deletes its copy at the end. Each passing language now prints how long each step takes, and `--device` picks the card without editing the tracked settings. The test refuses a card run without the card part and a reference sentence the voice credits do not give, and it no longer mistakes the bar's shadow for the bar.

## 2026-11-20 01:20 — P11: AssemblyAI
Voice typing can now send to AssemblyAI's Universal-3.6 Pro on its European servers, which do not train on the files, in one Sync request with the keyboard's language as one of its 32 codes. Its card shows $50 of free credit with no card needed and plain prices from $0.30 an hour, and says the first dictation tests the key, since this API has no call without sound. Every server fault except a paced 503 is retried before the network note, and the header that names the model can never repeat one the request already sends. ADR-0051 records this fourth request shape.

## 2026-11-19 23:06 — P11: Gemini
Voice typing can now send to Google's Gemini 3.5 Transcribe in one request, with the keyboard's language as a regional code (Arabic as ar-SA) and `store: false`, so Google stores no copy of the request. Its card warns that the free tier uses the voice to improve Google's products and that people may read it, says paid use is required in the EEA, Switzerland and the UK, and labels its prices and logging as the paid tier's. A step the user must take on a service's site, such as a verification, a payment method or a spending cap, now has its own note instead of "no credit left", and a used-up free day says so without guessing when it renews. A key test that fails now names its reason; only a missing answer points at the network.

## 2026-11-19 18:41 — P11: ElevenLabs
Voice typing can now send to ElevenLabs' Scribe v3 and v2 in its own form, with no sound tags such as "(laughter)", and its refusals are read by their exact codes, so used-up credit is never taken for a bad key. Its card warns that it trains on the voice by default and names the switch that stops it; the first dictation to any service with such a warning shows it again as a line on the bar, holding nothing back. The card prices light and heavy use at the cheapest plans that cover them ($6 and $22 a month) and says the free plan is for non-commercial use only, with attribution. The services table is split into the upload-form rows and the rows with their own shape.

## 2026-11-19 12:11 — P11: only the services the owner can test are offered
Azure is dropped, because it has no free path and needs a card; its MAI-Transcribe-2 model stays reachable through OpenRouter, and Azure's unused connection code is removed. OpenAI, Mistral, Together and DeepInfra are hidden for the same reason, while their tested code stays, so one can be switched on after a real test. A hidden service cannot be shown, picked or sent to, and an older saved setting that names one falls back to the local model. ADR-0050 records that only services the owner tests by hand are offered.

## 2026-11-19 11:09 — P11: six more cloud services
Voice typing can now send to OpenAI, Mistral, Together, Lemonfox, DeepInfra and OpenRouter, each built from its row in the services table, with the fields, language codes, limits, prices and terms read from the services' own pages. A refused key, a missing payment method and a country the service does not serve each show their own reason, and OpenRouter's keep-nothing switch marks a model without such a host as unavailable instead of moving the dictation elsewhere. Arabic prices now agree with their number, as in "دولار واحد", "من دولارين" and "15 دولارًا". Values no page states are marked untested for the final check with real keys.

## 2026-11-18 20:25 — P11: type a key, address and model on our keyboard
The Voice panel now joins the keyboard's panels, so a service key, a Custom address and an "Other model" id can be typed on our own keyboard, and the keys go to the panel only while one of these lines shows. The key shows only as dots, lives only in the voice module, and is wiped on Save, Cancel, leaving the page or closing the panel, and a test confirms that no screen and no log line holds it. Each line has its own Save, which screen readers name as "Save the key", "Save the address" or "Save the model", and a press on a line picks it with one key sound, also when Narrator presses it. ADR-0049 records that the panel takes keys only while a line shows.

## 2026-11-18 16:32 — P11: the cloud pages drawn in three languages
The Voice panel now draws the Cloud page, each service's card, the key and address lines, Compare's send warning and the Models page's Local | Cloud switch, in the keyboard's theme and in English, French and Arabic. Every page fits the panel at both sizes with no scrolling; a card, or a key page with an address, takes two pages. A service's own setting name ends its sentence after a colon, so it reads correctly in Arabic, and old facts or a refused key show in the warning tone. The bar names the cloud service while it works and says why when it cannot, and "Use the local model" dims when no local model is ready.

## 2026-11-18 08:27 — P11: the app connects cloud voice typing
The app now gives the voice module its cloud speech service, which the module requires, so a cloud dictation can reach the chosen service. Tests pin that only the voice module gets the service and the key, that the keyboard and the screens never see a key, and that a hidden run reads no key. The app itself still links no network code, and the staged build is 67 MB of its 80 MB budget.

## 2026-11-17 23:40 — P11: the Voice panel's cloud side
The Voice panel now has a Cloud page, Groq first, with one card per service: its dated facts, a link to where the key is made, and lines for the key, an address or a model id. Paste puts a key on its line as dots, seals it, and KeyXtend's clipboard history forgets that copy; switching a service off keeps the key, and only Remove key deletes it. The Models page gets a Local | Cloud switch, and Compare sends the recording only to the cloud models the user ticked. A clipboard answer now names the module that asked, so a vault password and a cloud key can never cross.

## 2026-11-17 21:15 — P11: a dictation can go to a cloud service
With a cloud model picked, the bar names the service from Start; the speech stays on this PC until Stop and then goes, cut at pauses, only to that service. After any fault the recording is kept and the bar offers Try again or "Use the local model", and nothing is typed twice. A password box, or a take the local model has written, never goes to the cloud, and a removed key or a switched-off service stops the upload with a note.

## 2026-11-17 18:59 — P11: the cloud services table and sealed keys
The voice module now has a table of cloud services, checked when it loads, with Groq first and a Custom row for a server of the user's own; each row holds its address, limits, models and its dated facts. Each service's key is sealed on its own on this PC and is never in the settings, a log or a test file. A service stays off until the user confirms it, a newer terms number asks again, and a property test shows that no change of settings ever moves a language from a local model to a service by itself.

## 2026-11-17 18:58 — P11: the worker sends speech to a cloud service
A third worker job, `speech-net`, sends each piece of a dictation to the chosen service over HTTPS with the user's key in its header only; it follows no redirect and caps every reply. The app hands it the pieces at Stop, cut at pauses and never mid-word, several at a time within the service's limits, and Cancel stops an upload between its chunks. A rate limit waits once before it is reported, and no cloud request is ever read as "no credit" because of words in its message.

## 2026-11-17 18:10 — P11: dictated text can be pasted at once
A new Options row, "Insert the text", offers Type it (the default) or Paste at once. Paste at once goes through the clipboard history's paste route, keeps the text out of both clipboard histories, and then puts the previous clipboard back. It types instead in a password box, when a password manager marked the clipboard private, after a copy from a skipped app, or when the clipboard cannot come back whole. A new decision record (ADR-0048) explains the route.

## 2026-11-17 16:10 — P11: the groundwork for cloud voice typing
The plugin API now has the `CloudSpeech` port, which only a holder of Network may use, with the faults a service can give and the panel's cloud views. A fake cloud service lets tests script texts and faults. Checked frames now carry a cloud request from the app to a third worker job, with each recording piece at most ten minutes, replies capped, and numeric or non-HTTPS addresses refused. No sound or key leaves the PC yet; the sending job comes next.

## 2026-11-17 12:40 — P11: the Dictation key glows amber until the microphone opens
After an idle spell the bar showed while the microphone was still starting, so the first words could be lost. Now no bar shows until the microphone is open: only the Dictation key glows amber, at once, with a Narrator name saying it is preparing, and the bar then appears in red. A worker test proves that speech heard while the model still loads is all written, and Compare models waits for its microphone the same way. The themes' tokens moved into one file per theme to keep each file small.

## 2026-11-17 07:20 — P11: the gear ends a running dictation before the panel opens
The bar's gear and the tray's Voice typing row opened the panel while the dictation kept recording. Opening the panel now first ends a preparing, listening or transcribing dictation exactly as Cancel does, so nothing is sent, transcribed or typed, and Transcribe anyway still works. The gear also ends a Compare models recording as Compare's own Cancel does, and flow, actor and property tests pin each case.

## 2026-11-16 21:40 — P11: the fetch tests' folders no longer race on Windows CI
Windows CI failed a fetch test with "Access is denied" while making its folder. Each test deleted the shared parent folder when it looked empty, and Windows refuses a new folder inside one being deleted; a probe on this PC showed the same error. Each test now has its own folder in the profile folder and deletes only that, and the fetch tests passed 100 runs at 64 threads.

## 2026-11-16 18:25 — P11 part 1: gates and the owner's check passed
G22's bench, staged edges and product runs on the processor and the card passed, with the box proofs and `g19 hung`, and the owner passed all 20 hands-on checks ([test report](test-reports/2026-11-16-p11-voice-typing.md)). The last card run's slow English and French texts stay open for part 2. At the owner's yes, the voice trial with their clips, the speed test, the research files and the old Whisper models were deleted; the ten speech models stay for part 2's gates.

## 2026-11-16 17:50 — P11: History's recordings stay within `voice_max_mb` at a later text
History's property test found that a later text of the same take kept its recording past `voice_max_mb`, when that recording was unpinned after the limit fell. A kept text now drops recordings oldest first and its own last, as `admit` already refuses one past the room left; the text itself always stays. A unit test and the saved property case cover it, and every property test of the changed crates passes at 2,000 cases.

## 2026-11-16 17:00 — P11: a fresh voice job's first answer gets the load wait
G22's card runs at times lost a dictation as `Late`, since a fresh voice job must answer Start within `ask_ms` (3 s) though it first loads the engine's parts and the detector. The bench now prints that first answer's time, 1.0 to 2.3 s on an idle PC. `Wait::Shift` gains a wait of its own for the first note, which a Start takes from `load_s`; each later note keeps `ask_ms`. The Listening bar's time and its nothing-heard hint now count from the microphone's opening.

## 2026-11-16 15:20 — P11: a dictated text no longer misses a box it was told late
G22's card runs at times typed nothing, as the voice module's last focus report said "no box" while target-window's box held focus. The field port gains `look`, which reads the box in front again on the watch's thread, its kind and id only, and the voice module looks once before a written text when no box was told. G22 now prints, for a failed language only, the copy's marked log lines, the windows that came to the front and the bar's button states.

## 2026-11-16 07:00 — P11: review and security
The saved review notes are fixed: a shared reserved-name check, a damaged notice per kind, dictated texts held as wiped secrets in the bar, History and Compare, the host's thread count, one weak-PC section for the gates, and no machine paths in tracked files. Downloads may follow Hugging Face's redirects under `hf.co`, while every layer makes a link name its host exactly. The branch code review's three host-rule findings are fixed, the security review found no high or medium issue, and each P11 security note now carries its final status.

## 2026-11-16 04:30 — P11: the docs for local voice typing
A new voice section in the architecture docs covers the events, ports, module, worker jobs, hosts, engine parts, files and UI, and the code map, crate READMEs and security notes now match the code as built. The spec's §8.2 gained an "as built" list, the roadmap and README name the voice bar and the transcribe.cpp models, and THIRD_PARTY marks the spike's Whisper tools removed. P11's item file lists what part 1 built, and its findings for P13, P15, F2, F8 and F11 sit in their own files.

## 2026-11-16 03:30 — P11: the spike's voice code removed
The spike's voice code is gone now that the product holds it: its voice core, worker, typer, weak job, caption, mic button, voice settings and the unused note helper. The spike's worker is now a small library that keeps only the cloud parts part 2 builds from, under a `[cloud]` section. The move map marks each voice row as moved, with its home in the product.

## 2026-11-16 02:20 — P11: the voice gates
G22 now runs on the product's test build: for English, French and Arabic it clicks Dictation in our test window, a gate worker plays the shipped clip, and it checks the words, the focus and the bar's place, printing numbers only. `g22 staged` proves the staged copy with no card part, with only the processor floor, and with a library planted in the models folder, and `boxed` proves HTTPS and a second-drive folder under the worker's restricted token. The sound flag lives in kx-ipc, the app's `gate-voice` build and kx-gates' `voice` feature hold it, and the stage and dev-install refuse any program that does. The gates no longer use the spike's voice code.

## 2026-11-15 23:55 — P11: the voice panel's pages
The Voice panel follows the approved mock-up: home, Models with downloads, Pause, Cancel and the folder line, devices with their memory, Compare, History and Options, each list paged at five rows where a click ticks a row and the footer acts on the ticked ones, with switches and steppers instead of sliders. It opens above the voice bar, or below it when there is no room, never over the keyboard, moves when the bar moves or grows, and takes clicks but never focus. A language whose default model is too big for the PC says on the Models page that local models will be very slow here; French names a language in lower case inside a sentence, and the bar's chip names it as a title. History's Copy and a multi-row Insert join texts with the new `history_join` setting, and the panel gets its rows as soon as it first opens.

## 2026-11-15 19:56 — P11: the voice bar, its tokens, icons and sounds
The floating voice bar follows the approved mock-up: a wave that follows the voice level (a plain meter under reduced motion), the red dot and timer, Cancel left and Stop right in every language, Stop dark and dimmed while transcribing, placed above the keyboard or else at the bottom centre, taking clicks but never focus. Its sizes, motion, icons and colour pairs are theme tokens, and two short chimes from our own recipes mark the start and end of listening when the voice sounds are on. The bar now says when Copy failed and when typing stopped part way, and the Dictation key's light reads its own flag. A review matched the wave height and the notes' weight to the mock-up, moved the timer's format into the texts and moved the notice window out of App.qml.

## 2026-11-15 16:23 — P11: voice typing in the app and its staged engine
Voice typing is now a default part of the app: it gets the speech, model-file and session services and its own sealed shelf, kept only in memory in a hidden run, and a life test boots every shipped module to show only voice takes that shelf. The install carries the worker built with its graphics-card part and without the gate, the engine's parts with the plain processor part required, the voice detector, the three check clips and their notices, at 66 MB of the 80 MB budget. Tidy keeps the network crate out of the voice job and the audio and engine crates out of the download job, and CI installs the Vulkan SDK from a pinned, hash-checked download and tests the worker's gate build. A review fixed prediction ending the word when the same layout was told again, which could drop a typed start as voice typing started (P7's open note).

## 2026-11-15 13:15 — P11: the voice module's actor
The voice module now runs: the Dictation key drives the flow and its light follows the microphone exactly, speech runs and every save happen off the actor's thread, and one typer thread types each text. The panel is drawn once per change, its clicks reach downloads, Compare, History and the folder reveal, and a failed save shows a note and is retried at stop. A panic hides the bar, closes the microphone and darkens the key, and with the module off nothing starts. A review stopped a stopped typer from typing queued texts, a model being fingerprinted again on every rescan, a dictation hung by a panicking port, and unretried count saves.

## 2026-11-15 10:54 — P11: the voice ports' follow-ups and the models folder reveal
The speech port's devices now carry their cores and whether a card is built-in graphics, a model that does not fit tells free memory beside the total, clip languages come from the speech port and number options carry their unit, so four stopgaps are gone. Compare marks a model that failed and runs the next, the bar gains notes for a name another file holds and a file still being checked, and the Models sheet knows when local models will be very slow on this PC. A new gated service that only the voice module holds (ADR-0046) shows the models folder in Explorer. A review made it accept only a held folder on a fixed local drive with no link from the drive down, and use a verb that runs no file.

## 2026-11-15 08:29 — P11: G22's weak-mode bench and the worker job's processor cap
`kx-gates g22 bench` reads each language's shipped check clip through the product's worker and speech host with its default model, on this PC, then in weak mode: a worker copy with only the plain processor parts, in a job capped at 2 cores at 30 % each before it starts, with 4 GB for the fit rule. It prints numbers only: Parakeet waits 0.3 s here and about 21 s for a 6 s clip in weak mode, and Cohere Transcribe Arabic does not fit 4 GB. The owner decided weak-mode speed is a reading, as the roadmap says, and Parakeet stays the default. A review moved the cap before the worker starts, dropped AVX from weak mode and made each take count the sound heard, not the clock.

## 2026-11-15 03:39 — P11: the hub's voice side and every voice text
The UI hub now turns the voice module's bar and panel views into JSON for QML, sends clicks back with the round QML drew, and places the bar the panel gap above the keyboard, else at the bottom centre, never over it, as the `place` setting asks. The tray's Dictation row shows a hidden keyboard first, then opens the panel, and every voice text exists in English, French and Arabic with correct plurals. A review fixed Arabic seconds and milliseconds, an "In use" mark on a model not yet downloaded, a text bundle rebuilt on every read, and a hub file past 300 lines.

## 2026-11-15 01:43 — P11: the speech, model files and session hosts
The app now serves the `Speech`, `ModelFiles` and `SessionWatch` ports over the worker's `voice` and `fetch` jobs: one job per model with its memory cap, `idle_min` on the clock, the recording handed to a new model in wiped pieces, devices, facts and card checks in jobs of their own, and one fetch job per download with room checked first. On Windows the worker starts with an allow-listed environment and its own folder, real cores and job peaks come from the system, a hidden non-activating window tells lock and sleep, and a new models folder is created with a protected rights list in one step. A review fixed a keep that could hang under the job's lock, an uncapped keep, a fingerprint that followed links or read a changing file, and rights that could land on another program's folder.

## 2026-11-15 00:19 — P11: the models folder, downloads, Compare and the Voice panel's state
The voice module now knows the models folder by fingerprint, offers `KeyXtend\models` on the drive with most room, refuses relative, device and climbing paths, and runs one download at a time with a note for a full drive, a read-only folder or a taken name. Compare records once and writes it with each ready model in a fresh job, keeping numbers only, and borrows the microphone through the flow so it never meets a dictation; the panel builds every page in 5-row pages with tick-then-act lists and Options checked by the settings' own ranges. A review fixed 15 points, among them a stale fingerprint cache after a folder change, a card granted to an unchecked file, re-checks after a driver update and stale panel clicks.

## 2026-11-14 23:17 — P11: the worker's Linux lint without the engine
The worker's Linux lint without the engine failed on sound parts that only the Windows microphone and the gate's sound file reach. They now carry the same scoped `expect(dead_code)` as the engine-only parts, and all six lint builds pass on Windows and Linux.

## 2026-11-14 23:10 — P11: the worker's fetch job
The worker's `fetch` job downloads a model file through `kx-platform-net` into `<file>.part`, resumes from the part's length, stops reading at the row's size, and names the file only after its size and SHA-256 pass and the part is flushed. Pause keeps the part, Cancel or a bad size or fingerprint deletes it, a stall past `stall_s` is a fault, and addresses are tried in table order, the next one taking over from the part's length. A review made it refuse a name already taken, a link at the part's name and a path that climbs out of the folder, and tell the first fault when no address serves.

## 2026-11-14 22:54 — P11: the voice history, kept recordings and counts
The voice module keeps a history of texts with pins, a limit that never drops a pin and a Delete all that always asks first, plus kept recordings in blobs under `voice_max_mb`, counts per day, model and card, card check results per card, driver and file, and Compare's numbers, never texts. Each is a versioned sealed blob that refuses a damaged or newer file without a panic, and history off, a password box or a hidden run keeps nothing. A review stopped the sound being asked for when it would not be kept, kept one recording per take, made Undo dim after the user stops a dictated text, and made a failed card drop only that model's check.

## 2026-11-14 22:38 — P11: the keyboard's Dictation key and dictated typing
The `mic` key now has the `voice` role: a click publishes `VoiceKey` and keeps every latch, and `VoiceLit` lights it red through a `rec` flag and paint. A `TextAsk` erases at the keyboard's `erase_gap_ms`, types at its `gap_ms` by the route a letter takes, splits anything past `text_cap` without cutting a character, leaves latches as they were, and answers `TextDone` with its count after one prediction `Break`. The user's own key, a clicked word or a click elsewhere stops it; a waiting dead key refuses it whole, and the UI fixtures, QML tests and G10 now treat Dictation as a live key that gates never press.

## 2026-11-14 22:11 — P11: the worker's speech engine, voice detector and graphics cards
The worker's `voice` job now writes text through `transcribe.cpp` on one engine thread while listening goes on, with Silero marking speech so pieces are cut at pauses, and lists the processor, cards and built-in graphics. A card short of room, failing to load or failing mid-run never ends a dictation: the same piece runs on the processor, and the self-test reads one clip on the card and the processor for the module to score. The engine loads its parts only from the worker's folder and refuses to start beside an engine path variable; text copies are wiped, and Try again cuts a long recording in steps so Cancel answers at once.

## 2026-11-14 21:41 — P11: the dictation flow
The voice module's flow is a pure state machine (Idle, Listening, Transcribing, Text, Note) that takes clicks, speech notes, the box's kind, lock and sleep, layout changes and clock ticks, and gives back runs to open, stop or close, the bar's view, the key's light, text to type and history to keep. The language is fixed at Start, every timing comes from `[voice]`, a click on an older bar is dropped, and every close wipes the recording. A review fixed seven bugs, among them a password re-read by Try again showing in plain, and pieces lost when the microphone was unplugged; a property test checks the light, the closes and the dots.

## 2026-11-14 21:04 — P11: the worker-only download crate `kx-platform-net`
`kx-platform-net` gives the worker's coming `fetch` job its `Http` port and a WinHTTP adapter: HTTPS only, every host and every redirect checked against the row's list before it is asked, at most five redirects, and resume by byte range with a 206 that must start where asked. Every wait is held within `stall_s`, cookies and WinHTTP's own redirects are off, TLS is 1.2 or 1.3, and errors carry fixed reasons, never an address. A review added length caps on joined addresses, dot-folder and fragment rules, and a sweep of tricky addresses; the crate joins CI's Linux list.

## 2026-11-14 20:45 — P11: the voice module's models table, device rule, scorer and settings
`kx-mod-voice` holds the ten-row models table with fixed-revision links, sizes, fingerprints, memory and language codes, and an order per tested language from the 2026-11-14 trial, with a general order for the rest marked untested. Automatic keeps the processor until a card passes its check, then takes the passed card with room and the best gain, and says why on every fallback. The scorer counts words wrong per language and keeps Arabic letter forms apart; `[voice]` settings repair a bad key alone, and a bad pick resets only its language with a notice.

## 2026-11-14 20:12 — P11: the worker's voice job records, cuts and hands over sound
The worker gains the `voice` job's capture side: the microphone through `cpal` behind a `Source` trait, resampling to 16 kHz mono, levels at `level_hz`, and a cutter that cuts only at a pause after `piece_s`, keeps `pad_ms` and loses no sample. With no model it records and hands its sound over at Stop; Cancel keeps the recording until Close, so "Transcribe anyway" works, and every sound buffer is wiped. The microphone closes on every ending, a panic included, and the queue from the microphone is bounded by `record_max_s`.

## 2026-11-14 19:49 — P11: fakes for speech, model files and the session
`FakeSpeech`, `FakeModelFiles` and `FakeSessionWatch` script levels, pieces, faults, a worker death, devices, card checks, download steps, and lock and sleep, and record every ask for the module's tests. Each checks asks with the port's own rules, and no note reaches a run after it closed, died or was dropped. They start no threads and never sleep, so the module's flow tests stay exact.

## 2026-11-14 19:21 — P11: the worker's streaming session
Beside one ask and one reply, the worker host now keeps a streaming session: asks with deadlines, plus notes the worker sends on its own, told in order on one thread. A late note, a bad or stray frame, or a quit ends the worker and holds the next start back from `restart_ms` doubling to `restart_max_ms`, and a worker holds at most two unread frames, so a flooding worker cannot grow the app's memory. A property test checks one worker at most, no note after a loss and none left running after drop.

## 2026-11-14 19:16 — P11: the voice module's API
The module API gains the `Voice` capability, the `Speech` port (gated `Microphone`), the `ModelFiles` port (gated `Network`), the ungated `SessionWatch` port and the voice events, with every value range taken from the frames. The view types draw each bar state and Voice panel page of the approved mock-up, and a password box can never offer Copy. Paths refuse control characters and device names such as `NUL.gguf`, and texts, sounds, paths and the user's own model file names never print.

## 2026-11-14 17:50 — P11: `kx-gates boxed`, and G19's hung step alone with z-order snapshots
`kx-gates boxed` starts a probe under the worker's own token and job; it proved that the microphone and the graphics card work inside the box. `g19 hung` runs G19's hung step alone and snapshots the whole z-order after the hang, after the close and at a cover timeout, then lifts the keyboard from outside to test the fix path. Both read classes and numbers only, never a title or text.

## 2026-11-14 17:45 — P11: the voice and fetch frames
The worker pipe gains the `voice` job's asks and notes (kinds 19 to 40) and the `fetch` job's (41 to 48), each kind with its own size range, and the `[voice]` value ranges now live once in `kx-ipc`. Text, sound and paths are wiped secrets that never print, and a device name with a hidden direction mark is refused. Round-trip, refusal and property tests cover every frame, and the vault frames are unchanged.
