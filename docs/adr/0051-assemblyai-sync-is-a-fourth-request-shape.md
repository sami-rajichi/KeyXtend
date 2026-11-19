# 51. AssemblyAI's Sync API is a fourth request shape

- **Date:** 2026-11-19
- **Status:** Accepted. Adds a fourth request shape to ADR-0050 decision 5's three; ADR-0047 and ADR-0050 apply as written otherwise.

## Context

- ADR-0050 left three request shapes: the upload form, ElevenLabs' form and Gemini's single request.
- On 2026-11-19 the owner asked for AssemblyAI. Its [Sync API](https://assemblyai.com/docs/api-reference/sync-api/transcribe-live.md) returns a piece's text in one request, with no upload, job or poll; its pages were read that day.
- No existing shape fits it. Its body is multipart, but its first part is a JSON `config`, its sound part is `audio`, and the model goes in its own header, `X-AAI-Model`, not in a field.
- Its [European servers do not train](https://www.assemblyai.com/docs/data-retention-and-model-training) on what they receive. Its newer STT-3.7 model needs the classic upload, job and poll requests, which the plan keeps out of scope.

## Decision

Decided with the owner on 2026-11-19.

1. **A fourth shape.** `assembly` joins the three: one multipart POST a piece, the JSON `config` part first, holding only the keyboard's one language code, then the piece as the WAV `audio` part. The key goes in `Authorization` and the model in the header the row names.
2. **The row names its model header.** The services table gives the shape's row a `model_header`, which the frames carry as a plain name and read strictly, under shape code 5. It never names the key's header or one the request sets itself, and no other shape takes one.
3. **Universal-3.6 Pro on the European servers.** The row sends only to `sync.eu.assemblyai.com`, with the only Sync model, chosen by the owner for its single request. STT-3.7 waits as a later idea.
4. **No key test.** The Sync host documents no call without sound, so the card says the first dictation tests the key, and the shape never sends a test request.
5. **Offered, untested.** Like every row, it is offered and the owner tests it by hand at the final check; it is hidden there if it fails (ADR-0050 decision 4).

## Consequences

- The worker holds four adapters; `shape::of` stays total, so a fifth shape is again one more file and one more arm.
- A piece is cut at two minutes, the Sync API's live limit, so it is expected to finish within the 30-second server deadline. The owner checks that by hand with a real key at task 24, with the rates, the credit reply and whether the free credit covers Sync.
- No new crate: the shape reuses the shared multipart body, status table, reply reader and `serde_json`.
