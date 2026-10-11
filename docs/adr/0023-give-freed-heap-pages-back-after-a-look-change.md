# 23. Give freed heap pages back after a look change

- **Date:** 2026-10-11
- **Status:** Accepted. Adds to ADR-0021 how memory stays flat over G8's churn; ADR-0021's decision 6 stands.

## Context

- G8 allows at most 3 MB of growth over 20 churn rounds, and each round presses the theme button once per theme (ADR-0021, `tools/kx-gates/kx-gates-p3.toml` `[g8]`).
- A look change frees a lot at once: each window's cached graphics for the last look, and the last theme's fonts.
- With the three theme leaks fixed, 20 rounds still grew 4.5 MB of working set and 4.6 MB of commit. The memory is freed, but Windows' standard heap keeps the freed pages committed for reuse.

## Decision

Decided by the owner on 2026-10-11.

1. **Give free pages back.** After a look change has been drawn and every shown window has dropped its cached graphics, the app asks the OS once to take back the heaps' free pages. Nothing is asked at idle.
2. **On Windows**, this is `HeapSetInformation` with `HeapOptimizeResources` (Windows 8.1 or later) on the process heaps. With no heap named, Windows takes every heap with a low-fragmentation front end.
3. **Not a working-set trim.** Only free pages are decommitted; pages in use stay where they are. G8's measure stays honest, and ADR-0021 decision 6 holds.
4. **A port.** `kx-platform`'s `Memory` port does it. The Windows adapter makes the call, the fake counts its calls, and other systems keep the default, which does nothing. A refusal is logged once and never stops the app.

## Consequences

- Brief C's probe of this call measured the growth over 20 rounds at +1.0 MB of working set and −0.1 MB of commit, against +4.5 / +4.6 MB without it.
- Each look change costs one more call, after its windows' release; pages given back cost a page fault when the heap needs them again.
- The device test checks one give-back per look change and none at idle; the Windows adapter's test calls it for real.

## Alternatives

- **The segment heap** (D2), set in the manifest: about 5–7 MB less commit, but it is process-wide and changes the speed of every allocation, so it needs a speed check first. Left for later as a P13 idea.
- **Trim the working set** (`EmptyWorkingSet`, `SetProcessWorkingSetSize`): it hides memory from G8 without freeing it; ADR-0021 decision 6 refuses it.
- **Leave it:** G8's growth limit fails.
