//! Translation keys and groups the UI names; a group's key takes an id from elsewhere, as in `theme.native`.

use kx_module_api::ports::{ClipKind, Filter, MAX_PRESETS, PanelNote};

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
/// A chip's name for readers: `$word`, the chip's text, after a word saying it is a suggestion.
pub const CHIP_NAME: &str = "chip.name";
/// A chip's name for readers in forget mode: `$word`, after a word saying a click forgets it.
pub const CHIP_FORGET_NAME: &str = "chip.forget-name";
/// The More chip's text, which opens the email list.
pub const CHIP_MORE: &str = "chip.more";
/// The Undo chip's text, which brings back the word just forgotten.
pub const CHIP_UNDO: &str = "chip.undo";
/// The lane's name for readers while it offers nothing.
pub const CHIP_NONE: &str = "chip.none";
/// The note a password box shows in place of chips.
pub const CHIP_LOCKED: &str = "chip.locked";
/// The suggestions menu's button.
pub const WORDS_MENU: &str = "words.menu";
/// The menu row that makes the next chip tapped forgotten.
pub const WORDS_FORGET: &str = "words.forget";
/// The forget row's short label.
pub const WORDS_FORGET_SHORT: &str = "words.forget-short";
/// The chip that ends forget mode or closes the email list.
pub const WORDS_CANCEL: &str = "words.cancel";
/// The menu row that pauses learning.
pub const WORDS_PAUSE: &str = "words.pause";
/// The pause row's short label.
pub const WORDS_PAUSE_SHORT: &str = "words.pause-short";
/// The menu row that resumes learning.
pub const WORDS_RESUME: &str = "words.resume";
/// The resume row's short label.
pub const WORDS_RESUME_SHORT: &str = "words.resume-short";
/// The menu row that turns suggestions off.
pub const WORDS_OFF: &str = "words.off";
/// The off row's short label.
pub const WORDS_OFF_SHORT: &str = "words.off-short";
/// The menu row that turns suggestions on.
pub const WORDS_ON: &str = "words.on";
/// The on row's short label.
pub const WORDS_ON_SHORT: &str = "words.on-short";
/// The clipboard panel's window title.
pub const WINDOW_CLIP: &str = "window.clip";
/// The mark between a number's whole part and its decimals.
pub const NUM_DECIMAL: &str = "num.decimal";
/// The mark between each three digits of a number's whole part.
pub const NUM_GROUP: &str = "num.group";
/// The clipboard panel's title.
pub const CLIP_TITLE: &str = "clip.title";
/// How long entries are kept: `$days`, plural by `$form`.
pub const CLIP_KEPT: &str = "clip.kept";
/// The search line's hint while it is empty.
pub const CLIP_SEARCH: &str = "clip.search";
/// The panel's Close button.
pub const CLIP_CLOSE: &str = "clip.close";
/// A filter's label: this, then its id in `CLIP_FILTERS`.
pub const CLIP_FILTER: &str = "clip.filter-";
/// Each filter's id, in `Filter::ORDER`.
pub const CLIP_FILTERS: [(Filter, &str); Filter::ORDER.len()] = [
    (Filter::All, "all"),
    (Filter::Text, "text"),
    (Filter::Images, "images"),
    (Filter::Files, "files"),
    (Filter::Media, "media"),
];
/// A kind's name, which a row with no text of its own shows: this, then its id in `CLIP_KINDS`.
pub const CLIP_KIND: &str = "clip.kind-";
/// Each kind's id, which QML also picks the row's icon by.
pub const CLIP_KINDS: [(ClipKind, &str); 5] = [
    (ClipKind::Text, "text"),
    (ClipKind::Link, "link"),
    (ClipKind::Image, "image"),
    (ClipKind::File, "file"),
    (ClipKind::Media, "media"),
];
/// The footer's Preview.
pub const CLIP_PREVIEW: &str = "clip.preview";
/// The footer's Pin.
pub const CLIP_PIN: &str = "clip.pin";
/// The footer's Pin while every target is pinned.
pub const CLIP_UNPIN: &str = "clip.unpin";
/// The footer's Delete.
pub const CLIP_DELETE: &str = "clip.delete";
/// The header's Delete all.
pub const CLIP_DELETE_ALL: &str = "clip.delete-all";
/// The footer's Insert with how many are ticked: `$count`.
pub const CLIP_INSERT: &str = "clip.insert";
/// The preview page's Insert, which acts on the previewed entry alone.
pub const CLIP_INSERT_ONE: &str = "clip.insert-one";
/// The preview page's Back.
pub const CLIP_BACK: &str = "clip.back";
/// Delete all's question: `$count` entries, plural by `$form`.
pub const CLIP_CONFIRM: &str = "clip.confirm";
/// The question's Yes.
pub const CLIP_YES: &str = "clip.yes";
/// The question's No.
pub const CLIP_NO: &str = "clip.no";
/// What an empty history shows.
pub const CLIP_EMPTY: &str = "clip.empty";
/// A row copied today.
pub const CLIP_TODAY: &str = "clip.today";
/// A row copied yesterday.
pub const CLIP_YESTERDAY: &str = "clip.yesterday";
/// A weekday's short name: this, then its ISO number, Monday 1.
pub const CLIP_DAY: &str = "clip.day-";
/// A day `$word` and its `$time`.
pub const CLIP_WHEN: &str = "clip.when";
/// A 24-hour time: `$hour` and `$minute`, two digits each.
pub const CLIP_TIME: &str = "clip.time";
/// A date: `$day`, `$month` and `$year`.
pub const CLIP_DATE: &str = "clip.date";
/// A size in MB: `$mb`.
pub const CLIP_SIZE: &str = "clip.size";
/// A size in GB: `$gb`.
pub const CLIP_SIZE_GB: &str = "clip.size-gb";
/// A size below `$mb`, the smallest two decimals show.
pub const CLIP_SIZE_SMALL: &str = "clip.size-small";
/// The storage line: `$used` of `$cap`, and `$free` on `$drive`.
pub const CLIP_STORAGE: &str = "clip.storage";
/// The storage line's Open folder.
pub const CLIP_OPEN_FOLDER: &str = "clip.open-folder";
/// A row's meta line: `$app`, `$when` and `$size`.
pub const CLIP_META: &str = "clip.meta";
/// A row's meta line with no app: `$when` and `$size`.
pub const CLIP_META_BARE: &str = "clip.meta-bare";
/// The preview's length: `$count` characters, plural by `$form`.
pub const CLIP_CHARS: &str = "clip.chars";
/// The chip on media that goes tonight.
pub const CLIP_EXPIRES: &str = "clip.expires";
/// A note: this, then its id in `CLIP_NOTES`.
pub const CLIP_NOTE: &str = "clip.note-";
/// Each note's id.
pub const CLIP_NOTES: [(PanelNote, &str); 5] = [
    (PanelNote::TooBig, "too-big"),
    (PanelNote::Disk, "disk"),
    (PanelNote::PinnedFull, "pinned-full"),
    (PanelNote::Busy, "busy"),
    (PanelNote::PasteFailed, "paste-failed"),
];
/// What readers hear for a ticked row: its place in the tick order, `$count`.
pub const CLIP_TICKED: &str = "clip.ticked";
/// What readers hear for a pinned row.
pub const CLIP_PINNED: &str = "clip.pinned";
/// What joins the parts of a row's name for readers.
pub const CLIP_JOIN: &str = "clip.join";
