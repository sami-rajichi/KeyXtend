# 52. A card's check reads the clip of the language in front

- **Date:** 2026-11-20
- **Status:** Accepted. Supersedes ADR-0044 decision 7's clip choice and its "kept per card, driver version and model"; that decision's clips, passing rule and other lines, and ADR-0044's other decisions, stand.

## Context

- ADR-0044 decision 7 checks a card on the first shipped clip whose language the model knows, in the order English, French, Arabic, and keeps one result per card, driver and model.
- So the Cohere Arabic model, which also knows English, is checked on the English clip. A card that went wrong only in Arabic would pass, and Arabic would then run on it.
- The task 20 investigation (2026-11-20) found this gap while tracing a gate fault. No user saw it, but nothing caught it either.
- The check is started for the model the language in front uses, so that language is known when it starts.

## Decision

Decided with the owner on 2026-11-20 ("check each language").

1. **The clip.** The check reads the shipped clip of the language in front when the model knows that language. Otherwise it reads the first clip the model knows, in the shipped order.
2. **What a result counts for.** A result counts for the card, its driver, the model file and the language of the clip it ran on. A model checked in English is checked again before the card serves it in Arabic.
3. **Older results.** A result kept before this decision names no language. It counts for none, so each language checks once again; its language is never guessed. The checks' file still loads.
4. **A change mid-check.** A result is kept for its clip's language, even if the language in front changed while it ran.
5. **The words apart.** The "a card's check ended" debug line also holds how many words apart the two texts were, a number only.

## Consequences

- Each language checks a card once, the first time the card would serve it. That adds a few seconds per language; the processor serves meanwhile, under the waits a check already has.
- A card that fails in Arabic but passed in English runs Arabic on the processor and keeps English on the card.
- A failure stops only its own card, driver, model and language: under Automatic, a model that failed in English is still checked in Arabic and French, and a failure with one model never stops another's check. Automatic still tries a card for a new model only once it passed a check (ADR-0044 decision 8).
- A driver update checks every language again, as before. A card that fails mid-run drops that file's results in every language.
- The checks' file keeps its first form, so an older app still reads it, as results of no language. Each result's language is kept in a file beside it (`numbers/check-langs`) that older apps never open; a result takes it back only when it is identical there in every field (card, driver, file, verdict with its numbers, and day), so one an older app rewrote any other way names none.
- The checks' file lists each card and file's passes before its other results. An older app keeps the last one, so after a downgrade it runs that model on the processor whenever any language did not pass.
