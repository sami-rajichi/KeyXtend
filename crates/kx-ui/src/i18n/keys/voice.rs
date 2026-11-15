//! Voice typing's translation keys (P11 design "What the user sees"): every key starts `voice.`, and a group's key
//! takes an id from `ids`, as in `voice.note-no-mic`.

mod ids;
pub mod vars;

pub use ids::*;

/// A word by id, and the start of every voice key: this, then a button's or a page's id, as in `voice.stop`.
pub const VOICE_WORD: &str = "voice.";
/// A note's line: this, then its `bar_hint_id`.
pub const VOICE_NOTE: &str = "voice.note-";
/// A mark's word: this, then its `mark_id`.
pub const VOICE_MARK: &str = "voice.mark-";
/// An option's label: this, then its setting key with hyphens for underscores.
pub const VOICE_OPTION: &str = "voice.opt-";
/// A bar place's name: this, then its `place_id`.
pub const VOICE_PLACE: &str = "voice.place-";
/// A language's name by ISO 639 code, as History names a text's language.
pub const LANG_NAME: &str = "langname";
/// The tray menu's Dictation row, which opens the Voice panel.
pub const TRAY_VOICE: &str = "tray.voice";
/// The bar's window title.
pub const WINDOW_VOICE: &str = "window.voice";
/// The Voice panel's window title.
pub const WINDOW_VOICE_PANEL: &str = "window.voice-panel";

