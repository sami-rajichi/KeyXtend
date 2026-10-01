# 20. The keyboard's text follows the keyboard's language

- **Date:** 2026-10-01
- **Status:** Accepted. Supersedes ADR-0017's language choice ("`auto` follows the Windows display language") for the keyboard's text; the rest of ADR-0017 stands.

## Context

ADR-0017 chose one UI language: the Windows display language, unless `[ui] language` named one.

- On the owner's PC, Windows shows French while the owner types in English, so French key words (Échap, Suppr, Maj) sat over an English layout.
- A physical keyboard prints the words of its own layout, and the language key already shows the layout in front.
- The tray icon belongs to the taskbar, which speaks the system's language.

## Decision

Decided by the owner on 2026-10-01.

1. **Key words follow the keyboard's language.** The words on keys (`label.*`) and the space bar follow the language of the layout in front, the one the language key shows. An English layout gets the English words, a French layout the French words, and an Arabic layout the English words Arabic keyboards print (ADR-0017).
2. **So does everything on and around the keyboard:** tips, screen-reader names, window titles, the notice banner, the language list, the size letters and the top bar, and later the Settings window.
3. **The tray follows the system.** The tray icon's tooltip and its menu speak the system's display language (`SystemLook::ui_language`).
4. **Arabic flips text only.** Keys, the top bar, the side block and every window keep their places; Arabic text reads right to left in the theme's Arabic font. Nothing uses `LayoutMirroring`.
5. **Fallbacks.** A layout whose language has no texts, such as German, gets English. So does a layout whose language Windows cannot name, and the time before the keyboard starts: the keyboard's text never follows the system's language.

How it works:

- `LangLabel::iso` carries the layout's ISO 639 code from `kx-mod-layouts`, and `KeyboardFace::lang` hands the shown layout's code to the UI.
- The hub keeps two text sets: the keyboard's and the system's. After an action whose state changed, it asks the face for the language; a new one switches the keyboard's set and sends `Wake::Texts`, and QML reads only the texts again (`textsChanged`).
- The banner's notices are told again in the new language (`shown_notices`), and a tip shown in the last language closes.
- `[ui] language` keeps its values: `auto` now follows the keyboard's language, and `en`, `fr` or `ar` pins the keyboard's text. The tray always follows the system.

## Consequences

- The words switch as soon as the relabel timer sees the app in front switch layout, and nothing moves.
- Window titles change with the language. The shape and the backdrop find a window by its live title at each call, so they follow; the guard uses no title.
- Each switch builds one text set from its compiled-in Fluent file; switching back builds it again.
- ARCHITECTURE's rule "right-to-left layouts use `LayoutMirroring`" ends; gate G7's right-to-left panel now means right-to-left text in a panel that keeps its places.

## Alternatives

- **Keep following Windows:** French words over an English layout; refused by the owner.
- **The keyboard's language for the tray too:** the tray belongs to the system's taskbar; refused (decision 3).
- **Mirror the top bar and the pop-ups in Arabic, as Windows does:** the controls would move under the user's finger at each layout switch; refused (decision 4).
- **QML reads the language from the state and tells the hub:** the choice would live in QML, where the hub's and the app's tests cannot see it; refused for a port call.
