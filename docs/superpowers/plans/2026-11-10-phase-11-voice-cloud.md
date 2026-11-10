# Phase 11, part 2: voice typing through cloud services (v0.7)

## Goal
- The user may pick, per language, a cloud service and model beside the local ones, with their own key. The bar and everything after Stop stay the same, and nothing leaves the PC before Stop.
- Design: `docs/superpowers/specs/2026-11-10-voice-cloud-design.md` (approved 2026-11-10; replaces the cloud lines of spec §8.2 and of ADR-0007); `docs/items/P11.md`. It builds on part 1, `2026-11-10-phase-11-voice-typing.md`, which must be merged first.

## Scope
- In: ADR-0046; the `CloudSpeech` port, its host and fake; the `kx-ipc` net frames; the worker's `speech-net` job with six request shapes; the services table and its dated facts; sealed keys; the Cloud page, the service cards, the key line and the bar's cloud mark; the settings added; G22's cloud line; the spike's last voice files removed.
- Out: a shipped key or an account made for the user; live transcription; services that need three or more requests, other than Gemini; NVIDIA's trial, Alibaba direct and Cohere's trial key; facts refreshed over the network.

## Design notes
- **Crates:** no new crate; touched: `kx-module-api`, `kx-ipc`, `kx-platform`, `kx-platform-net`, `kx-platform-fake`, `kx-mod-voice`, `kx-ui`, `kx-theme`, `apps/keyxtend`, `apps/keyxtend-worker`, `tools/kx-gates`.
- **ADR-0046:** the third job `speech-net` (network, no microphone); the relay at Stop through the app as unread secret bytes; the shapes and the services table; keys sealed in `<data>/voice/` (replaces ADR-0007's secret-store line); `Network` now also covers the services the user switched on; terms numbers and asking again.
- **Port `ports/cloud.rs`** (`CloudSpeechKey`, gated `Network`): `CloudSpeech::send(CloudAsk)` takes the row's fields (shape, address, key header, model, language code, limits), the key as a `Secret` and the pieces. It gives each piece's text, or a `CloudFault`: key, credit, allowance with its renewal, rate, too big, language, empty, strange reply, network, timeout. `test` says whether a key works; dropping the run cancels it.
- **Events:** `VoiceView` gains the Cloud page, a service's card and the bar's cloud mark with the service's name. `VoiceAct` gains switch on, confirm, key set (a `Secret`), Test, Remove key and "Use the local model". The key is typed on the panel's own line (`PanelLine`, ADR-0040).
- **Frames:** `kx-ipc/src/net*.rs`: `NetSend` and `NetTest`, with the notes `NetText` and `NetFault`; the key and the sound are secrets; a reply's size is capped in `limits.rs`.
- **Worker:** `Job::SpeechNet` in `apps/keyxtend-worker/src/{net.rs, net/*.rs}`. Trait `Shape` builds a request from the row's fields and reads the reply into text or a fault. One file per shape: `form.rs` (from the spike's `cloud.rs` and `multipart.rs`), `openrouter.rs`, `deepinfra.rs`, `azure.rs`, `eleven.rs`, `gemini.rs`; `wav.rs` wraps a piece. HTTPS only, through `kx-platform-net`, with no redirect to another host.
- **Host:** `kx-platform/src/cloud/` takes the pieces from the `voice` job at Stop and feeds `speech-net`, several at a time within the row's limits, in wiped buffers, with `net_wait_s` and `net_tries`.
- **Module table:** `kx-mod-voice/services.toml`, one row per service: id, name, shape, address or "typed", key header, where the key is made, limits, up to ten models with their languages and codes, the facts as numbers and flags with their date and source, the warning, the terms number. Rows at the start: Groq, OpenAI, Mistral, Together, Lemonfox, OVHcloud (with a key), Custom, OpenRouter, DeepInfra, Azure classic and MAI-Transcribe-2, ElevenLabs, Gemini.
- **A card's sentences** are translations filled with the row's numbers, so no price or claim is written in code or twice.
- **Store:** `<data>/voice/keys/`, one sealed blob per service; a hidden run reads and writes none.
- **Settings `[voice]` added:** `cloud` false; `services` {} (each switched-on service with its confirmation date and terms number, the Custom and Azure addresses, and OpenRouter's keep-nothing switch, on to start); `net_wait_s` 60 (5–300); `net_tries` 2 (1–5); `facts_days` 180 (30–730). A `picks` entry may now name a service and model.
- **Tokens:** `icons.toml [voice]` gains cloud (bundled) and, from Lucide, a key, a warning and an outside-link icon; `shape.toml [voice]` gains the card's parts; colours reuse `pop_*` and the themes' warning tone.
- **Texts (EN, FR, AR), `i18n/voice/`:** `voice.cloud`; `voice.mark-{off,needs-key,ready,untested,unavailable}`; the card's lines `voice.card-{where,trains,keeps,free,light,heavy,read-on,old,link,confirm}`; each warning; `voice.key-{line,paste,test,ok,bad,remove}`; `voice.on-bar-cloud`; `voice.use-local`; `voice.other-model`; `voice.send-warning`; `voice.note-{net,key,credit,allowance,rate,too-big,empty,strange,terms,no-local}`.

## Tasks
1. **Licences, ADR and pointers.** *Files:* `docs/adr/0046-*.md`, `docs/adr/README.md`, the spec §8.2 and ADR-0007 pointers, `THIRD_PARTY.md`, `deny.toml`. *Implement:* `kx-licence-check` for any crate the shapes need (a JSON reader, Base64) and the new icons; the ADR. *Verify:* `cargo deny check`; `cargo xtask tidy` 0 errors.
2. **Module API.** *Files:* `kx-module-api/src/{ports.rs, ports/cloud*.rs, ports/voice*.rs}`. *Test first:* the service names its gate; `CloudAsk` and a key-carrying act print redacted; an address that is not HTTPS cannot be built. *Implement:* the types. *Verify:* `cargo test -p kx-module-api`.
3. **Fakes.** *Files:* `kx-platform-fake/src/cloud.rs`. *Test first:* scripted texts, each fault, a slow answer and a cut are told as scripted; sent pieces are counted, never kept. *Verify:* `cargo test -p kx-platform-fake`.
4. **Frames.** *Files:* `kx-ipc/src/{code.rs, limits.rs, net*.rs, tests/*}`, `docs/security/kx-ipc.md`. *Test first:* each frame round-trips; the key and sound print redacted; an oversized reply is refused; property: random bytes never panic. *Verify:* `cargo test -p kx-ipc`.
5. **Worker: the job and the upload form.** *Files:* `apps/keyxtend-worker/src/{args.rs, net.rs, net/{shape,form,wav,reply}.rs}`, reply fixtures. *Test first (a fake `Http`):* the form's fields in the row's order, with the model and the language; the key only in its header; a reply read to text; each fault of edge cases 2 to 5; no redirect to another host; the key in no error or log; property: a random reply never panics. *Implement:* `Job::SpeechNet`, `Shape`, the form. *Verify:* `cargo test -p keyxtend-worker net`.
6. **Host relay.** *Files:* `kx-platform/src/{cloud.rs, cloud/*.rs}`. *Test first:* nothing is sent before Stop; Cancel before Stop sends nothing and Cancel while uploading drops the requests; pieces go several at a time within the row's limits and come back in order; after `net_tries` a fault comes back and the recording is kept; buffers are wiped. *Verify:* `cargo test -p kx-platform cloud`.
7. **Module: services, keys and rules.** *Files:* `kx-mod-voice/{services.toml, defaults.toml, src/{services,keys,cloud,settings}.rs}`. *Test first:* every row is valid (HTTPS, a known shape, languages, limits, dated facts, a terms number); a cloud model is never a default; a service is off until confirmed, a newer terms number asks again, and a declined confirmation switches it off; keys seal per service and a hidden run keeps none; facts older than `facts_days` are marked old; property: no flow ever moves from a local model to a service by itself. *Verify:* `cargo test -p kx-mod-voice`.
8. **Module: flow, panel and compare.** *Files:* `kx-mod-voice/src/{flow/*.rs, panel.rs, view.rs, compare.rs, counts.rs, history.rs}`, `docs/security/kx-mod-voice.md`. *Test first:* edge cases 1 and 6 to 12. *Implement:* the cloud pick in the flow, the Cloud page's state, ticked cloud models in Compare, the service's name in counts and history. *Verify:* `cargo test -p kx-mod-voice`; `cargo xtask tidy`.
9. **The app.** *Files:* `apps/keyxtend/{src/boot.rs, tests/life/{voice,ships}.rs}`, `xtask/src/*`. *Test first:* `CloudSpeech` reaches the module alone; a key reaches neither the ui's view nor the keyboard; the app still links no network crate. *Verify:* `cargo test -p keyxtend`; `cargo xtask stage` within 80 MB.
10. **Mock-up first.** *Files:* `design/voice-bar.html`. *Implement:* the Cloud page, a card with and without a warning, the key line and the bar's cloud mark, in the three themes and in Arabic. *Verify:* the owner's yes in a pop-up, before any QML.
11. **UI.** *Files:* `kx-ui/src/{hub/voice.rs, bridges/voice.rs}`, `kx-ui/qml/Voice{Cloud,Card,KeyLine,Bar,Models}.qml`, `i18n/voice/{en,fr,ar}.ftl`, `kx-theme/tokens/{icons,shape}.toml`, `qmltest/tst_voicecloud.qml`, `apps/keyxtend/tests/agree/{texts,icons}.rs`. *Test first (three languages):* each mark; the card's lines with their date, and the old tone past `facts_days`; a warning card; the key as dots with Paste and Test; the cloud mark and the service's name on the bar; cloud models marked on the Models page. *Verify:* `qmltestrunner` at both scalings; `cargo test -p kx-ui -p keyxtend`.
12. **The form's rows.** *Files:* `services.toml`, fixtures. *Test first:* Groq, OpenAI, Mistral, Together, Lemonfox, OVHcloud and the Custom row each build their request and read their reply; a Custom address that is not HTTPS is refused. *Verify:* `cargo test -p kx-mod-voice -p keyxtend-worker`; commit, as this step is usable alone.
13. **OpenRouter.** *Files:* `net/openrouter.rs`, its row, fixtures. *Test first:* the sound as Base64 in the JSON body; the keep-nothing flag present while the switch is on; a model with no such host shown as unavailable; "Other model" takes a typed id. *Verify:* as task 12; commit.
14. **Azure.** *Files:* `net/azure.rs`, the classic and MAI rows, fixtures. *Test first:* one upload with its settings part; the model as one line of that part; a wrongly typed resource address is refused with a note. *Verify:* as task 12; commit.
15. **DeepInfra.** *Files:* `net/deepinfra.rs`, its row, fixtures. *Test first:* the model sits in the address and the file field is `audio`. *Verify:* as task 12; commit.
16. **ElevenLabs.** *Files:* `net/eleven.rs`, its row, fixtures. *Test first:* its own field names and key header; its warning on the card and at the first dictation. *Verify:* as task 12; commit.
17. **Gemini.** *Files:* `net/gemini.rs`, its row, fixtures. *Test first:* the three requests in order, and a failure at each step. *Implement:* only after the owner's real-key test passes (the six clips each come back as text in the set language); otherwise the row is left out and the reason noted in `docs/items/P11.md`. *Verify:* as task 12; commit.
18. **Spike removal.** *Files:* what is left of `spike/worker/`, `spike/spike.toml`, `docs/spike-move-map.md`. *Test first:* the spike still builds and its tests pass. *Verify:* `cargo test -p spike-core -p qt-kb`.
19. **Gates.** *Files:* `tools/kx-gates/src/g22*.rs`, `kx-gates-p11.toml`. *Test first:* the config check. *Implement:* G22's cloud line on the product, off unless a key is set on the test PC. *Verify:* `cargo test -p kx-gates`.
20. **Docs.** `docs/architecture/*`, `docs/security/{kx-mod-voice,keyxtend-worker,kx-platform,kx-platform-net,keyxtend}.md`, `docs/items/{P11,P13}.md`, spec §8.2 "as built", and a line in `.claude/skills/kx-release/SKILL.md`: read every service's facts again at each release.
21. **Review and security.** `kx-review` on every changed file, `kx-security-check`, `/code-review` and `/security-review`; fix every finding.
22. **Manual check, done.** `kx-windows-manual-test` with the owner's own keys, then `kx-module-done` and the records; each row is corrected from what the real keys show, or marked untested.

## Edge cases
1. Nothing leaves before Stop; Cancel sends nothing; Cancel while uploading drops the requests.
2. A wrong or revoked key; no credit left; the free allowance used up (with its renewal time, if the service tells).
3. Too many requests: wait once, then a note.
4. An empty text; a reply that is not the shape's, such as a hotspot's sign-in page; a reply over the cap.
5. The network fails mid-upload; a timeout past `net_wait_s`; the service is down.
6. After any failure the recording is kept: the bar offers Try again and "Use the local model", and nothing is typed twice.
7. The language is not one the service's model knows; a regional code the service refuses.
8. A piece over the service's limit is cut to fit; a 10-minute dictation goes in several pieces.
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

## Security
- Capabilities: `Network` (now also the services the user switched on), `SecretStore` (keys), `Voice`; the key reaches the module and the `speech-net` job only.
- Data: the key (a `Secret`: sealed at rest, wiped after each run, in no setting, log or crash report); sound (leaves the PC only at Stop, only to the service named on the bar); replies (untrusted text, size-capped); facts (public, dated).
- Threats: a stolen key; a hostile or oversized reply; a look-alike address in the Custom row; a redirect to another host; sound sent to a service other than the one shown; a silent move from local to cloud; a password's speech uploaded; a service that changes its terms.

## Manual Windows check
- With the owner's own keys, one service per connection the owner has a key for: switch it on, read its card, type the key, Test.
- Dictate in English, French and Arabic: the bar shows the cloud mark and the service's name, and Stop types the text.
- Cancel before Stop, then see on the service's own usage page that nothing was sent.
- Cut the network during an upload: Try again and "Use the local model" both work, and nothing is typed twice.
- A password box with a cloud model picked uses the local model.
- Compare models with one cloud model ticked; the card of a service without a key check says "untested".

## Done when
- The Definition of Done in `docs/roadmap.md` holds.
- Every connection is either checked with a real key or marked untested; no test reaches the real network; the owner's manual check passes.
