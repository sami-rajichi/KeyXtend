# 58. Speech has its own gate, and the microphone gate is Snip's

- **Date:** 2026-11-25
- **Status:** Accepted (owner, 2026-11-25). Supersedes ADR-0042 decision 8's "`Microphone` (the speech port)" for the voice module. ADR-0056 decision 10 still gives Snip `Microphone`, for its recorder's pass.

## Context

- ADR-0042 decision 8 gates the speech port with `Microphone`, and ADR-0056 decision 10 gives Snip `Microphone` for the microphone's pass to its recorder. So Snip's gate admitted speech, and voice's admitted Snip's pass.
- Only a manifest line and a test kept each module from the other's port. One manifest edit would have handed Snip a microphone path that ignores Snip's own Mic switch, with no capability or ADR change.

## Decision

1. **A `Speech` gate.** A new capability, `Speech`, gates `SpeechKey`: the worker's speech jobs, local and cloud dictation's recording, the devices and a card's check. Only the voice module holds it.
2. **`Microphone` is Snip's.** `Microphone` gates only `MicPassKey`, the app's pass without which the recorder never hears the microphone. Only the snip module holds it, and the voice module no longer does.
3. **`OpenFolder` stays shared.** `MediaOpenKey` keeps the `OpenFolder` gate, held by voice (ADR-0046) and Snip (ADR-0056). The voice module never asks for `MediaOpen`, and the snip module never asks for `Speech`.
4. **Exact holders.** The app's tests name each of these keys' holders and each capability's holders, never "the gate's holders", so a new holder fails a test.

## Consequences

- The policy, not only the manifests, keeps speech to the voice module and the microphone's pass to Snip.
- A grant of `Microphone` alone no longer opens speech, as the module API's contract tests check.
- Both worker jobs still hear the microphone, `voice` and `record` (ADR-0056); this changes only which module may ask for each.
