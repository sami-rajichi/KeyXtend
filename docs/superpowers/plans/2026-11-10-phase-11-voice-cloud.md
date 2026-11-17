# Phase 11, part 2: voice typing through cloud services (v0.7)

## Goal
- The user may pick, per language, a cloud service and model beside the local ones, with their own key. The bar and everything after Stop stay the same, and nothing leaves the PC before Stop.
- Design: `docs/superpowers/specs/2026-11-10-voice-cloud-design.md` (approved 2026-11-10; replaces the cloud lines of spec §8.2 and of ADR-0007); `docs/items/P11.md`. It builds on part 1, `2026-11-10-phase-11-voice-typing.md`, which must be merged first.
- Amended 2026-11-17 with the owner's notes of that day: the gear ends a running dictation, a Preparing state before listening, and the services' facts read live during the build.

## Scope
- In: ADR-0047; the `CloudSpeech` port, its host and fake; the `kx-ipc` net frames; the worker's `speech-net` job with four request shapes (owner, 2026-11-17: OpenRouter and DeepInfra now take the upload form, Gemini one request); the services table and its dated facts; sealed keys; the Cloud page, the service cards, the typed lines and the bar's cloud mark; the settings added; G22's cloud line; the spike's last voice files removed.
- In (2026-11-17): the gear ends a running dictation before the panel opens; the Preparing state on the bar and the Dictation key; a timing line per stage in G22, for part 1's slow card texts.
- Out: a shipped key or an account made for the user; live transcription; services that need three or more requests, other than Gemini; NVIDIA's trial, Alibaba direct and Cohere's trial key; facts refreshed over the network.

