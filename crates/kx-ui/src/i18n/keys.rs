//! Translation keys and groups the UI names; a group's key takes an id from elsewhere, as in `theme.native`.

/// Module names, by module id.
pub const MODULE: &str = "module";
/// Theme names, by `themes.toml` id.
pub const THEME: &str = "theme";
/// Top-bar button names, by `icons.toml` `[bar]` id.
pub const BAR: &str = "bar";
/// The language key's design names, by `[keyboard] lang_key` value.
pub const LANG_KEY: &str = "langkey";
/// The tray menu's "Show keyboard".
pub const TRAY_SHOW: &str = "tray.show";
/// The tray menu's "Quit".
pub const TRAY_QUIT: &str = "tray.quit";
/// A notice's "Try again" button.
pub const RETRY: &str = "button.retry";
/// A notice's "OK" button.
pub const OK: &str = "button.ok";
/// What joins two list items.
pub const LIST_JOIN: &str = "list.join";
/// A list that leaves `$more` items out, after the named ones in `$list`.
pub const LIST_MORE: &str = "list.more";
/// A list that names nothing: only how many, `$more`.
pub const LIST_COUNT: &str = "list.count";
/// The space bar: `$lang`, then `$layout`.
pub const SPACE: &str = "lang.space";
