# Voice typing — design (P11, part 2: cloud services)

Decided with the owner on 2026-11-09 and 2026-11-10, from two research rounds read on those days. This file builds on part 1, [`2026-11-10-voice-typing-design.md`](2026-11-10-voice-typing-design.md), which must be merged first: the bar, the Voice panel, the worker session, history and counts are the same, and only the engine differs. It replaces the cloud lines of spec §8.2 and of ADR-0007. Prices, free allowances and privacy facts below were true on the day read; they change, so they live in a table with their date and are never written in code. Lines marked **(designer)** are choices the owner has not made; the last section gathers them.

## As built (2026-11-20)

- **Offered rows:** Groq, Lemonfox, Custom, OpenRouter, ElevenLabs, Gemini and AssemblyAI. OpenAI, Mistral, Together and DeepInfra are built and tested but hidden (`offered = false`), since the owner cannot test them by hand (ADR-0050).
- **Dropped:** OVHcloud (owner, 2026-11-17) and Azure, classic and MAI (owner, 2026-11-19). MAI-Transcribe-2 is reached through OpenRouter.
- **Four request shapes, not six** (ADR-0050, ADR-0051): the upload form (OpenRouter and DeepInfra took it too), ElevenLabs' form, Gemini's single JSON request, and AssemblyAI Sync. AssemblyAI's classic upload-and-poll API stays a non-goal.
- **Facts:** the rows carry the facts read on 2026-11-17 (AssemblyAI's on 2026-11-19), in `crates/kx-mod-voice/services.toml` and `services-own.toml`. The table below stays as the design day's record.
- **Untested:** every offered row ships marked untested on its card until the owner's hands-on check (task 24), which hides any row that fails (ADR-0050). Gemini is offered on the same terms, not held back until six dictations pass.
- **Typed addresses:** the table keeps a rule that a typed address must sit under the row's domain, but no shipped row uses it. Custom is the only typed row and takes any HTTPS host name.
- **Also in part 2:** Paste at once (ADR-0048), the panel's typed lines (ADR-0049), the Preparing state, the gear ending a dictation, the Models page's Local | Cloud switch, and the card check by language (ADR-0052).

## Why

- A cloud service gives users with a weak PC, or a language no local model serves well, a way to dictate.
- The owner wants as many services as fit, each with its free allowance and cost shown, and wants to add or drop one without touching the feature's code.
- No service is trusted by default: the user's voice leaves the PC only for a service the user switched on, with that user's own key.

## Goals, non-goals, anti-goals

- **Goals:**
  - the user picks, per language, a local model or a cloud service and model; the bar and everything after Stop behave the same;
  - many services through few connections: a service that speaks a known request shape is one table row;
  - each service shows what it costs, what is free, and what it does with the voice, with the date checked;
  - the user's own key, stored sealed; a test button says whether it works;
  - a failed upload never loses the dictation.
- **Non-goals in part 2:**
  - a shipped key, an account made for the user, or sign-in with a consumer plan (no plan allows it);
  - live transcription while speaking;
  - services that need three or more requests per clip, other than Gemini (Soniox, Gladia, Rev.ai, classic AssemblyAI);
  - NVIDIA's own trial (another protocol, product use forbidden), Alibaba direct, Cohere's trial key (commercial use forbidden); HeyGen has no transcription.
- **Anti-goals (a failure even if it works):**
  - sound sent anywhere before the user switched that service on, or sent to a service other than the one shown on the bar;
  - a silent switch from a local model to a cloud service;
  - dictation into a password box sent to a cloud service;
  - a key in the settings file, a log, a crash report or the repository;
  - a price or a privacy claim written in code, or shown without its date.

## What the user sees

- The Voice panel gains a **Cloud** page: a paged list of services, each marked Off, Needs a key, or Ready.
- **Switching a service on** shows its card first: where the voice goes, whether the service trains on it or keeps it, what is free and what it costs for light and heavy use, the date these facts were read, and a link to the service's own page. A service that trains on the voice by default carries a warning and names the account switch that stops it. The user confirms once per service.
- **The key** is typed on the panel's own line (ADR-0040), shown as dots, with Paste; then Test. Remove key deletes it.
- **Choosing:** on the Models page, each ready service's models appear beside the local ones for the language in front, marked with a cloud icon. "Use for this language" works as for a local model. Local stays the default; a cloud model is never the table's default (owner: off until switched on).
- **On the bar**, a cloud dictation shows a small cloud mark and the service's name while listening and transcribing, so the user always knows the voice is leaving the PC. The chip shows the service's short name: Custom's own short name, and every other service its name. When a password box sends a warned first dictation to the model on this PC, the card's warning goes at the hand-over and its band stays empty until the text, so Cancel never moves; while it writes, a model's name may be cut so the width holds, and readers still hear it whole.
- Compare models can include cloud models: the user ticks which ones, and it says that the recording will be sent to each ticked service (designer).

## How a cloud dictation runs

1. Listening is unchanged: the `voice` job records, sends levels and cuts pieces at pauses, within the service's limits. It still has no network, and for a cloud pick it loads no model.
2. **Nothing leaves before Stop (owner).** Cancel therefore sends nothing, and the field in front is checked once, at Stop, when the text is about to be typed (and at Start, to show the note early).
3. At Stop each piece goes from the `voice` job to the app, and from the app to a third job, `speech-net`, which has the network and no microphone (designer). The pieces travel as secret bytes in wiped buffers; they are not encrypted between the jobs, and the app passes them on without reading sound.
4. `speech-net` sends the pieces to the service in the request shape its row names, with the language the keyboard shows, several at a time within the service's limits (requests a minute, seconds and megabytes, from its row), and returns the texts.
5. The texts are joined in order and typed, as in part 1. History and counts note the service and model.
6. The key travels to `speech-net` with each run in a wiped buffer, and lives nowhere else outside the sealed store.

## Connections and services

A connection is one request shape, written once as a small adapter. A service is a table row that names its shape. Six shapes cover the owner's list:

| Shape | Services ready at the start | Notes |
|---|---|---|
| Upload form, OpenAI style | Groq, OpenAI, Mistral, Together, Lemonfox, OVHcloud, and a Custom row | the row names the address, the model, the field names and their order, and the key header |
| OpenRouter JSON | OpenRouter | the sound as Base64 text; this shape is known to carry the flag that limits routing to hosts that keep nothing, so OpenRouter uses it alone |
| DeepInfra form | DeepInfra | the model sits in the address and the file field is `audio` |
| Azure fast transcription | Azure AI Speech (classic) and MAI-Transcribe-2 | one upload with a small settings part; the user types their resource's address; the model is one line of that part |
| ElevenLabs form | ElevenLabs Scribe | its own field names and key header |
| Gemini | Google Gemini 3.5 Transcribe | three requests per clip as documented; built last, and offered only after a test with a real key |

- **As built (2026-11-20):** four shapes. The upload form serves Groq, OpenAI, Mistral, Together, Lemonfox, OpenRouter, DeepInfra and Custom; ElevenLabs keeps its form; Gemini sends one JSON request with the sound inline; AssemblyAI Sync sends a JSON settings part first and names the model in a header.
- **Why direct ElevenLabs and Gemini (owner):** both are also on OpenRouter, but OpenRouter is pay-as-you-go only, and it cannot use an ElevenLabs plan's monthly allowance or Google's free tier and plan credit.
- **Custom row (designer):** the user types an HTTPS address, a model name and a key for any service that speaks the upload form, such as a server of their own.
- **OpenRouter's keep-nothing switch (designer)** is on the service's card and on to start. While it is on, a model with no such host is shown as unavailable, with the reason.
- **Models per service (designer):** a row offers a short list, at most ten, of the models that serve English, French or Arabic well, not the service's whole catalogue. A "Other model" line lets the user type a model's id.
- **The big NVIDIA model** (Nemotron 3.5 Transcribe) is too large for any laptop; it is reached through DeepInfra or OpenRouter.

### The services table

One row per service: id, shown name, shape, address (or "typed by the user"), key header, the models offered with the languages each knows and the code each wants, limits per request, and where its key is made. Each row also carries the facts shown on its card, with the date read and the source: free allowance, price for light and heavy use, training, retention, region, and the warning if any.

- The table is compiled in, like part 1's. Reading every row's facts again is a step of each release; nothing is refreshed over the network (designer).
- Each row has a terms number, raised when its training, retention or region fact changes. A service whose number is newer than the one the user confirmed asks again before the next upload (designer).

Facts read on 2026-11-09 and 2026-11-10 (light use is 15 hours a month, heavy is 60):

| Service | Free allowance | Light, heavy | The voice |
|---|---|---|---|
| Groq (Whisper) | 8 hours a day, 20 requests a minute | $0, or $0.60 to $6.66 | no training; nothing kept by default; United States |
| OpenRouter (30 models) | a small sign-up allowance | from $0.18, $0.72; 5.5% fee on credit | keeps nothing itself; the host decides; a flag asks for hosts that keep nothing |
| OVHcloud (Whisper), dropped | no key: 2 requests a minute, 60 s clips | €0.69, €2.76 | never trains; European Union |
| OpenAI (gpt-transcribe) | none; ChatGPT Pro carries API credit | $4.05, $16.20 | no training unless opted in |
| Mistral (Voxtral) | unclear; Pro plan lists $15 a month of API credit | $3.15, $12.60 | free mode may train; an account switch |
| Together | none; $5 first purchase | $1.35, $5.40 | keeps requests until an account switch is off |
| Lemonfox (Whisper) | first month | $5, $10 | deleted after processing |
| DeepInfra | unverified | from $0.18, $0.72; big Nemotron $1.80, $7.20 | no training; in memory only |
| ElevenLabs (Scribe v2) | a small monthly allowance, non-commercial | $3.30, $13.20 | **trains by default**; one account switch stops it |
| Gemini 3.5 Transcribe | a free tier; plan credit of $10 to $100 | $0, or about $4.50, $18 | **free tier outside Europe trains and may be read by staff** |

- The default the panel suggests first is Groq: nothing else is free at that scale.
- Gemini and ElevenLabs carry their warnings on the card and again at the first dictation (designer).
- **As built (2026-11-20):** these facts were read again on 2026-11-17 and are kept only in the services table. The first-dictation warning is a line under the bar's row, not a blocking step.

## Keys

- A key is the user's own. KeyXtend ships none and makes no account (ADR-0007).
- It is typed on the panel line, held as a `Secret`, and sealed in the `voice` folder of the data folder with that folder's key (ADR-0010), one blob per service. It is never in `settings.toml`, a log or memory after use.
- **Test (designer):** a cheap call that needs the key but sends no sound where the service has one; otherwise the card says the first dictation will test it.
- A hidden run keeps no key (ADR-0037): cloud is off there.

## Privacy and security

- Cloud is off until the user switches a service on (owner; CLAUDE.md). There is no automatic move from local to cloud, ever.
- A password box never goes to the cloud: the dictation uses the language's local model, or the bar says no local model is ready (designer). This means a box the system reports as a password (`FieldWatch`'s password kind). Fields it cannot read, such as a terminal or a remote desktop, behave as text; the bar's cloud mark is what tells the user (designer).
- `speech-net` runs under the worker's restricted token and job limits, talks only to the address in the row or the Custom row, over HTTPS, and follows no redirect to another host. Replies are size-capped and parsed by small, property-tested readers.
- The app relays the pieces as secret bytes and never parses sound. The `voice` job never gets the network.
- The module's Network capability, used in part 1 for downloads only, now also covers the services the user switched on.
- The card's link opens through `OpenLink`, and Paste on a typed line uses the clipboard route Quick-fill uses (`ReadClipboard`); the new ADR adds the voice module as a holder of both (designer).
- `docs/security/kx-mod-voice.md` gains the cloud threats: a stolen key, a hostile reply, a look-alike address in the Custom row, and a service that changes its terms.

