# 54. A voice choice stays until the owner changes it

- **Date:** 2026-11-22
- **Status:** Accepted. Supersedes ADR-0047 decision 9's "picks back to the local default", and the cloud spec's and plan's edges 16 and 21 where they drop cloud picks; ADR-0047's other decisions stand, decision 12 among them.

## Context

- At the hands-on check, the owner picked Groq for English, then switched Groq Off. The installed build (38b865b6) dropped Groq's picks at Off, so English fell back to a local model and stayed there after Groq came back.
- `9c06cd8b` already keeps a service's listed picks waiting while it is Off (owner, 2026-11-21). The `cloud` switch, declined terms and the start-up repair still dropped cloud picks.
- The owner asked on 2026-11-22 that a choice they made stays until they change it themselves.

## Decision

Decided by the owner on 2026-11-22.

1. **The Cloud switch Off** keeps every cloud pick waiting: local models serve, and nothing is sent. Cloud On again brings each language back to its cloud pick. This is the owner's own act, so ADR-0047 decision 12 ("nothing moves to cloud on its own") still holds.
2. **New terms declined** turns the service Off, and its picks wait for its Confirm, as a service Off does. Accepting the terms later brings them back.
3. **Side effects keep the choice.** Only the owner changing that very choice drops it, such as a new local pick, a cleared typed address or a new typed model id. Any other act or repair leaves the choice waiting, marked or unavailable, as the screens already show.
4. **A service Off keeps its entry aside.** Off and declined terms move the service's entry, with its typed address, typed model and keep-nothing switch, from `services` to a new `[voice] parked` map, where nothing sends to it or dictates with it; only the waiting row, the start repair and its Confirm read it. Its typed picks wait as listed ones do, and its Confirm moves the entry back with the new day and terms number and shows the card's warning again at the first dictation.

## Consequences

- The start-up repair keeps a cloud pick while cloud is off, and keeps a listed pick when its service entry is dropped. It still drops a pick whose row or model the table no longer offers, and names it in the settings notice.
- While a cloud pick waits with cloud off, the language dictates with its local default; Start shows no cloud note, as cloud offers nothing.
- An older version reads a waiting pick as before: with cloud off it drops it at start with the settings notice.
- The start-up repair checks a parked entry as an On one, and drops it, named `parked.<id>`, when its row is gone or not offered, or when the service is also On.
- An older version cannot read `parked`: its section refuses a key it does not know, so it resets that key with the settings notice and the parked entries are lost there, while its other values stay.
