//! Voice typing's cloud texts and ids (ADR-0047), in their own `cloud.` group and files: the Cloud page, a card's labels
//! and fact groups, the key page's lines, each cloud button, key test, address fault and cloud reason, and Models' sides.

use super::VoiceMark;
use kx_module_api::ports::{
    AddressFault, CloudBand, CloudButton, CloudLine, CloudWhy, KeyCheck, ModelSide, Renewal,
    ServiceMark, Unavailable,
};

/// The start of every cloud key.
pub const CLOUD_WORD: &str = "cloud.";
/// The Cloud page's line under its title.
pub const VOICE_CLOUD_SUB: &str = "cloud.sub";
/// The `cloud` switch's label.
pub const VOICE_CLOUD_SWITCH: &str = "cloud.switch";
/// The line while cloud services are off, which the Models page's Cloud side shows too.
pub const VOICE_CLOUD_OFF: &str = "cloud.off-line";
/// A service row's free allowance, short: `$free`.
pub const VOICE_CLOUD_FREE: &str = "cloud.free";
/// The Custom row's name.
pub const VOICE_CUSTOM: &str = "cloud.custom";
/// The Custom row's line.
pub const VOICE_CUSTOM_LINE: &str = "cloud.custom-line";
/// The home list's Cloud value while on: `$ready`.
pub const VOICE_CLOUD_HOME: &str = "cloud.home";
/// How many services are ready: `$count`, plural by `$form`.
pub const VOICE_CLOUD_READY: &str = "cloud.ready";
/// A card's line under its title while it asks.
pub const VOICE_CARD_SUB: &str = "cloud.card-sub";
/// A card's line under its title once on: `$date`.
pub const VOICE_CARD_ON: &str = "cloud.card-on";
/// A card's label: this, then a `CARD_*` label.
pub const VOICE_CARD: &str = "cloud.card-";
/// The card's labels, in order; light and heavy take `$hours`.
pub const CARD_LABELS: [&str; 6] = ["where", "trains", "keeps", "free", "light", "heavy"];
/// The card's label for the least a request bills.
pub const CARD_BILLS: &str = "bills";
/// The card's label for a rule the service sets on its use.
pub const CARD_RULE: &str = "rule";
/// A use's hours a month: `$count`, plural by `$form`.
pub const VOICE_CARD_HOURS: &str = "cloud.card-hours";
/// The day the facts were read: `$date`.
pub const VOICE_CARD_READ_ON: &str = "cloud.card-read-on";
/// The words after the day once the facts are old.
pub const VOICE_CARD_OLD: &str = "cloud.card-old";
/// The card's link: `$site`.
pub const VOICE_CARD_LINK: &str = "cloud.card-link";
/// The card's Confirm.
pub const VOICE_CARD_CONFIRM: &str = "cloud.card-confirm";
/// The line before Confirm: `$name`.
pub const VOICE_CARD_ASK: &str = "cloud.card-ask";
/// The Custom row's name inside the confirm line.
pub const VOICE_CARD_CUSTOM: &str = "cloud.card-custom";
/// The card's line for a connection no real key checked yet.
pub const VOICE_CARD_UNTESTED: &str = "cloud.card-untested";
/// A fact's text: this, then a `FACT_*` group, `-`, then the fact's id.
pub const VOICE_FACT: &str = "cloud.fact-";
/// A warning fact's short mark on its service's row: this, then the fact's id.
pub const VOICE_WARN_MARK: &str = "cloud.warn-mark-";
/// The fact group of where the voice goes.
pub const FACT_WHERE: &str = "where";
/// The fact group of training.
pub const FACT_TRAINS: &str = "trains";
/// The fact group of what is kept.
pub const FACT_KEEPS: &str = "keeps";
/// The fact group of the free allowance.
pub const FACT_FREE: &str = "free";
/// The fact group of a use's price.
pub const FACT_PRICE: &str = "price";
/// The fact group of the least a request bills.
pub const FACT_BILLS: &str = "bills";
/// The fact group of a rule the service sets on its use.
pub const FACT_RULE: &str = "rule";
/// The fact group of a card's warning.
pub const FACT_WARN: &str = "warn";
/// Money in a currency: this, then its ISO 4217 code in lower case; `$amount`.
pub const VOICE_MONEY: &str = "cloud.money-";
/// Money in a currency with no text of its own: `$amount` and `$code`.
pub const VOICE_MONEY_ANY: &str = "cloud.money";
/// The key line's caption.
pub const VOICE_KEY_LINE: &str = "cloud.key-line";
/// The key line's hint.
pub const VOICE_KEY_HINT: &str = "cloud.key-hint";
/// The line that the key is sealed.
pub const VOICE_KEY_SEALED: &str = "cloud.key-sealed";
/// The key page's link: `$site`.
pub const VOICE_KEY_LINK: &str = "cloud.key-link";
/// A key test's line: this, then its `key_check_id`.
pub const VOICE_KEY: &str = "cloud.key-";
/// The address line's caption.
pub const VOICE_ADDRESS_LINE: &str = "cloud.address-line";
/// The address line's hint for Custom: `$https`.
pub const VOICE_ADDRESS_HINT: &str = "cloud.address-hint";
/// The address line's hint for a service's own resource.
pub const VOICE_ADDRESS_HINT_OWN: &str = "cloud.address-hint-own";
/// An address fault's line: this, then its `address_fault_id`; `$https` too.
pub const VOICE_ADDRESS: &str = "cloud.address-";
/// The Other model line's caption.
pub const VOICE_MODEL_LINE: &str = "cloud.model-line";
/// The Other model line's hint.
pub const VOICE_MODEL_HINT: &str = "cloud.model-hint";
/// The Other model line's refusal.
pub const VOICE_MODEL_REFUSED: &str = "cloud.model-refused";
/// A typed line's Save (ADR-0049).
pub const VOICE_LINE_SAVE: &str = "cloud.line-save";
/// A line's Save as readers hear it: this, then its `cloud_line_id`.
pub const VOICE_LINE_SAVE_NAME: &str = "cloud.line-save-";
/// The keep-nothing switch's label.
pub const VOICE_KEEP_NOTHING: &str = "cloud.keep-nothing";
/// The bar's cloud mark for readers: `$name`.
pub const VOICE_ON_BAR_CLOUD: &str = "cloud.on-bar";
/// "Use the local model", on a cloud note.
pub const VOICE_USE_LOCAL: &str = "cloud.use-local";
/// Compare's warning before Start: `$names`.
pub const VOICE_SEND_WARNING: &str = "cloud.send-warning";
/// A cloud note's line: this, then its `cloud_why_id`.
pub const VOICE_NOTE_CLOUD: &str = "cloud.note-";
/// The used-up allowance's line with the clock time it renews at: `$at`.
pub const VOICE_NOTE_RENEWS: &str = "cloud.note-allowance-at";
/// The used-up allowance's line when it renews on a later day: `$date` and `$at`.
pub const VOICE_NOTE_RENEWS_ON: &str = "cloud.note-allowance-on";
/// A Models side's chip: this, then its `model_side_id`.
pub const VOICE_MODELS_SIDE: &str = "cloud.side-";
/// The side chips' group name.
pub const VOICE_MODELS_SIDES: &str = "cloud.sides";
/// The Cloud side's band: this, then its `cloud_band_id`; Off shows `VOICE_CLOUD_OFF`.
pub const VOICE_BAND: &str = "cloud.band-";
/// A cloud model's line on the Models page.
pub const VOICE_CLOUD_MODEL: &str = "cloud.model";