## Edge cases for the plan

- The network fails mid-upload; a timeout; the service is down. The recording is kept: the bar offers Try again and "Use the local model" (designer), and nothing is typed twice.
- A wrong or revoked key; no credit left; the free allowance used up (say when it renews, if the service tells); too many requests (wait once, then say so).
- The service returns empty text; a reply that is not what the shape expects (a sign-in page from a hotspot); a reply too large.
- The language is not one the service's model knows; a regional code the service refuses.
- A piece over the service's limit; a very long dictation; Cancel while uploading (the request is dropped).
- The key is removed, or the service switched off, while a dictation runs.
- A password box while a cloud model is picked, or one that comes in front before Stop (nothing is uploaded); a hidden run.
- No local model as a fallback: "Use the local model" is dimmed and offers Download (designer).
- The `cloud` switch turned off: no service is offered, nothing is sent, and cloud picks fall back to the local default (designer). Since ADR-0054, they wait, and Cloud On brings each back.
- An Azure address typed wrong (void since Azure was dropped); a Custom address that is not HTTPS.
- A card always shows the date its facts were read; older than `facts_days`, the date takes a warning tone (designer).
- A service's terms number changed by an update; the user declines the new confirmation (the service goes back to Off). Since ADR-0054, its picks wait for its Confirm.
- A service switched Off keeps its listed picks: Start shows the Off note with Open service and never listens, and only that service's Confirm brings the pick back (owner, 2026-11-21). Since ADR-0054 decision 4, its entry waits in `parked` with its typed address, typed model and keep-nothing switch, and its typed picks wait too.

