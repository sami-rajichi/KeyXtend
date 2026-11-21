//! The cloud rules (ADR-0047 decisions 9 and 12): which services the Cloud page offers, where each stands, a card's
//! facts, and what a language dictates with. Nothing moves a language from its local model to a service but the
//! user's own pick, and a hidden run or `cloud` off offers none.

mod acts;
mod offer;

pub use offer::Offered;

use kx_module_api::ports::{
    CardPart, CloudAsk, CloudCard, Date, Fact, ServiceMark, Unavailable, UsePrice, fits_iso,
};

use crate::services::{self, Address, Model, Service, Services};
use crate::settings::{ServiceOn, VoiceSettings};

/// What parts a cloud pick's service id from its model's id, as in `groq:whisper-large-v3-turbo`; a service id never
/// holds it, so the first one parts them when a row id comes before it.
pub const PICK_MARK: char = ':';

/// The pick that names `service`'s `model`.
#[must_use]
pub fn pick_text(service: &str, model: &str) -> String {
    format!("{service}{PICK_MARK}{model}")
}

/// The service id and model id a cloud pick names, by the shipped table (`Services::pick`); `None` for a local pick.
#[must_use]
pub fn cloud_pick(value: &str) -> Option<(&str, &str)> {
    services::table().ok()?.pick(value)
}

/// The shipped row, the model id and, when the row lists it, the listed model that cloud pick `value` names.
#[must_use]
pub fn pick_row(value: &str) -> Option<(&'static Service, &str, Option<&'static Model>)> {
    let table = services::table().ok()?;
    let (id, model) = table.pick(value)?;
    let row = table.get(id)?;
    Some((row, model, row.model(model)))
}

/// The model a cloud pick names.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Chosen<'a> {
    /// One of the row's models.
    Listed(&'a Model),
    /// The id the user typed on the Other model line.
    Typed(&'a str),
}

impl<'a> Chosen<'a> {
    /// The id a pick names: the listed model's, or the typed one.
    #[must_use]
    pub fn id(self) -> &'a str {
        match self {
            Self::Listed(m) => &m.id,
            Self::Typed(id) => id,
        }
    }

    /// The model id an ask for this model sends: a listed model's own unless its request names none, or the typed id.
    #[must_use]
    pub fn sent(self) -> Option<&'a str> {
        match self {
            Self::Listed(m) => (!m.unnamed).then_some(m.id.as_str()),
            Self::Typed(id) => Some(id),
        }
    }
}

/// A language's cloud pick, as the user made it and the rules still take it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice<'a> {
    /// The service.
    pub service: &'a Service,
    /// The model.
    pub model: Chosen<'a>,
    /// The code the service wants for the language: the model's own, or the keyboard's as it is for a typed one.
    pub code: String,
    /// The row's terms number is newer than the one confirmed, so the user is asked again before the next upload.
    pub ask: bool,
    /// Why its model cannot be used now; the flow then says so at Start and sends nothing.
    pub unavailable: Option<Unavailable>,
    /// The card's warning, until a dictation at the row's terms number showed it; `None` for none or once shown.
    pub warn: Option<Fact>,
}

/// The rules over a services table, in a run that is hidden or not.
#[derive(Copy, Clone, Debug)]
pub struct Cloud<'a> {
    /// The services table.
    table: &'a Services,
    /// The run's saves would be hidden (ADR-0037), so cloud is off.
    hidden: bool,
}

impl<'a> Cloud<'a> {
    /// The rules over `table`; a `hidden` run offers nothing and reads no key.
    #[must_use]
    pub fn new(table: &'a Services, hidden: bool) -> Self {
        Self { table, hidden }
    }

    /// True when cloud services may be used: `cloud` on, in a run that is not hidden.
    #[must_use]
    pub fn on(&self, s: &VoiceSettings) -> bool {
        s.cloud && !self.hidden
    }

