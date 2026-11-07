# 39. The vault's data services need a capability of their own

- **Date:** 2026-11-02
- **Status:** Accepted. Supersedes ADR-0038 decision 3 for the services only; its event gate stands.

## Context

- ADR-0038 decision 3 let `Vault` gate both the Quick-fill events and the notebook store, the `Boxes` and the `Places` ports.
- The keyboard, prediction and the ui hold `Vault` as an event gate. The kernel checks only the capability (`kx-kernel/src/registry.rs`), so any of them could reach the notebook by adding a key to its `requires`.
- ADR-0035 decision 2 says a module reaches each service its capabilities allow, so one shared capability shares the services too.

## Decision

1. **`VaultData`.** A new capability gates the notebook store (`VaultStoreKey`), `BoxesKey` and `PlacesKey`. The vault module alone holds it.
2. **`Vault` is the event gate.** It still gates every Quick-fill event that carries values. The keyboard, prediction and the ui keep it for those events only.

## Consequences

- A module that only shows or types Quick-fill's values cannot open the notebook or a backup box, even if it asks.
- The ships tests check that only the vault holds `VaultData` and that the three services refuse the other modules.
