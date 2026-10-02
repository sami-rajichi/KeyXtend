# 17. Translations: Fluent files compiled in

- **Date:** 2026-09-30
- **Status:** Accepted. Its language choice for the keyboard's text is superseded by ADR-0020.

## Context

Every text the user sees lives in translation files (owner's rule 1). Version 1 speaks English, French and Arabic, and the Arabic must be correct.

- Texts take arguments (a module name, a list of settings, a count), plural forms (Arabic has six) and Latin words inside right-to-left text.
- The uiAccess process reads nothing it shows from disk (ADR-0016).
- Code names each text by a key such as `key.esc` or `notice.settings-reset` (`kx-module-api` notices, the `kx-mod-keyboard` view).

## Decision

Decided with the owner on 2026-09-29 and 2026-09-30.

- **Fluent** ([Project Fluent](https://projectfluent.org/), `fluent-bundle` 0.16, licences in `THIRD_PARTY.md`): one `.ftl` file per language in `crates/kx-ui/i18n/`, compiled in with `include_str!`. The languages are one table, `kx_ui::LANGS`, English first.
- **The dot rule:** keys in code stay `group.name`. Fluent ids cannot hold dots, so `group.name` is message `group`, attribute `name`; a key without a dot is a plain message.
- **Fallback:** a key missing in French or Arabic comes from English. A key missing everywhere shows the key itself, and a module or theme name shows its id. A broken line leaves the rest loaded, and nothing panics.
- **Language choice:** `[ui] language = "auto"` follows the Windows display language (`SystemLook::ui_language`), matched to EN, FR or AR, else English. A supported value (`en`, `fr`, `ar`) wins; any other value follows Windows too, as `pick` does.
- **Arguments:** notices carry typed values (`NoticeArg`: text, count or list), never typed text, clipboard data or secrets. A module id shows by its name (`module.<id>`). Each language joins a list its own way (Arabic with "، ") and ends it with "and N more"; a list that names none shows only the count. Counts reach Fluent as numbers.
- **Plurals:** Fluent's CLDR plural forms. `intl_pluralrules` 7.0.2 drops CLDR's `n % 100` for Arabic, so 103 would take the wrong form; plural selects therefore use `$form`: past 100, the count's last two digits when they are 3 or more, else the count itself. The text still shows the real count.
- **Digits:** Western digits 0–9 in all three languages, Fluent's default formatting.
- **Bidi:** Fluent's isolation marks stay on, so each argument, such as a Latin setting name in Arabic text, keeps its order. Control characters, line breaks and bidi controls inside argument values, from a quoted TOML key for example, are dropped first (`kx_module_api::is_hidden`, the table the log escapes too).
- **Words on keys follow real keyboards.** French prints French words (Échap, Suppr, Verr. Maj, Maj, Entrée, Alt Gr). Arabic keeps the English words Arabic keyboards print (Esc, Del, Ctrl). Tips and screen-reader names use Microsoft's Windows terms where they exist; an Arabic key that prints a Latin abbreviation is "مفتاح" and that word, such as مفتاح Esc (owner, 2026-09-30).
- **Theme names** keep model names and translate plain words: Native Adaptive, ET66, Modern Dolch; Natif adaptatif, ET66, Dolch moderne; الأصلي المتكيّف، ET66، دولش الحديث.
- **One place per name:** theme names moved from `themes.toml` to `theme.<id>`, and the space bar's joiner from `spike.toml` to `lang.space`.

## Consequences

- A new language is one file and one table row; tests check that all files hold the same keys. It also needs a check of `form()` against its CLDR plural rules, as the Arabic and French tests do.
- `kx-ui` cannot read `board.toml` or the module list, so the app checks that every board id, icon id and module id has its text in each language.
- Translators select plural forms on `$form`, not `$more`; this keeps working if `intl_pluralrules` fixes the modulo.
- Changing a text needs a rebuild, the price of reading nothing from disk.

## Alternatives

- **Qt Linguist (`.ts`/`.qm`):** native to QML, and its plurals cover Arabic's six forms, but the Rust side needs the same texts; refused.
- **gettext (`.po`):** widely known, but weaker arguments and one plural formula per file; refused.
- **Plain JSON tables:** simple, but no plurals, arguments or bidi isolation; refused.
- **Files read from disk:** users could add languages, but the uiAccess process must not read what it shows; refused.
- **ICU4X for plural rules:** correct Arabic, but a large new dependency for a two-digit fix; refused.