## Design notes
- **Crates:** no new crate; touched: `kx-module-api`, `kx-ipc`, `kx-platform`, `kx-platform-net`, `kx-platform-fake`, `kx-mod-voice`, `kx-mod-keyboard`, `kx-ui`, `kx-theme`, `apps/keyxtend`, `apps/keyxtend-worker`, `tools/kx-gates`.
- **The gear (owner, 2026-11-17):** opening the panel, from the gear or the tray, while a dictation prepares, listens or transcribes first ends it as Cancel does: nothing is sent, transcribed or typed. The bar keeps Cancel's note with Transcribe anyway, as part 1 does.
- **Preparing (owner, 2026-11-17):** `VoiceBar::Preparing` shows from Start until the microphone's `Open` stage, with a short text and its own Narrator name; then listening and the red glow begin. `VoiceLit` gains a preparing light, drawn amber on the Dictation key. A model still loading is never an error; only the job's existing `load_s` wait ends in a note.
- **Live facts (owner, 2026-11-17):** the services table takes the facts read on the services' own pages on 2026-11-17, and they are read again on the day of the manual check.
- **ADR-0047:** the third job `speech-net` (network, no microphone); the relay at Stop through the app as unread secret bytes; the shapes and the services table; keys sealed in `<data>/voice/` (replaces ADR-0007's secret-store line); terms numbers and asking again; the capabilities below.
- **Capabilities:** `Network` now also covers the services the user switched on. The voice module becomes a holder of `OpenLink` (the card's link, beside ADR-0041's selection module) and of `ReadClipboard` (Paste on a typed line, through the `ClipTake` route Quick-fill uses).
- **Port `ports/cloud.rs`** (`CloudSpeechKey`, gated `Network`): `CloudSpeech::send(CloudAsk)` takes the row's fields (shape, address, key header, model, language code, limits), the key as a `Secret` and the pieces. It gives each piece's text, or a `CloudFault`: key, credit, allowance with its renewal, rate, too big, language, empty, strange reply, network, timeout. `test` says whether a key works; dropping the run cancels it.
- **Listening:** a cloud dictation opens part 1's `voice` job with no model (`job_mb` cap), asked to cut within the service row's limits. A password box at Stop hands the sound to a new job with the local model, as any model change does.
- **Events:** `VoiceView` gains the Cloud page, a service's card and the bar's cloud mark with the service's name. `VoiceAct` gains the `cloud` switch, switch on, confirm, key set (a `Secret`), Test, Remove key, set address, set model, the keep-nothing switch, open the card's link and "Use the local model". Keys, addresses and model ids are typed on the panel's own line (`PanelLine`, ADR-0040).
- **Frames:** `kx-ipc/src/net*.rs`: `NetSend` and `NetTest`, with the notes `NetText` and `NetFault`; the key and the sound are secrets; a reply's size is capped in `limits.rs`.
- **Worker:** `Job::SpeechNet` in `apps/keyxtend-worker/src/{net.rs, net/*.rs}`. Trait `Shape` builds a request from the row's fields and reads the reply into text or a fault. One file per shape: `form.rs` (from the spike's `cloud.rs` and `multipart.rs`), `azure.rs`, `eleven.rs`, `gemini.rs`; `wav.rs` wraps a piece. A form row names its language field and whether it goes as a list (OpenAI's `languages[]`), and may add fixed fields such as OpenRouter's keep-nothing `provider` JSON. HTTPS only, through `kx-platform-net`, with no redirect to another host. Tidy keeps `net` files from the audio crate.
- **Host:** `kx-platform/src/cloud/` takes the pieces from the `voice` job at Stop and feeds `speech-net`, several at a time within the row's limits, in wiped buffers, with `net_wait_s` and `net_tries`. The job's cap is `job_mb`.
- **Module table:** `kx-mod-voice/services.toml`, one row per service: id, name, shape, address or "typed", key header, where the key is made, limits, up to ten models with their languages and codes, the facts as numbers and flags with their date and source, the warning, the terms number, and whether it is offered. Rows at the start, Groq first: Groq, OpenAI, Mistral, Together, Lemonfox, Custom, OpenRouter, DeepInfra, Azure classic and MAI-Transcribe-2, ElevenLabs, Gemini (not offered until its check passes). OVHcloud is dropped (owner, 2026-11-17). Facts and request fields come from the services' own pages read on 2026-11-17 (the SDD workspace's `research-cloud-2026-11-17.md`).
- **A card's sentences** are translations filled with the row's numbers, so no price or claim is written in code or twice.
- **Store:** `<data>/voice/keys/`, one sealed blob per service; a hidden run reads and writes none.
- **Settings `[voice]` added:** `cloud` false; `services` {} (each switched-on service with its confirmation date and terms number, the Custom and Azure addresses, typed model ids, and OpenRouter's keep-nothing switch, on to start); `net_wait_s` 60 (5–300); `net_tries` 2 (1–5); `facts_days` 180 (30–730). A `picks` entry may now name a service and model.
- **Tokens:** `icons.toml [voice]` gains cloud (bundled) and, from Lucide, a key, a warning and an outside-link icon; `shape.toml [voice]` gains the card's parts; colours reuse `pop_*`. `themes.toml` gains `warn`/`warn_ink` (card warnings, old facts) and `prep`/`prep_ink` (the amber Preparing light) in every theme and variant, through `kx-theme`.
- **Texts (EN, FR, AR), `i18n/voice/`:** `voice.cloud`; `voice.mark-{off,needs-key,ready,untested,unavailable}`; the card's lines `voice.card-{where,trains,keeps,free,light,heavy,read-on,old,link,confirm}`; each warning; `voice.key-{line,paste,test,ok,bad,remove}`; `voice.{address-line,model-line,other-model,keep-nothing}`; `voice.on-bar-cloud`; `voice.use-local`; `voice.send-warning`; `voice.note-{net,key,credit,allowance,rate,too-big,empty,strange,terms,no-local}`; `voice.preparing` and the bar's Narrator name `voice.name-preparing`.

## Tasks
1. **Licences, ADR and pointers.** *Files:* `docs/adr/0047-*.md`, `docs/adr/README.md`, the spec §8.2 and ADR-0007 pointers, `THIRD_PARTY.md`, `deny.toml`. *Implement:* `kx-licence-check` for any crate the shapes need (a JSON reader, Base64) and the new icons; the ADR. *Verify:* `cargo deny check`; `cargo xtask tidy` 0 errors.
2. **Mock-up first.** *Files:* `design/voice-bar.html`. *Implement:* the Cloud page, a card with and without a warning, the typed lines, the bar's cloud mark, and the Preparing bar with the amber Dictation key, in the three themes and in Arabic. *Verify:* the owner's yes in a pop-up, before any panel state or QML.
3. **The gear ends a dictation.** *Files:* `kx-mod-voice/src/{actor.rs, flow/acts.rs, flow/tests/*}`, `kx-mod-voice/src/actor/tests/view.rs`. *Test first:* the gear or the tray while preparing, listening or transcribing ends the dictation as Cancel does, with nothing sent, transcribed or typed, then the panel opens; edge 25. *Implement:* the panel's open first cancels a running dictation. *Verify:* `cargo test -p kx-mod-voice`; commit.
4. **Preparing.** *Files:* `kx-module-api/src/ports/voice{.rs, /bar.rs}`, `kx-mod-voice/src/flow/{notes,view}.rs`, `kx-mod-keyboard/src/face/{voice.rs, inner/voice.rs}`, `kx-ui/src/hub/voice/*.rs`, `kx-ui/qml/{VoiceBar,Key,Led}.qml`, `kx-theme/tokens/themes.toml`, `i18n/voice/{en,fr,ar}.ftl`, `qmltest/tst_voicebar.qml`. *Test first:* Preparing shows from Start until the microphone opens, then the red listening light; a loading model is no error; Stop or the key while preparing closes with nothing sent; the amber key and the bar's Narrator name in three languages; edges 26 and 27. *Implement:* `VoiceBar::Preparing`, the preparing light, its tokens and texts, with `kx-theme`. *Verify:* `cargo test -p kx-module-api -p kx-mod-voice -p kx-mod-keyboard -p kx-ui`; `qmltestrunner` at both scalings; commit.
5. **Module API.** *Files:* `kx-module-api/src/{ports.rs, ports/cloud*.rs, ports/voice*.rs}`. *Test first:* the service names its gate; `CloudAsk` and a key-carrying act print redacted; an address that is not HTTPS cannot be built. *Implement:* the types. *Verify:* `cargo test -p kx-module-api`.
6. **Fakes.** *Files:* `kx-platform-fake/src/cloud.rs`. *Test first:* scripted texts, each fault, a slow answer and a cut are told as scripted; sent pieces are counted, never kept. *Verify:* `cargo test -p kx-platform-fake`.
7. **Frames.** *Files:* `kx-ipc/src/{code.rs, limits.rs, net*.rs, tests/*}`, `docs/security/kx-ipc.md`. *Test first:* each frame round-trips; the key and sound print redacted; an oversized reply is refused; property: random bytes never panic. *Verify:* `cargo test -p kx-ipc`.
8. **Worker: the job and the upload form.** *Files:* `apps/keyxtend-worker/src/{args.rs, net.rs, net/{shape,form,wav,reply}.rs}`, reply fixtures, `xtask` tidy list. *Test first (a fake `Http`):* the form's fields in the row's order, with the model and the language; the key only in its header; a reply read to text; the key in no error or log; property: a random reply never panics; edges 2 to 5 and 14. *Implement:* `Job::SpeechNet`, `Shape`, the form. *Verify:* `cargo test -p keyxtend-worker net`; `cargo xtask tidy`.
9. **Host relay.** *Files:* `kx-platform/src/{cloud.rs, cloud/*.rs, speech/*.rs}`. *Test first:* a cloud dictation opens the `voice` job with no model and the row's cut limits; pieces go several at a time within the row's limits and come back in order; after `net_tries` a fault comes back and the recording is kept; buffers are wiped; edges 1 and 8. *Verify:* `cargo test -p kx-platform cloud speech`.
10. **Module: services, keys and rules.** *Files:* `kx-mod-voice/{services.toml, defaults.toml, src/{services,keys,cloud,settings}.rs}`. *Test first:* every row is valid (HTTPS, a known shape, languages, limits, dated facts, a terms number); a cloud model is never a default; a service is off until confirmed; keys seal per service; property: no flow ever moves from a local model to a service by itself; edges 11, 15, 16, 21 and 24. *Verify:* `cargo test -p kx-mod-voice`.
11. **Module: flow, panel and compare.** *Files:* `kx-mod-voice/src/{flow/*.rs, panel.rs, view.rs, compare.rs, counts.rs, history.rs}`, `docs/security/kx-mod-voice.md`. *Test first:* a cloud pick shows the cloud mark from Start; counts and history name the service; edges 6, 7, 9, 10, 12, 19, 20, 22 and 23. *Implement:* the cloud pick in the flow, the Cloud page's state and typed lines, ticked cloud models in Compare. *Verify:* `cargo test -p kx-mod-voice`; `cargo xtask tidy`.
12. **The app.** *Files:* `apps/keyxtend/{src/boot.rs, tests/life/{voice,ships}.rs}`, `xtask/src/*`. *Test first:* `CloudSpeech` reaches the module alone; `OpenLink` and `ReadClipboard` gain exactly the voice module; a key reaches neither the ui's view nor the keyboard; the app still links no network crate. *Verify:* `cargo test -p keyxtend`; `cargo xtask stage` within 80 MB.
13. **UI.** *Files:* `kx-ui/src/{hub/voice.rs, bridges/voice.rs}`, `kx-ui/qml/Voice{Cloud,Card,TypedLine,Bar,Models}.qml`, `i18n/voice/{en,fr,ar}.ftl`, `kx-theme/tokens/{icons,shape}.toml`, `qmltest/tst_voicecloud.qml`, `apps/keyxtend/tests/agree/{texts,icons}.rs`. *Test first (three languages):* the `cloud` switch; each mark; the card's lines with their date; a warning card; the key as dots with Paste and Test; the address and model lines; the keep-nothing switch; the cloud mark and the service's name on the bar; cloud models marked on the Models page; the Models page's "very slow here" line for a default that is Not for this PC leads to the Cloud page (owner, 2026-11-15); edges 15, 17 and 18. *Verify:* `qmltestrunner` at both scalings; `cargo test -p kx-ui -p keyxtend`.
14. **The form's rows.** *Files:* `services.toml`, fixtures. *Test first:* Groq, OpenAI (its `languages[]` list), Mistral, Together, Lemonfox, DeepInfra, OpenRouter and the Custom row each build their request and read their reply; OpenRouter's keep-nothing field present while its switch is on, and "Other model" takes a typed id; edges 13 (Custom) and 17. *Verify:* `cargo test -p kx-mod-voice -p keyxtend-worker`; commit, as this step is usable alone.
15. **OpenRouter.** Folded into task 14 (owner, 2026-11-17): OpenRouter now takes the upload form.
16. **Azure.** *Files:* `net/azure.rs`, the classic and MAI rows, fixtures. *Test first:* one upload with its settings part; the model as one line of that part; its reply reader, each fault and the property; edge 13 for the resource address. *Verify:* as task 14; commit.
17. **DeepInfra.** Folded into task 14 (owner, 2026-11-17): DeepInfra now documents the upload form.
18. **ElevenLabs.** *Files:* `net/eleven.rs`, its row, fixtures. *Test first:* its own field names and key header; its warning on the card and at the first dictation; its reply reader, each fault and the property. *Verify:* as task 14; commit.
19. **Gemini.** *Files:* `net/gemini.rs`, its row, fixtures. *Test first:* one request with the sound inline (Base64) within its size limit, and each failure; its reply reader and the property; a row marked not offered never shows. *Implement:* the adapter and its row, not offered until task 24's check. *Verify:* as task 14; commit.
20. **Gates.** *Files:* `tools/kx-gates/src/g22*.rs`, `kx-gates-p11.toml`. *Test first:* the config check. *Implement:* G22's cloud line on the product, off unless a key is set on the test PC; a timing line per stage of each passing language (start, microphone open, load, each piece, Stop to text), for part 1's slow card texts. *Verify:* `cargo test -p kx-gates`; a card run of G22 shows where the time goes.
21. **Spike removal.** *Files:* what is left of `spike/worker/`, `spike/Cargo.toml`, `spike/spike.toml`, `docs/spike-move-map.md`. *Test first:* the spike's remaining crate and the gates still build. *Verify:* `cargo test --manifest-path spike/Cargo.toml`; `cargo build -p kx-gates`.
22. **Docs.** `docs/architecture/*`, `docs/security/{kx-mod-voice,keyxtend-worker,kx-platform,kx-platform-net,keyxtend}.md`, `docs/items/{P11,P13}.md`, spec §8.2 "as built", and a line in `.claude/skills/kx-release/SKILL.md`: read every service's facts again at each release.
23. **Review and security.** `kx-review` on every changed file, `kx-security-check`, `/code-review` and `/security-review`; fix every finding.
24. **Manual check, done.** First read every service's own pages again, tell the owner which can be tested without paying and whether a card is needed, and ask which keys they will make. Then `kx-windows-manual-test` with the owner's own keys; each row is corrected from what the real keys show, or marked untested. Gemini's row is offered only if its six dictations pass; else the reason goes to `docs/items/P11.md`. Then `kx-module-done` and the records.

## Edge cases
1. Nothing leaves before Stop; Cancel sends nothing; Cancel while uploading drops the requests.
2. A wrong or revoked key; no credit left; the free allowance used up (with its renewal time, if the service tells).
3. Too many requests: wait once, then a note.
4. An empty text; a reply that is not the shape's, such as a hotspot's sign-in page; a reply over the cap.
5. The network fails mid-upload; a timeout past `net_wait_s`; the service is down.
6. After any failure the recording is kept: the bar offers Try again and "Use the local model", and nothing is typed twice.
7. The language is not one the service's model knows; a regional code the service refuses.
8. Pieces are cut within the service's limits; a 10-minute dictation goes in several pieces.
9. The key is removed, or the service switched off, while a dictation runs: a note, nothing sent after it.
10. A password box while a cloud model is picked: the local model serves, or the bar says none is ready; a box the system cannot read behaves as text.
11. A hidden run: cloud is off and no key is read.
12. Compare sends only to the ticked services, after its warning; an unticked service gets nothing.
13. An Azure address typed wrong; a Custom address that is not HTTPS.
14. A redirect to another host is not followed.
15. A card shows the date its facts were read; past `facts_days` the date takes the warning tone.
16. A newer terms number asks again before the next upload; declined, the service goes back to Off and its picks to the local default.
17. OpenRouter's keep-nothing switch on: a model with no such host is unavailable, with the reason.
18. A connection with no real-key check shows "untested" on its card.
19. The field in front becomes a password box before Stop: nothing is uploaded.
20. "Use the local model" with no local model ready is dimmed and offers Download.
21. `cloud` off: no service is offered, nothing is sent, and cloud picks fall back to the local default.
22. A Custom address and model, an Azure address and an "Other model" id are typed on the panel line and kept in `services`.
23. The card's link opens only the row's HTTPS address; Paste puts a key on its line as dots.
24. Groq is listed first on the Cloud page.
25. The gear or the tray while a dictation prepares, listens or transcribes: it ends as Cancel does, nothing is sent, transcribed or typed, and the panel opens.
26. Preparing until the microphone opens: no error while the model loads; Stop or the key then closes with nothing sent.
27. The microphone never opens within the job's `load_s` wait: the existing microphone note, never a crash or a stuck amber key.

## Security
- Capabilities: `Network` (now also the services the user switched on), `SecretStore` (keys), `Voice`, `OpenLink` (the card's link) and `ReadClipboard` (Paste); the key reaches the module and the `speech-net` job only.
- Data: the key (a `Secret`: sealed at rest, wiped after each run, in no setting, log or crash report); sound (leaves the PC only at Stop, only to the service named on the bar); replies (untrusted text, size-capped); facts (public, dated).
- Threats: a stolen key; a key left in the clipboard after Paste; a hostile or oversized reply; a look-alike address in the Custom row; a redirect to another host; sound sent to a service other than the one shown; a silent move from local to cloud; a password's speech uploaded; a service that changes its terms; the `speech-net` job reaching the microphone.

## Manual Windows check
- Install the merged build first; under load and in G22's weak mode, the first Dictation click shows amber Preparing, then red, with no error.
- The gear while listening stops at once, types nothing and opens the panel.
- Keys: copy one on the provider's page, then press Paste on the key line at once; KeyXtend's history keeps no copy. Clear Windows' own clipboard history afterwards. No key in the chat.
- Keys at the check (owner, 2026-11-17): Groq, Gemini, OpenRouter, ElevenLabs, and Azure on its trial; every other row ships marked untested.
- With the owner's own keys, one service per connection the owner has a key for: switch it on, read its card, type or paste the key, Test.
- Dictate in English, French and Arabic: the bar shows the cloud mark and the service's name, and Stop types the text.
- Cancel before Stop, then see on the service's own usage page that nothing was sent.
- Cut the network during an upload: Try again and "Use the local model" both work, and nothing is typed twice.
- A password box with a cloud model picked uses the local model.
- Compare models with one cloud model ticked; the card of a service without a key check says "untested".
- Gemini, if the owner has a key: six dictations (English, French and Arabic, a short and a long one each) each come back as text in the set language.

## Done when
- The Definition of Done in `docs/roadmap.md` holds.
- Every connection is either checked with a real key or marked untested; no test reaches the real network; the owner's manual check passes.
