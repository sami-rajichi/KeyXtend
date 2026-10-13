# 23. Give freed heap pages back after a look change

- **Date:** 2026-10-11
- **Status:** Accepted. Adds to ADR-0021 how memory stays flat over G8's churn; ADR-0021's decision 6 stands.

## Context

- G8 allows at most 3 MB of growth over 20 churn rounds. Each round shows key tips, the List design's language list and the bubble, and presses the theme button once per theme (ADR-0021, `tools/kx-gates/kx-gates-p3.toml` `[g8]`, `tools/kx-gates/src/g8pops.rs`).
- A look change frees a lot at once: each window's cached graphics for the last look, and the last theme's fonts.
- With the three theme leaks fixed, 20 rounds still grew 4.5 MB of working set and 4.6 MB of commit, while the heap blocks in use stayed flat. Windows' standard heap keeps the freed memory committed for reuse.
- Most of that growth is free heap space in pieces under 4 KB, scattered between blocks in use, which a decommit cannot return; only whole free pages can go back.

## Decision

Decided by the owner on 2026-10-11.

1. **Give free pages back.** After a look change, once every shown window has dropped its cached graphics (after its next frame, or on hiding), the app asks the OS once to take back the heaps' free pages. Back-to-back changes merge into one give-back, once no window still waits. Nothing is asked at idle.
2. **On Windows**, this is `HeapSetInformation` with `HeapOptimizeResources` (Windows 8.1 or later) on the process heaps. With no heap named, Windows takes every heap with a low-fragmentation front end.
3. **Not a working-set trim.** Only free pages are decommitted; pages in use stay where they are. G8's measure stays honest, and ADR-0021 decision 6 holds.
4. **A port.** `kx-platform`'s `Memory` port does it. The Windows adapter makes the call, the fake counts its calls, and other systems keep the default, which does nothing. A refusal is logged once and never stops the app.

## Consequences

- Each run read as G8 reads it, 20 rounds after one warm round grew a median of +3.1 MB of working set and +2.3 MB of commit with the give-back, against +4.2 / +4.2 without it (4 runs each, brief D). Two of the 4 runs went over 3 MB, so G8 is borderline to failing with one warm-up round.
- Five warm-up rounds do not pass reliably either. On a later build, 4 interleaved runs of each grew a median of +1.6 / +2.4 MB with five (1.2–2.6 / 0.3–4.7), against +4.0 / +7.3 with one (3.1–5.2 / 1.4–9.7). That is more than brief D's with one, for no known cause; the PC was in use. One run with five still grew 4.7 MB of commit. The owner chose five warm-up rounds anyway, as memory settles over them, and G8 now lists each round's memory, so a failed run shows a steady climb (a leak) or a single jump (settling).
- The segment heap with five warm-up rounds grew a median of +1.8 / +2.0 MB (1.2–2.9 / 1.1–3.8), with one run of 4 over 3 MB too. After the warm-up it held about the same commit as the standard heap and 4–14 MB more working set.
- Each look change costs one more call, after its windows' release: a median of 0.5 ms on the GUI thread and at most 1.8 ms over 552 calls; with the segment heap, a median of 2.6 ms and at most 7.4 ms. Pages given back cost a page fault when the heap needs them again.
- The device test checks one give-back per look change, one for a window that hid before its frame, and none at idle; the Windows adapter's test calls it for real.

## Alternatives

- **The segment heap** (D2), set in the manifest: about 5–7 MB less commit with no warm-up, but about the same after five warm-up rounds. It is process-wide and changes the speed of every allocation, so it needs a speed check first. Left for later as a P13 idea.
- **Trim the working set** (`EmptyWorkingSet`, `SetProcessWorkingSetSize`): it hides memory from G8 without freeing it; ADR-0021 decision 6 refuses it.
- **Leave it:** G8's growth limit fails.
