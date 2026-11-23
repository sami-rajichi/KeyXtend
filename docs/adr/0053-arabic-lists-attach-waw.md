# 53. Arabic lists attach و to the next word

- **Date:** 2026-11-21
- **Status:** Accepted. Supersedes ADR-0022 decision 5's loose " و "; that decision's reason for using و rather than "، " stands, as do ADR-0022's other decisions.

## Context

- ADR-0022 decision 5 joined Arabic lists with a loose " و ", since after "،" Latin setting names ran left to right as one run.
- Written Arabic attaches و to the next word. Compare's send warning already joins service names with CLDR's Arabic pattern "{0} و{1}", which the owner approved on 2026-11-21.
- The comma was tried again on 2026-11-21, and `tst_order` caught the swapped names in the reset notice.

## Decision

Decided by the owner on 2026-11-21 ("Attached و").

1. **One joiner.** `list.join`, `list.and-join` and `list.and` in `ar.ftl` are all " و": a space, then و joined to the next word.
2. **Never the comma.** Arabic lists keep و between names, never "،", as ADR-0022 decision 5 found.

## Consequences

- One rule for every Arabic list: "bubble_corner وerase_gap_ms و3 إعدادات أخرى".
- A Latin name follows و directly, as the owner approved for service names ("Groq وElevenLabs").
- Each name keeps its place in the Arabic order, which `tst_order` checks.
