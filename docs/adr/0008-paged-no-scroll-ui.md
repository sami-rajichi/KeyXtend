# 8. Every UI works without scrolling, dragging or right-click

- **Date:** 2026-09-26
- **Status:** Accepted

## Context
Target users have no scroll wheel and cannot drag. In mock-up v1 the clipboard and quick-fill panels were unusable because they needed scrolling.

## Decision
- Lists show 5 rows per page, with big ▲/▼ page buttons and a "page n of m" indicator. A short last page is padded.
- The keyboard's arrows, Enter, Space and Esc drive any open panel.
- The scroll pad can also scroll our own lists.
- Use −/+ steppers instead of sliders.
- Panels scale with the keyboard size, but not below 0.8×.

## Consequences
UI review checklist item: "Can every action in this view be done with short left clicks only?"
