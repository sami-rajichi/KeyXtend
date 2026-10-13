# 22. Texts carry no bidi marks

- **Date:** 2026-10-10
- **Status:** Accepted. Supersedes ADR-0017's bidi rule ("Fluent's isolation marks stay on") and its Arabic list join ("، ").

## Context

ADR-0017 kept Fluent's isolation marks (U+2068 and U+2069) around each argument, so a Latin word inside Arabic text kept its order.

- **Memory:** no theme font has a glyph for these marks, so Qt searches every system font for one, at a cost of 15–17 MB (QTBUG-14856).
- **Order:** a pixel check of every text with arguments, in EN, FR and AR and in each theme, found that the marks themselves turned Arabic around.
  - Unicode skips marked text when it picks a line's direction (UAX #9, rules P2–P3). The space bar `{ $lang } · { $layout }` has no letter outside its marks, so Qt laid it out left to right, and an Arabic reader met the layout first.
  - Fluent nests marks in lists, so `و3 إعدادات أخرى` came before the setting names it follows.
- **Looks:** the marks split a text run, which moved ET66's full stop after a list by about 1 px. Dolch's lines took the taller line of the system font Qt borrowed for the marks.

## Decision

Decided by the owner on 2026-10-10.

1. **No isolation marks.** Fluent's isolation is off (`i18n.rs`), so the first letter of a text sets its reading order, as in any plain text.
2. **No other direction marks either, for now.** ET66's and Dolch's fonts (IBM Plex Sans, IBM Plex Sans Arabic, Rubik) lack LRM and RLM, so these would bring the font search back there. Native's fonts (Segoe UI, Segoe UI Variable, Noto Sans Arabic) have them, which leaves Native a cheap fix for the known gap below.
3. **OS names are cleaned.** The space bar's parts and the language list's names lose their hidden marks, as notice arguments already did, and a part that held only marks is left out.
4. **Accepted changes:** Arabic texts now read in Arabic order. ET66's full stop in EN and FR notices moves about 1 px, and Dolch's lines are 2–3 px shorter.
5. **Arabic lists join with " و ", not "، ":** after the comma, Latin setting names ran left to right as one run; the owner chose و (2026-10-10).

## Consequences

- Each language's own texts read in their order. `tst_order` checks it from where Qt lays out each part.
- **Known gap:** when a UI's direction differs from the layout's language, the space bar can read in the other order, with the layout first.
  - New with the marks gone: an EN or FR UI pinned by `[ui] language` over an Arabic layout, and a right-to-left layout without UI texts, such as Hebrew or Persian, which speaks English.
  - Already there with the marks: an Arabic UI pinned over a Latin layout.
  - The fix is a P13 note: draw the parts as separate texts in the UI's order, or in Native, add LRM or RLM. `tst_order` expects these cases to fail until then.
- No text the hub fills in, and no OS name the keyboard draws, holds a bidi control. The texts test checks every filled text, and that QML draws OS names only through the hub's cleaning.

## Alternatives

- **A tiny bundled font that draws the marks as nothing:** it saves the memory, but it keeps the wrong Arabic order. Matching Dolch's pictures exactly would also need it to copy the borrowed font's line height. Refused.
- **Direction marks around the whole text:** ET66's and Dolch's fonts lack LRM and RLM, so there they trigger the same font search. Refused for now; Native's fonts have them, so the P13 note keeps that option for Native.
- **A base direction on Qt Quick `Text`:** Qt Quick has no such property; `LayoutMirroring` changes only alignment.
