# kx-mod-selection

The selection chips (P10, ADR-0041): selected text in another app shows a strip of keycaps, Copy, Cut, Paste and Translate, by its last line; a click into an editable box shows Paste by the caret. The module reads no text before a click: only Translate reads the selection, once, and opens the chosen site's page in the user's browser after one consent. It never types; the keyboard sends the edit keys for it.

This crate holds the module's pure rules so far: which keycaps show (`chips`), the strip's show and hide state machine on `Mono` time (`Strip`), the Translate link (`Site::link`, `Site::target`) and the compiled-in site table (`sites.toml`). The actor that runs them on the bus comes next (P10 plan, task 10).

## Services

| Service | Direction | Use |
|---|---|---|
| `SelectionWatch` (`SelectionWatchKey`) | needs | Tells focus and selection changes, looks at the focused box with no text, and reads the selection once for Translate. |
| `WebLink` (`WebLinkKey`) | needs | Hands the Translate link to the desktop shell, which opens the user's browser. |

It provides no service.

## Capabilities

| Capability | Why |
|---|---|
| `ReadFocusedText` | It gates the selection watch, Translate's one read and `AimedLine`. |
| `Selection` | It gates the chips' events: `ChipsView`, `ChipAct`, `EditAsk` and `EditDone`. |
| `OpenLink` | It gates the browser link; only this module holds it. |

It holds no `InjectInput`.

## Settings (`[selection]`)

| Key | Default | Range | Meaning |
|---|---|---|---|
| `on` | true | | The whole helper. |
| `copy`, `cut`, `paste`, `translate` | true | | Each keycap. |
| `caret_paste` | true | | Paste by a caret just clicked into. |
| `caret_s` | 5 | 2–30 | Seconds the caret's Paste stays. |
| `icons`, `words` | true | never both off | Each keycap's icon and word. |
| `minimised` | true | | The chips show while the keyboard is minimised. |
| `translate_to` | "" | a short tag of letters and hyphens | Translate's language; empty follows the system's display language. |
| `translate_site` | "google" | an id in `sites.toml` | The translate site. |
| `translate_chars` | 1000 | 50–5000, within the site's limit | The most characters Translate sends. |
| `translate_ok` | false | | The user said yes to sending selected text to the site. |
| `settle_ms` | 120 | 30–1000 | The wait after a change before a look. |
| `read_ms` | 250 | 50–2000 | The longest one look or read may take. |
| `looks_per_s` | 10 | 1–30 | The most looks in one second. |
| `consent_s` | 30 | 5–120 | Seconds a Translate question waits for its answer. |

A bad value goes back to its default with the settings notice.

## Sites (`sites.toml`)

Each row has an id, a name, a link pattern with `{to}` and `{text}` once each, a character limit, the tags it takes whole, a fallback target and `link_max`, the most bytes of the whole encoded link. The table is checked at its first use; a bad table turns Translate off and never panics.