/// The key of a fact of `group` with text `id`.
#[must_use]
pub fn fact_key(group: &str, id: &str) -> String {
    format!("{VOICE_FACT}{group}-{id}")
}

/// The key of the short mark a service's row shows for its warning fact `id`.
#[must_use]
pub fn warn_mark_key(id: &str) -> String {
    format!("{VOICE_WARN_MARK}{id}")
}

/// The mark a service's row shows for `mark`.
#[must_use]
pub const fn service_mark(mark: ServiceMark) -> VoiceMark {
    match mark {
        ServiceMark::Off => VoiceMark::Off,
        ServiceMark::NeedsKey => VoiceMark::NeedsKey,
        ServiceMark::NeedsAddress => VoiceMark::NeedsAddress,
        ServiceMark::Ready => VoiceMark::On,
    }
}

/// A cloud button's id, which QML sends back on a click.
#[must_use]
pub const fn cloud_button_id(button: CloudButton) -> &'static str {
    match button {
        CloudButton::On => "on",
        CloudButton::Off => "off",
        CloudButton::Facts => "facts",
        CloudButton::Lines => "lines",
        CloudButton::Paste => "paste",
        CloudButton::Test => "test",
        CloudButton::Remove => "remove",
        CloudButton::Link => "link",
    }
}

