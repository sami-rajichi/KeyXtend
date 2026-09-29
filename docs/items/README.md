# Item notes

One file per item from `docs/FEATURES.md` (`P2.md`, `F3.md`…). Each item is built in its own session, and its file is how one session tells the next what is done and what is waiting.

## What a file holds

- **Already built:** what exists before the item starts, usually a link to its part of `docs/spike-move-map.md`.
- **Notes:** findings and decisions from earlier sessions that this item must handle.
- **Ideas:** improvements the owner asked for, each dated; a built idea is marked `Built (date)`.

## Rules

- The session that builds an item reads its file first, and removes each note once it is handled.
- A finding that belongs to another item goes into that item's file, not into the current work.
- When the owner says "idea for P13: …" in any session, it is added under P13's **Ideas** and the current work carries on.
- `/kx-feature P13` on a finished item builds its saved ideas, in their own session.
- An F-item's file is created when it first has something to hold.
- These files are public: no personal details, no secrets.
