# 60. Videos have no editor, and the strip plays and speeds them

- **Date:** 2026-11-26
- **Status:** Accepted (owner, 2026-11-26). Supersedes the video parts of ADR-0055 and ADR-0056 only where they name the editor; their other lines stand.

## Context

- The P12 design opened a recorded video in the editor, with Play, the speeds 1×, 2× and 4×, its length at that speed, Copy and Save ([`2026-11-23-snip-design.md`](../superpowers/specs/2026-11-23-snip-design.md)). ADR-0055 decision 2 lets a video be played and sped up, and ADR-0056 decisions 3 and 10 give the speed change and Play.
- At the hands-on check on 2026-11-26, Edit on a video's strip opened that editor, and Play opened the laptop's player under our topmost editor, where the owner could not see it.
- The owner: a video editor that only plays elsewhere does not earn its window. The strip should carry the choices, and our keyboard and the small strip should stay on top.

## Decision

Decided by the owner on 2026-11-26.

1. **No editor for a video.** After Stop, or a recording that ends by itself, the strip shows the video's length, then Play, Speed, Copy, Save and Close, with no Edit. The editor opens for pictures only.
2. **Speed is one key.** Each click steps through `SNIP_SPEEDS` (1×, 2×, 4×) and back to the first. The key shows its speed after its word, and the length shows at that speed. A speed change counts as a change, so a kept video is no longer kept.
3. **Copy and Save use the picked speed.** The `record` job's speed change makes the copy (ADR-0056 decision 3), so a Copy at 2× followed by Speed 4× copies again at 4×.
4. **The strip stays after Copy or Save.** It shows the tick and done word for `shown_ms`, so the user can pick another speed or the other key. Close ends the snip, and asks first when nothing was kept since the last change.
5. **Play opens the user's own player.** It goes through `MediaOpen` and the running Explorer, as before (ADR-0056 decision 10). Our strip and keyboard keep their place on top. A refused open shows the `PlayFailed` note, and the strip stays.

## Consequences

- The editor loses its video row, its poster and its title for a video. `SnipPlay.qml`, `VideoEdit`, `EditBody`, `EditKey::Play`, `SnipClick::Speed`, `SnipClick::Keep` and the editor's video texts go.
- `StripKey` gains `Play` and `Speed`. `Taken` carries the speed, and its length is given at that speed. New texts: `snip.strip-play`, `snip.strip-speed` and `snip.speed-times`.
- G24's video run clicks the strip's Speed, Copy, Save and Close instead of the editor's speeds and keys (`snip.strip.speed` and `snip.strip.play` in place of `snip.speed.<n>`).
- Speed's icon is a stand-in, Lucide `chevron-right`. A fast-forward mark needs a new icon file and its licence check first.
- Only the owner's live run shows the player in front of the desktop while our strip and keyboard stay above it.

## Alternatives considered

- **Keep the editor for a video:** it would hold three keys the strip can hold, and its topmost window hid the player.
- **Three speed keys on the strip:** that is a wider strip for a rare choice. One key that steps keeps the strip as short as a picture's.
- **End the snip after a video's Copy or Save, as a picture's strip does:** a second Copy at another speed would then need a new recording.
