# 35. The clipboard panel's state needs ReadClipboard, and services follow their gates

- **Date:** 2026-10-31
- **Status:** Accepted. Supersedes ADR-0034 decision 1's gate list; the rest of ADR-0034 stands.

## Context

- ADR-0034 left `ClipPanel` without a gate, as it carries only a flag.
- While `{ open: true }` holds, the keyboard turns every navigation and character key into `ClipKey` instead of typing. So any module could mute the keyboard, and with no clipboard module running only a keyboard restart brought typing back.
- Every real sender and listener of `ClipPanel` (the clipboard module, the keyboard and the ui) already holds `ReadClipboard`.
- ADR-0034 said the keyboard holds both clipboard capabilities "as event gates only". The host grants services by capability, so the keyboard and the ui can reach every service their gates open; the life tests already pin that.

## Decision

1. `ClipPanel` needs `ReadClipboard`, beside `ClipKey`, `ClipView` and `ClipAct`. `ClipOpen`, `HistoryWanted` and `HistoryAsk` stay open, as they turn no key and carry nothing.
2. A module reaches each service its capabilities allow. So the keyboard and the ui, which hold `ReadClipboard`, may reach `Clipboard`, and the keyboard, which holds `WriteClipboard`, may reach `ClipboardWriter`, though neither asks for one today.
3. The keyboard needs `ReadClipboard` to start, as it already needs `WriteClipboard`: the bus refuses its `ClipPanel` listener without it.

## Consequences

- Only a module that may read the clipboard can open the panel's keys; the contract tests check the refusal.
- A keyboard granted neither clipboard capability fails its start with a clear refusal instead of typing into a panel it cannot hear.
