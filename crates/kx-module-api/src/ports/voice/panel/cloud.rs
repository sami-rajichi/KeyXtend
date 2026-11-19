//! The Cloud page, a service's card and its key page (ADR-0047): the services with their marks, a card's facts as the
//! services table's ids, numbers and dates, and the typed lines. A card's sentences are the ui's translations filled
//! with these values, so no price or claim is written twice, and typed text stays redacted.

use crate::Redacted;
use crate::ports::{CloudButton, Date, Hundredths};

listed! {
    /// Where a service stands, as its row's mark says; a cloud model's Unavailable is its Models row's.
    pub enum ServiceMark ("Every mark.") {
        /// Not switched on.
        Off,
        /// Switched on, with no key yet.
        NeedsKey,
        /// Switched on with a key, on a typed row with no address yet, so the user is never told to add a key.
        NeedsAddress,
        /// A key is set, tested or not, and the row has somewhere to send.
        Ready,
    }
}

listed! {
    /// Why a typed address was refused, which its line's warning names (edge 13).
    pub enum AddressFault ("Every fault.") {
        /// It does not start with `https://`.
        Https,
        /// Its host is no host name, such as a numeric address.
        Host,
        /// Its host is not under the domain the service's row names.
        Domain,
        /// It is longer than the frames take (`HttpsUrl::MAX_BYTES`).
        Long,
        /// Its path or query holds a character an address may not, such as a space, `@`, `#` or a non-ASCII letter.
        Path,
    }
}

listed! {
    /// How the key's test stands on the key page.
    pub enum KeyCheck ("Every test state.") {
        /// Not tested since the key was set.
        Untested,
        /// The test runs.
        Testing,
        /// The service took the key.
        Works,
        /// The service refused the key.
        Refused,
        /// The test got no answer about the key, as the network failed; Test may be tried again.
        Failed,
        /// The text set or pasted is no key a request can carry, so nothing was kept.
        Unfit,
        /// The row has no test call, so the first dictation tests the key.
        NoTest,
        /// The clipboard was busy or gave no answer to Paste, which may be tried again.
        Busy,
    }
}

listed! {
    /// The part of a card a page shows, as a card takes two pages of the panel (ADR-0008).
    pub enum CardPart ("Every part, in page order.") {
        /// Where the voice goes, whether the service trains on it, and what it keeps.
        Voice,
        /// The free allowance, the prices of light and heavy use, and the least a request bills.
        Price,
    }
}

listed! {
    /// A typed line of a service's key page (ADR-0049).
    pub enum CloudLine ("Every line, in page order.") {
        /// The key, shown only as dots.
        Key,
        /// The address, for a typed row such as Custom.
        Address,
        /// The Other model id.
        Model,
    }
}

listed! {
    /// The part of a key page a page shows, when a typed address makes it two pages of the panel (ADR-0008).
    pub enum LinesPart ("Every part, in page order.") {
        /// The key line, its test's answer and where keys are made.
        Key,
        /// The address and Other model lines and the keep-nothing switch.
        Model,
    }
}

/// A cloud service as a line names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceName {
    /// A service by its own name from its row, such as Groq.
    Named(String),
    /// The Custom row, which the ui names in the user's language.
    Custom,
}

/// A value a fact's text prints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactValue {
    /// A whole number whose unit and plural the text gives, such as 8 hours or 20 requests.
    Count(u32),
    /// A number kept in hundredths, such as 550 for a 5.5% fee.
    Number(Hundredths),
    /// Money, in hundredths of its currency, such as 60 for $0.60.
    Money {
        /// The amount.
        amount: Hundredths,
        /// Its ISO 4217 code, such as `USD`.
        currency: String,
    },
    /// A day, such as the end of an offer.
    Day(Date),
    /// A name from the service's own pages, printed as it is, such as its account switch's label.
    Text(String),
}

/// A fact's value under the name its text gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactArg {
    /// The name, such as `hours`.
    pub name: String,
    /// The value.
    pub value: FactValue,
}

/// One fact, as the services table holds it: the id of its text and the values that fill it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fact {
    /// Its text's id among its line's, such as `us` for where the voice goes or `day` for the free allowance.
    pub id: String,
    /// The values its text prints.
    pub args: Vec<FactArg>,
    /// It takes the warning tone, as a service that trains on the voice by default.
    pub warn: bool,
}

