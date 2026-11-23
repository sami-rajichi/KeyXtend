# Test report: P11 voice typing, part 2 (cloud services)

- **Dates:** 2026-11-21 (first check), 2026-11-22 and 2026-11-23 (re-check of the fixes), on the development PC.
- **Windows:** Windows 11 Home Single Language 26H2, build 26300.
- **Display:** 1920×1080, 125 % (120 DPI), one screen.
- **Apps:** the owner's own text fields for the dictations; Qt 6.11.2.
- **Builds:**
  - `keyxtend`, release, staged, signed and installed under `Program Files\KeyXtend-dev` with uiAccess: 38b865b6 for the first check, 31e58f86 for the re-check.
  - The voice gate build (`--features gate-voice`) beside the gate worker (`--features gate,card`), driven by `kx-gates` with the `voice` feature, at fc4cbce8.
- **Keys:** the owner's own, pasted only into the Voice panel's key line; none was typed, logged or written anywhere else.
- **Load:** the owner's usual load (WSL, Docker), left running.
- **Scope:** every offered service with the owner's real key, the plan's hands-on checks, the fixes for what the first check found, and G22's cloud line.

## Gates

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | `g22 keyxtend`, local line (processor) | Each clip typed into target-window, which keeps the foreground; the bar above the keyboard, on top, never activated | Pass | 0 wrong in each language, twice; first text 1.4 s Arabic, 0.4 s English and French |
| 2 | `g22 keyxtend`, cloud line: Groq, ElevenLabs, Gemini, AssemblyAI | Each clip typed within `card_diff` (1 word), the window rules as row 1 | Pass | At most 1 wrong in any language, in both runs |
| 3 | The cloud line: Lemonfox | As row 2 | Reading | English and French pass; Arabic 2 of 10 wrong in both runs (note 4) |
| 4 | The cloud line: OpenRouter, Whisper Large v3 Turbo | As row 2 | Reading | Arabic and French pass; English 9 of 14 wrong in both runs (note 5) |
| 5 | The cloud line: Custom (Groq's address) | As row 2 | Reading | Refused as a bad key in each language (note 6) |

## First check (owner, installed 38b865b6, 2026-11-21)

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | Above the Start menu | The keyboard and the bar stay above it and never take focus | Pass | |
| 2 | The Cloud page | The seven offered services, each marked untested | Pass | |
| 3 | Groq: card, accept, On, key Paste, Test | The card's facts; Test works | Pass | The dots showed only after a click elsewhere (F1) |
| 4 | KeyXtend's history after a key Paste | No copy of the key | Pass | Note 1 |
| 5 | Groq dictation in English, French and Arabic | The cloud mark and Groq on the bar; Stop types the text | Pass | |
| 6 | Paste at once | The text at once; the clipboard keeps the earlier copy | Pass | |
| 7 | The gear while listening; Cancel before Stop | Stops and types nothing; Groq's usage unchanged | Pass | |
| 8 | Wi-Fi off at Stop, then Try again or the local model | The note offers both; each types once | Pass | |
| 9 | A password box with Groq in use | The local model writes, dots only | Pass | |
| 10 | Compare with Groq ticked | Groq's models run beside the local ones | Fail | They could not be ticked (F2) |
| 11 | Gemini: card warning, key, six dictations | The training warning; EN, FR and AR correct | Pass | `ar-SA` accepted |
| 12 | ElevenLabs: card warning, key, Scribe in EN, FR and AR | The training warning; correct text | Pass | |
| 13 | AssemblyAI: key, EN, FR and AR | Correct text | Pass | Its no-test note read as a refusal at first, as F1 hid the dots |
| 14 | Lemonfox: key, dictations | Correct text | Pass | Test on rows without a test call confused (F3) |
| 15 | Custom with Groq's address | Dictation through the typed address | Blocked | The address and model lines took no paste (F4); the key came first and the rest on a second page (F5) |
| 16 | Groq Off with a Groq pick | A note with Open service | Fail | The local model wrote with no note (F6) |
| 17 | The Device page on a layout change; the Arabic Cloud, card and key pages | Redrawn; correct right-to-left Arabic | Pass | |
| 18 | Aquatic contrast theme | Preparing wears a ring, only Recording fills | Pass | |
| 19 | Narrator on the Cloud page, card and key page | It says each part's name | Pass | |
| 20 | The voice helper after the last dictation | It closes when idle | Pass | Gone 180 s into the watch |

## Re-check of the fixes (owner, installed 31e58f86)

| # | Fix | Action | Result | Notes |
|---|---|---|---|---|
| R1 | F1 | Groq key page: Paste; the dots show at once | Pass | |
| R2 | F2 | Compare with a Groq model ticked: the warning names Groq; its result shows | Pass | |
| R3 | F3 | AssemblyAI and Lemonfox key pages show no Test; the note stays | Pass | |
| R4 | F4 | Paste a model id on Other model, Save | Pass | |
| R5 | F5 | Custom: address, model and key on one page; Groq's values; English dictation | Pass | |
| R6 | F6 | Groq Off with a Groq pick: Home names the model; Start shows the Off note with Open service | Pass | |
| R7 | F6 | Models from Home opens on the Cloud side, on the pick's page | Pass | |
| R8 | F7 | Key Paste, then a Paste at once dictation: pasted at once, the clipboard left empty | Pass | |
| R9 | — | OpenRouter: card, key, Test, Turbo in EN, FR and AR; GPT Transcribe with keep-nothing shows the note | Pass | Note 2 |
| R10 | — | Remove key with the service On, then Off, then Paste it again | Pass | |
| R11 | — | G22's cloud line | See Gates rows 2 to 5 | Two runs, 2026-11-23 |
| R12 | — | Lemonfox in French and Arabic | Covered | By the first check |
| R13 | — | A local model on the bar: the device picture and the short name beside the language | Pass | Parakeet in English, the Arabic model in Arabic |
| R14 | — | Each language's cloud pick stays after a restart, a service Off then On, and the Cloud switch Off then On | Pass | Note 3 |
| R15 | — | Every service once more in EN, FR and AR | Covered | By the first check |
| R16 | — | A first dictation's warning with Gemini or ElevenLabs; Stop | Pass | Cancel and Stop never move, and the warning stays until the text |
| R17 | — | Custom on the bar in each theme | Pass | Custom, Personnalisé and مخصّص show whole |
| R18 | — | OpenRouter: Test with a management key shows its own note | Skipped | The owner's choice; note 2 |

## Notes

1. KeyXtend's history forgets the newest copy when a key is pasted on the key line. Windows' own clipboard history (Win+V) is outside KeyXtend; the owner clears it after pasting keys.
2. A management key made on OpenRouter's Management keys page passed Test but could never dictate, as OpenRouter's docs say. Test now fails such a key with its own note, read from the reply fields OpenRouter's row names. The owner skipped the live check of that note, so only tests on OpenRouter's documented fields cover it; a regular API key dictates.
3. A pick could be lost when a service went Off; an Off service now keeps its typed values and picks aside until it is turned on again.
4. Lemonfox's one model, Whisper Large v3, wrote the Arabic clip with 2 words wrong where the gate allows 1. That is the model's accuracy; the owner's own Arabic dictations with Lemonfox passed on 2026-11-21, so the row stays offered (ADR-0050).
5. KeyXtend sends OpenRouter the same form as Groq, whose Turbo wrote the English clip with no word wrong; OpenRouter passes the request to a host of its own choosing. The owner's English dictation with Turbo passed by hand, so the row stays offered; the finding is in `docs/items/P11.md`.
6. The daily Custom row points at Groq's address, which refused Custom's saved key while Groq's own row's key worked. The owner passed Custom by hand at R5 and chose to leave the saved key as it is.
