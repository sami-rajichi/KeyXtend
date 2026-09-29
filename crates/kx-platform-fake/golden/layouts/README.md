Layout tables for `FakeLayoutSource`: one TOML file per layout with `id`, `controls`, `[facts]` and a `[keys.<scan>]` table of `KeyChars` per key; every code is `0x` hex text and each scan code appears once (see `LayoutFile` in `src/layout_file.rs`).

| File | Layout | Made by |
|---|---|---|
| `en-us.toml` | US, on English (United States) | `kx-gates capture-layouts` |
| `fr-fr.toml` | French (Legacy, AZERTY), on French (France) | `kx-gates capture-layouts` |
| `ar-tn.toml` | Arabic (102) AZERTY, on Arabic (Tunisia), right to left | `kx-gates capture-layouts` |
| `example.toml` | A small slice of French AZERTY for the fake's own tests | Hand-written |

The captured tables come from real Windows layouts through the product's `LayoutSource`: the command reads the first installed layout of each language named in `[capture]` of `tools/kx-gates/kx-gates.toml`, names the file after its locale, and only reads layouts, never adding or removing one. A language with no installed layout makes the command fail and names it, though the other tables are still written. Capture again rather than edit a table by hand.