/// The price of a use of so many hours a month, light or heavy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsePrice {
    /// Hours of sound a month.
    pub hours: u16,
    /// What they cost.
    pub price: Fact,
}

/// A service's row on the Cloud page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceRow {
    /// The service.
    pub name: ServiceName,
    /// Where it stands.
    pub mark: ServiceMark,
    /// No real key has checked this connection yet, so the Untested tag shows beside its name.
    pub untested: bool,
    /// Its free allowance, which the row's second line names; `None` for the Custom row, whose line says what to type.
    pub free: Option<Fact>,
    /// Its card's warning fact id, whose own short mark the row shows; `None` without a warning.
    pub warn: Option<String>,
    /// It is ticked.
    pub tick: bool,
}

/// The Cloud page: the `cloud` switch, then the services, Groq first, tick then act.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudSheet {
    /// Cloud services are on; while off, none is offered and nothing is sent.
    pub on: bool,
    /// The shown page's services.
    pub rows: Vec<ServiceRow>,
    /// The footer's buttons that act now; the others stay in their place dimmed.
    pub acts: Vec<CloudButton>,
}

/// A service's card: its facts with the day they were read, shown before it is switched on and from Facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudCard {
    /// The service.
    pub service: ServiceName,
    /// Where the voice goes.
    pub place: Fact,
    /// Whether the service trains on the voice.
    pub trains: Fact,
    /// What the service keeps.
    pub keeps: Fact,
    /// What is free.
    pub free: Fact,
    /// The price of light use.
    pub light: UsePrice,
    /// The price of heavy use.
    pub heavy: UsePrice,
    /// The least a request bills when the service rounds up, such as Groq's 10 seconds; `None` when it bills the
    /// sound's own length.
    pub bills: Option<Fact>,
    /// The day these facts were read on the service's own pages.
    pub read: Date,
    /// They are older than `facts_days`, so their day takes the warning tone.
    pub old: bool,
    /// The host of the facts' page, which the link names, such as `groq.com`; `None` for the Custom row.
    pub site: Option<String>,
    /// The card's warning, for a service that trains by default, with its account switch's name as a `Text`; `None`
    /// for none.
    pub warn: Option<Fact>,
    /// No real key has checked this connection yet.
    pub untested: bool,
    /// The day it was switched on; `None` while off, so the card asks to switch it on.
    pub on: Option<Date>,
    /// The facts the shown page holds; the day read and Untested show on each.
    pub part: CardPart,
}

/// A service's key page: the key line with Paste and Test, the address and model lines and its switch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceSheet {
    /// The service.
    pub service: ServiceName,
    /// Where it stands.
    pub mark: ServiceMark,
    /// The key's characters, one dot each, 0 for none; it tells a secret's length, so never log it.
    pub key: Redacted<u32>,
    /// The key typed on its line and not saved yet, one dot per character, 0 for none; never log it either.
    pub typed: Redacted<u32>,
    /// How the key's test stands.
    pub check: KeyCheck,
    /// The address line's text, as typed, just refused or kept, for a typed row such as Custom; `None` for a row with
    /// its own address.
    pub address: Option<Redacted<String>>,
    /// Why the address on the line was refused, which its warning names; `None` while it is kept or empty.
    pub address_fault: Option<AddressFault>,
    /// The Other model line's id, as typed, just refused or kept, empty for none.
    pub model: Redacted<String>,
    /// The typed model has no host that keeps nothing while `keep` is on, so its line says why.
    pub model_na: bool,
    /// The id on the Other model line was refused, as the frames cannot carry it, so its line says why.
    pub model_refused: bool,
    /// `OpenRouter`'s keep-nothing switch, for a row that has one.
    pub keep: Option<bool>,
    /// The host where keys are made, which the link names; `None` for the Custom row.
    pub site: Option<String>,
    /// The page's buttons that act now; the others stay in their place dimmed.
    pub acts: Vec<CloudButton>,
    /// The lines the shown page holds; `None` while one page holds them all.
    pub part: Option<LinesPart>,
    /// The shown line that takes the letters while the panel is aimed at.
    pub typing: CloudLine,
    /// That line's caret: UTF-16 units of the text it draws, or dots on the key line; it may tell a secret's length.
    pub caret: Redacted<u32>,
    /// The shown lines whose Save acts now, as each holds text.
    pub saves: Vec<CloudLine>,
}