/// The bar's and the panel's name, Dictation.
pub const VOICE_TITLE: &str = "voice.title";
/// The gear's name.
pub const VOICE_GEAR: &str = "voice.gear";
/// The Listening state's word.
pub const VOICE_LISTENING: &str = "voice.listening";
/// The Transcribing state's word.
pub const VOICE_TRANSCRIBING: &str = "voice.transcribing";
/// The hint while listening hears nothing yet.
pub const VOICE_NOTHING: &str = "voice.nothing";
/// Copy's word once it copied.
pub const VOICE_COPIED: &str = "voice.copied";
/// The dots a password box's text shows as.
pub const VOICE_MASK: &str = "voice.mask";
/// Download with the model's `$size`.
pub const VOICE_DOWNLOAD_SIZE: &str = "voice.download-size";
/// The home list's header: `$lang`.
pub const VOICE_LANG: &str = "voice.lang";
/// The home list's model row with no model ready.
pub const VOICE_NO_MODEL: &str = "voice.no-model";
/// Automatic on the processor.
pub const VOICE_AUTO_CPU: &str = "voice.auto-cpu";
/// Automatic on card `$device`.
pub const VOICE_AUTO_ON: &str = "voice.auto-on";
/// Compare's last run: `$date`.
pub const VOICE_LAST_RUN: &str = "voice.last-run";
/// Compare before its first run.
pub const VOICE_NEVER: &str = "voice.never";
/// `$count` texts, plural by `$form`.
pub const VOICE_TEXTS: &str = "voice.texts";
/// Every option at its default.
pub const VOICE_DEFAULTS: &str = "voice.defaults";
/// `$count` options changed.
pub const VOICE_CHANGED: &str = "voice.changed";
/// Try again's hint on the home list.
pub const VOICE_AGAIN_HINT: &str = "voice.again-hint";
/// A header for language `$lang`.
pub const VOICE_FOR_LANG: &str = "voice.for-lang";
/// The line before the best model's name, for `$lang`.
pub const VOICE_BEST: &str = "voice.best";
/// The general order serves `$lang`, untested.
pub const VOICE_UNTESTED: &str = "voice.untested";
/// The default for `$lang` is not for this PC, so local models will be very slow here.
pub const VOICE_SLOW: &str = "voice.slow";
/// A model that does not fit: `$need` and `$free`.
pub const VOICE_UNFIT: &str = "voice.unfit";
/// A download's progress: `$done` of `$total`.
pub const VOICE_GOT: &str = "voice.got";
/// A paused download's progress: `$done` of `$total`.
pub const VOICE_PAUSED: &str = "voice.paused";
/// The Models page's Delete.
pub const VOICE_DELETE_FILE: &str = "voice.delete-file";
/// The folder line: `$path` and `$free`.
pub const VOICE_FOLDER: &str = "voice.folder";
/// The first download's question: `$path` and `$free`.
pub const VOICE_FOLDER_OFFER: &str = "voice.folder-offer";
/// The folder list's title.
pub const VOICE_FOLDERS: &str = "voice.folders";
/// The folder list at the drives.
pub const VOICE_DRIVES: &str = "voice.drives";
/// The Device page's header line.
pub const VOICE_DEVICE_HINT: &str = "voice.device-hint";
/// Automatic's name.
pub const VOICE_AUTO: &str = "voice.auto";
/// Automatic's line.
pub const VOICE_AUTO_LINE: &str = "voice.auto-line";
/// The processor's name.
pub const VOICE_CPU: &str = "voice.cpu";
/// The processor's line: `$cores`, `$total` and `$free`.
pub const VOICE_CPU_LINE: &str = "voice.cpu-line";
/// `$count` cores, plural by `$form`.
pub const VOICE_CORES: &str = "voice.cores";
/// A card's line: `$total` and `$free`.
pub const VOICE_CARD_LINE: &str = "voice.card-line";
/// Built-in graphics' line: `$total` and `$free`.
pub const VOICE_BUILT_IN_LINE: &str = "voice.built-in-line";
/// A card's check while it runs.
pub const VOICE_CHECKING: &str = "voice.checking";
/// `$times` faster, plural by `$form`.
pub const VOICE_FASTER: &str = "voice.faster";
/// A slow card: `$faster`, and the `$need` it missed.
pub const VOICE_WHY_SLOW: &str = "voice.why-slow";
/// A card whose text differed: `$words`, and the `$allowed`.
pub const VOICE_WHY_DIFFERS: &str = "voice.why-differs";
/// A card with too little free memory.
pub const VOICE_WHY_ROOM: &str = "voice.why-room";
/// A card that reported an error.
pub const VOICE_WHY_ERROR: &str = "voice.why-error";
/// A failed check's band: the card's `$name` and `$why`.
pub const VOICE_FAILED: &str = "voice.failed";
/// The Device page's Use.
pub const VOICE_USE_DEVICE: &str = "voice.use-device";
/// Compare's instruction with a set text.
pub const VOICE_INTRO: &str = "voice.intro";
/// Compare's instruction for a language with no set text.
pub const VOICE_INTRO_FREE: &str = "voice.intro-free";
/// Compare's set text, which the user reads aloud.
pub const VOICE_READ_TEXT: &str = "voice.read-text";
/// Compare's models: `$count` and `$device`, plural by `$form`.
pub const VOICE_READY: &str = "voice.ready";
/// Compare's Start.
pub const VOICE_START: &str = "voice.start";
/// Compare while the user reads.
pub const VOICE_RECORDING: &str = "voice.recording";
/// Compare's step: `$at` of `$of`.
pub const VOICE_STEP: &str = "voice.step";
/// The results' columns: this, then `model`, `wrong`, `wait` or `memory`.
pub const VOICE_COL: &str = "voice.col-";
/// The results' columns, in order.
pub const VOICE_COLS: [&str; 4] = ["model", "wrong", "wait", "memory"];
/// Words wrong: `$wrong` of `$words`.
pub const VOICE_WRONG_OF: &str = "voice.wrong-of";
/// What Compare keeps.
pub const VOICE_KEPT: &str = "voice.kept";
/// The results' header: `$date` and `$device`.
pub const VOICE_RESULT_LINE: &str = "voice.result-line";
/// History's header: `$texts`.
pub const VOICE_HISTORY_SUB: &str = "voice.history-sub";
/// A History row's line: `$when`, `$lang` and `$model`.
pub const VOICE_META: &str = "voice.meta";
/// History's Pin while every ticked text is pinned.
pub const VOICE_UNPIN: &str = "voice.unpin";
/// History's Insert with the ticked `$count`.
pub const VOICE_INSERT: &str = "voice.insert";
/// History's Insert with one text ticked.
pub const VOICE_INSERT_ONE: &str = "voice.insert-one";
/// Delete all's question: `$count`, plural by `$form`.
pub const VOICE_CONFIRM: &str = "voice.confirm";
/// The question's Yes.
pub const VOICE_YES: &str = "voice.yes";
/// The question's No.
pub const VOICE_NO: &str = "voice.no";
/// An empty History.
pub const VOICE_EMPTY: &str = "voice.empty";
/// What readers hear for a ticked row: `$count`.
pub const VOICE_TICKED: &str = "voice.ticked";
/// What readers hear for a pinned row.
pub const VOICE_PINNED: &str = "voice.pinned";
/// What joins a row's name for readers.
pub const VOICE_JOIN: &str = "voice.join";
/// Options' header line.
pub const VOICE_FOR_ALL: &str = "voice.for-all";
/// Options' footer.
pub const VOICE_APPLIES: &str = "voice.applies";
/// A switch on.
pub const VOICE_ON: &str = "voice.on";
/// A switch off.
pub const VOICE_OFF: &str = "voice.off";
/// A stepper's minus for readers: `$name`.
pub const VOICE_MINUS: &str = "voice.minus";
/// A stepper's plus for readers: `$name`.
pub const VOICE_PLUS: &str = "voice.plus";
/// Whole seconds: `$count`.
pub const VOICE_SECONDS: &str = "voice.seconds";
/// Minutes: `$count`.
pub const VOICE_MINUTES: &str = "voice.minutes";
/// Milliseconds: `$count`.
pub const VOICE_MS: &str = "voice.ms";
/// Words: `$count`.
pub const VOICE_WORDS: &str = "voice.words";
/// A factor: `$times`, plural by `$form`.
pub const VOICE_TIMES: &str = "voice.times";
/// Seconds with decimals: `$s`.
pub const VOICE_SECS: &str = "voice.secs";
/// Threads at 0: Auto with the real `$count`.
pub const VOICE_AUTO_N: &str = "voice.auto-n";
/// A size in MB: `$mb`.
pub const VOICE_MB: &str = "voice.mb";
/// A size in GB: `$gb`.
pub const VOICE_GB: &str = "voice.gb";