## To confirm with a real key

The research could not settle these. The builder makes no account, so each waits for the owner's own key at the hands-on check; then its row is corrected. A connection with no real-key check ships marked "untested" on its card.

- OVHcloud: its language field. Until its terms are read as allowing use without a key in a shipped app, its row asks for a key (designer).
- MAI-Transcribe-2: its price from 2027-01-01, and whether its sound is used for training.
- Gemini: a one-request path, the free tier's limits, and a report of empty replies left unanswered. Its connection is built but its row is not offered. The test is six dictations by the owner at the hands-on check (English, French and Arabic, a short and a long one each); it passes when each comes back as non-empty text in the set language. If it fails, the row stays out and the reason is noted in the item file (designer).
- OpenRouter: whether regional codes pass through.
- Mistral: whether the plan's credit covers speech. ElevenLabs: how its plan allowance and its API price fit together.
- DeepInfra: its sign-up credit, and its OpenAI-style address.
- **As built (2026-11-20):** the OVHcloud and MAI items are void. OpenAI, Mistral, Together and DeepInfra are hidden, and every offered row ships marked untested until the owner's hands-on check, which hides any row that fails.

## Build order

1. The `speech-net` job, the relay, and the upload form with Groq; then its other rows and the Custom row.
2. OpenRouter's JSON shape.
3. Azure, classic and MAI.
4. DeepInfra.
5. ElevenLabs.
6. Gemini: built, and offered only once the real-key test passes.

