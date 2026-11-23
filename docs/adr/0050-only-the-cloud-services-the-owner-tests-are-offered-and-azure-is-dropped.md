# 50. Only the cloud services the owner tests by hand are offered, and Azure is dropped

- **Date:** 2026-11-19
- **Status:** Accepted. Narrows ADR-0047 decision 5 (four request shapes become three) and the services it plans, drops decision 6's Azure host, and replaces decision 7's Gemini line; its other decisions apply as written.

## Context

- ADR-0047 planned eleven services through four request shapes, and the table offered every upload-form row built so far. A connection no real key has checked ships marked untested.
- OpenAI, Mistral, Together AI and DeepInfra need a card or a payment before any use, so the owner cannot test them by hand.
- Azure needs a request shape of its own and a trial account. Its MAI-Transcribe-2 model is already reachable through the OpenRouter row.

## Decision

Decided with the owner on 2026-11-19.

1. **Only tested services are offered.** A service is offered only when the owner can test it by hand. Groq, Lemonfox, OpenRouter and Custom are offered now; ElevenLabs and Gemini follow with their own rows.
2. **The rest stay in the table, hidden.** OpenAI, Mistral, Together AI and DeepInfra keep their rows with `offered = false`, and their code, texts and tests stay, so one can be switched on after a hands-on test. A row not offered never shows on the Cloud page, cannot be confirmed, picked or ticked in Compare, and nothing is sent to it.
3. **A saved entry falls back.** An entry or pick for a row not offered, from a hand edit or an older settings file, is dropped alone and named at the next read. Its languages dictate with their local model and never move to another service, as for any row no longer offered (ADR-0047 decision 12).
4. **The hands-on check decides.** Every offered row ships marked untested until the final hands-on check, where any offered row the owner cannot test, or that fails, is hidden the same way. Custom stays offered: the owner tests it with Groq's address and model typed into it.
   - This replaces ADR-0047 decision 7's line that Gemini waits for six dictations by the owner: Gemini is offered and marked untested like every other row.
5. **Azure is dropped.** There is no Azure row and no Azure request shape. ADR-0047's four request shapes become three: the upload form, ElevenLabs' form and Gemini's single request.

## Consequences

- Every service a user can switch on is one the owner has used by hand.
- Switching a hidden row on is one table line after its hands-on test, and a test pins the offered set, so no row is shown or hidden by accident.
- The fit test still measures every row's card, hidden ones too, so a row switched on later already fits the panel.
- MAI-Transcribe-2 is reached only through OpenRouter, with OpenRouter's terms, fee and keep-nothing switch.
