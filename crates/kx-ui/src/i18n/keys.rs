//! Translation keys and groups the UI names; a group's key takes an id from elsewhere, as in `theme.native`.

use kx_module_api::ports::MAX_PRESETS;

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
/// The size buttons' letters, by an id in `SIZES`.
pub const SIZE: &str = "size";
/// The size buttons' names for tips and readers, by an id in `SIZES`.
pub const SIZE_NAME: &str = "sizename";
/// The size buttons' ids, in `[keyboard] presets` order: one per preset the face offers, so the counts cannot part.
pub const SIZES: [&str; MAX_PRESETS] = ["s", "m", "l"];
/// The look's modes, by `[ui] mode` value.
pub const MODE: &str = "mode";
/// The D-pad's arrows and Stop, by `icons.toml` `[dpad]` id.
pub const DPAD: &str = "dpad";
/// The keyboard window's title.
pub const WINDOW_KEYBOARD: &str = "window.keyboard";
/// The minimise bubble's title.
pub const WINDOW_BUBBLE: &str = "window.bubble";
/// The notice banner's title.
pub const WINDOW_NOTICE: &str = "window.notice";
/// The tray menu's title.
pub const WINDOW_MENU: &str = "window.menu";
/// The language list's title.
pub const WINDOW_LIST: &str = "window.list";
/// A tip's title, by which it lifts itself in front of our windows as it shows.
pub const WINDOW_TIP: &str = "window.tip";
/// The pointer helper's title: the hold ring and the mode badge.
pub const WINDOW_POINTER: &str = "window.pointer";
/// The title of the scroll pad's outline windows.
pub const WINDOW_OUTLINE: &str = "window.outline";
/// The badge's word while the pad scrolls slowly.
pub const BADGE_SLOW: &str = "badge.slow";
/// The badge's word while the pad scrolls fast.
pub const BADGE_FAST: &str = "badge.fast";
/// The mode badge's word while Grab holds a press down.
pub const BADGE_HOLDING: &str = "badge.holding";
/// What joins the badge's parts that readers hear after its name (`Badge.qml`).
pub const BADGE_JOIN: &str = "badge.join";
/// How many more notices wait behind the one the banner shows: `$more`.
pub const BANNER_MORE: &str = "banner.more";
/// A pager's number: page `$at` of `$count`.
pub const PAGE_OF: &str = "page.of";
/// A pager's button that turns back a page.
pub const PAGE_BACK: &str = "page.back";
/// A pager's button that turns on a page.
pub const PAGE_ON: &str = "page.on";
/// What readers call the cover that ends a move or resize.
pub const CATCH: &str = "catch";
/// What links the names of Arabic marks typed together, which readers hear for a mark key (`Key.qml`).
pub const MARK_JOIN: &str = "mark.join";
/// The words keys print, by board id; a shortcut's tip names a key by its label first.
pub const LABEL: &str = "label";
/// Key names a shortcut's tip shows when no label names them, by key name, such as `keyname.numplus`.
pub const KEYNAME: &str = "keyname";
/// What joins the keys of one step in a shortcut's tip: `+`.
pub const COMBO_JOIN: &str = "combo.join";
/// What joins the steps of a shortcut's tip.
pub const COMBO_THEN: &str = "combo.then";
/// A shortcut's tip: its `$name`, then the `$keys` it sends.
pub const KEYS_TIP: &str = "tip.keys";
/// What readers hear after the Next key's name: the shortcuts layer's page `$page` of `$pages`.
pub const LAYER_PAGE: &str = "key.layer-page";