Each step is usable on its own and ends with its tests and a commit.

- **As built (2026-11-20):** OpenRouter and DeepInfra joined the upload form; Azure was dropped before its adapter was written, and its unused shape removed (task 16h); ElevenLabs, Gemini and then AssemblyAI Sync (task 19b) followed, and the card check by language came last (task 20b).

## Settings `[voice]` added by part 2

| Setting | Start | Meaning |
|---|---|---|
| `cloud` | false | the Cloud page's master switch |
| `services` | empty | the services the user switched on, each with its confirmation date |
| `net_wait_s`, `net_tries` | 60, 2 | wait per request; tries before a note |
| `facts_days` | 180 | age after which a card's facts show as old |

Picks stay in part 1's `picks`: a pick may now name a service and model.

- **As built (2026-11-20):** part 2 also added `insert` (Type it or Paste at once), `paste_ms` and `take_ms`. `services` also keeps each terms number, the Custom address, typed model ids, OpenRouter's keep-nothing switch and the warning shown. The values live in `crates/kx-mod-voice/defaults.toml`.

## Moving the spike

- `cloud.rs` and `multipart.rs` move from the spike's worker into the upload-form adapter; the spike's key from an environment variable gives way to the sealed store.
- Gate G22's cloud line is aimed at the product and stays off unless a key is set on the test PC.
- **As built (2026-11-20):** the spike's worker is gone (task 21). G22's cloud line borrows the sealed keys from the daily data folder that `[g22_cloud] daily_dir` names in the git-ignored local gate file.

## Tests and checks

- Each adapter is tested against a small fake service inside the tests: good replies, each error of the edge cases, slow and cut connections. No test reaches the real network.
- Property tests for every reply reader and for the services table (every row valid: HTTPS, a known shape, languages, limits, dated facts).
- Qt Quick tests for the Cloud page, a service card with and without a warning, the key line, and the bar's cloud mark, in three languages.
- The manual Windows check uses the owner's own keys, one service per connection the owner has a key for; a connection without a real-key check is marked untested on its card. As built, every offered row is marked untested until that check, which hides a row that fails or cannot be tested (ADR-0050 decision 4).

## Left to the builder

- Names of the job, frames, adapters and files; how the relay is paced.
- The exact fields of each shape, read from each service's current pages when its adapter is built.
- The Test call per service; the wording of cards and warnings in three languages, with the owner's Arabic check.

## The designer's calls

For the owner to check while reading: the third worker job and the relay through the app; the Custom row; OpenRouter's keep-nothing switch; short model lists; Compare models with ticked cloud models; warnings shown twice for Gemini and ElevenLabs; the Test call; password boxes forced to a local model, other unreadable fields treated as text; "Use the local model" after a failed upload; facts read again at each release and shown as old after 180 days; asking again when a service's terms change; OVHcloud with a key; Gemini's pass rule.

Added after the plan's check: Gemini built but not offered until six dictations pass; the card's link and Paste as two more capabilities; "Use the local model" dimmed when none is ready; what the `cloud` switch does when turned off.

The owner approved this design on 2026-11-10, and settled that nothing is sent before Stop.