    /// The services cloud offers, in order, Groq first (edge 24); none while cloud is off (edges 11 and 21).
    #[must_use]
    pub fn offered(&self, s: &VoiceSettings) -> Vec<&'a Service> {
        if !self.on(s) {
            return Vec::new();
        }
        self.table.offered().collect()
    }

    /// Where `service` stands: Off until the user confirms its card while cloud is on, then it needs a key, then a
    /// typed row needs its address, so a user with a key is never told to add one; Ready once it has both.
    #[must_use]
    pub fn mark(&self, s: &VoiceSettings, service: &Service, key: bool) -> ServiceMark {
        match self.entry(s, service) {
            None => ServiceMark::Off,
            Some(_) if !key => ServiceMark::NeedsKey,
            Some(on) if !reaches(service, on) => ServiceMark::NeedsAddress,
            Some(_) => ServiceMark::Ready,
        }
    }

    /// True when `service` is on with a terms number older than its row's, so the next upload asks again (edge 16).
    #[must_use]
    pub fn asks(&self, s: &VoiceSettings, service: &Service) -> bool {
        self.entry(s, service)
            .is_some_and(|on| on.terms < service.terms)
    }

    /// What `lang` dictates with in the cloud; `None` means its local model, as for cloud off, a hidden run, a service
    /// not on or not offered, or a model that does not know the language (edge 21). A model the keep-nothing switch
    /// rules out still resolves, marked unavailable, so nothing moves to the local model by itself (edge 17).
    #[must_use]
    pub fn choice<'s>(&self, s: &'s VoiceSettings, lang: &str) -> Option<Choice<'s>>
    where
        'a: 's,
    {
        let (id, model) = self.table.pick(s.picks.get(lang)?)?;
        self.resolve(s, lang, id, model)
    }

    /// `service`'s card on `today`: its facts with their day, which takes the warning tone past `facts_days` (edge 15).
    #[must_use]
    pub fn card(&self, s: &VoiceSettings, service: &Service, today: Date) -> CloudCard {
        let facts = &service.facts;
        let price = |hours, fact: &Fact| UsePrice {
            hours,
            price: fact.clone(),
        };
        CloudCard {
            service: service.shown(),
            place: facts.place.clone(),
            trains: facts.trains.clone(),
            keeps: facts.keeps.clone(),
            free: facts.free.clone(),
            light: price(self.table.light_hours(), &facts.light),
            heavy: price(self.table.heavy_hours(), &facts.heavy),
            bills: facts.bills.clone(),
            rule: facts.rule.clone(),
            read: facts.read,
            old: facts.old(today, s.facts_days),
            site: facts.source.as_ref().map(|url| url.host().to_owned()),
            warn: facts.warn.clone(),
            untested: service.untested,
            on: self.entry(s, service).and_then(ServiceOn::day),
            part: CardPart::Voice,
        }
    }

    /// `service`'s entry while it is on: cloud on, the row offered, and an entry the row still takes.
    fn entry<'s>(&self, s: &'s VoiceSettings, service: &Service) -> Option<&'s ServiceOn> {
        let on = self.on(s) && service.offered;
        let entry = s.services.get(&service.id).filter(|e| entry_ok(service, e));
        entry.filter(|_| on)
    }

    /// The choice that `id`'s `model` would be for `lang`: a listed model that knows it, or the typed Other id.
    fn resolve<'s>(
        &self,
        s: &'s VoiceSettings,
        lang: &str,
        id: &str,
        model: &str,
    ) -> Option<Choice<'s>>
    where
        'a: 's,
    {
        let service: &'s Service = self.table.get(id)?;
        let entry = self.entry(s, service)?;
        if !fits_iso(lang) || !reaches(service, entry) {
            return None;
        }
        let (model, code) = match service.model(model) {
            Some(listed) => (Chosen::Listed(listed), listed.code(lang)?.to_owned()),
            None if typed(service, entry, model, lang) => {
                (Chosen::Typed(entry.model.as_str()), lang.to_owned())
            }
            None => return None,
        };
        let unavailable = self.unavailable(s, service, model);
        let ask = entry.terms < service.terms;
        let warn = (service.facts.warn.clone()).filter(|_| entry.warned != service.terms);
        Some(Choice {
            service,
            model,
            code,
            ask,
            unavailable,
            warn,
        })
    }
}

/// True when `model` is `entry`'s typed Other id on a row with that line, no listed model's, so it never shows twice,
/// and `lang` goes as it is.
fn typed(service: &Service, entry: &ServiceOn, model: &str, lang: &str) -> bool {
    let own = !entry.model.is_empty() && entry.model == model && service.model(model).is_none();
    service.other && own && CloudAsk::lang_fits(lang)
}

/// True when `service`'s pieces have somewhere to go: its own address, or one typed for a typed row (edge 13).
fn reaches(service: &Service, on: &ServiceOn) -> bool {
    service.url(&on.address).is_some()
}

/// True when `on` is an entry `row` still takes: a plain day, a terms number from 1 to the row's, so a hand edit
/// cannot skip a later one, and typed lines the row allows.
pub(crate) fn entry_ok(row: &Service, on: &ServiceOn) -> bool {
    let typed_row = matches!(row.address, Address::Typed { .. });
    let address = on.address.is_empty() || (typed_row && row.url(&on.address).is_some());
    let model = on.model.is_empty() || (row.other && CloudAsk::model_fits(&on.model));
    let terms = (1..=row.terms).contains(&on.terms);
    on.day().is_some() && terms && address && model
}

#[cfg(test)]
mod tests;