/// A key page line's id, which QML sends back with a click on it or its Save.
#[must_use]
pub const fn cloud_line_id(line: CloudLine) -> &'static str {
    match line {
        CloudLine::Key => "key",
        CloudLine::Address => "address",
        CloudLine::Model => "model",
    }
}

/// A cloud button's word; Link's is the card's, as the key page words its own with `VOICE_KEY_LINK`.
#[must_use]
pub const fn cloud_button_key(button: CloudButton) -> &'static str {
    match button {
        CloudButton::On => "cloud.on",
        CloudButton::Off => "cloud.off",
        CloudButton::Facts => "cloud.facts",
        CloudButton::Lines => "cloud.lines",
        CloudButton::Paste => "cloud.key-paste",
        CloudButton::Test => "cloud.key-test",
        CloudButton::Remove => "cloud.key-remove",
        CloudButton::Link => VOICE_CARD_LINK,
    }
}

/// A key test's id, whose line is `cloud.key-` then it unless `key_check_why` names a reason; `None` while untested,
/// which shows no line.
#[must_use]
pub const fn key_check_id(check: KeyCheck) -> Option<&'static str> {
    Some(match check {
        KeyCheck::Untested => return None,
        KeyCheck::Testing => "testing",
        KeyCheck::Works => "ok",
        KeyCheck::Refused => "bad",
        KeyCheck::Failed => "failed",
        KeyCheck::Credit => "credit",
        KeyCheck::Account => "account",
        KeyCheck::Region => "region",
        KeyCheck::Allowance => "allowance",
        KeyCheck::Rate => "rate",
        KeyCheck::Strange => "strange",
        KeyCheck::Unfit => "unfit",
        KeyCheck::NoTest => "no-test",
        KeyCheck::Busy => "busy",
    })
}

/// The cloud reason whose note a key test's answer reads as, a used allowance with `renews_at`, so each reason is
/// written once; `None` for the rest.
#[must_use]
pub const fn key_check_why(check: KeyCheck, renews_at: Option<Renewal>) -> Option<CloudWhy> {
    Some(match check {
        KeyCheck::Credit => CloudWhy::Credit,
        KeyCheck::Account => CloudWhy::Account,
        KeyCheck::Region => CloudWhy::Region,
        KeyCheck::Allowance => CloudWhy::Allowance { renews_at },
        KeyCheck::Rate => CloudWhy::Rate,
        KeyCheck::Strange => CloudWhy::Strange,
        _ => return None,
    })
}

/// An address fault's id.
#[must_use]
pub const fn address_fault_id(fault: AddressFault) -> &'static str {
    match fault {
        AddressFault::Https => "https",
        AddressFault::Host => "host",
        AddressFault::Port => "port",
        AddressFault::Domain => "domain",
        AddressFault::Long => "long",
        AddressFault::Path => "path",
    }
}

/// A cloud reason's id, whose line is `cloud.note-` then it.
#[must_use]
pub const fn cloud_why_id(why: &CloudWhy) -> &'static str {
    match why {
        CloudWhy::Net => "net",
        CloudWhy::Key => "key",
        CloudWhy::Credit => "credit",
        CloudWhy::Account => "account",
        CloudWhy::Region => "region",
        CloudWhy::Allowance { .. } => "allowance",
        CloudWhy::Rate => "rate",
        CloudWhy::TooBig => "too-big",
        CloudWhy::Language(_) => "language",
        CloudWhy::Empty => "empty",
        CloudWhy::Strange => "strange",
        CloudWhy::Terms => "terms",
        CloudWhy::Off => "off",
        CloudWhy::Moved => "moved",
        CloudWhy::NoLocal => "no-local",
        CloudWhy::Fault => "fault",
        CloudWhy::Unavailable(Unavailable::KeepNothing) => "keep-nothing",
    }
}

/// A Models side's id, which QML sends back on a click.
#[must_use]
pub const fn model_side_id(side: ModelSide) -> &'static str {
    match side {
        ModelSide::Local => "local",
        ModelSide::Cloud => "cloud",
    }
}

/// A Cloud side band's id.
#[must_use]
pub const fn cloud_band_id(band: CloudBand) -> &'static str {
    match band {
        CloudBand::Off => "off",
        CloudBand::NoneOn => "none-on",
        CloudBand::Listed => "listed",
    }
}
