//! The cloud models a language may pick (design "Choosing"): each switched-on service's listed models that know it,
//! then its typed Other id, why one cannot be used now, and the pick waiting for a service Off. The Models page's Cloud
//! side and Compare list them.

use kx_module_api::ports::Unavailable;

use super::{Choice, Chosen, Cloud, entry_ok, reaches, typed};
use crate::services::Service;
use crate::settings::{ServiceOn, VoiceSettings};

/// A cloud model a language may pick: its service and the model.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Offered<'a> {
    /// The service.
    pub service: &'a Service,
    /// The model.
    pub model: Chosen<'a>,
}

impl Offered<'_> {
    /// The id a pick names: the listed model's, or the typed one.
    #[must_use]
    pub fn id(&self) -> &str {
        self.model.id()
    }

    /// The shown name: the listed model's, or the typed id.
    #[must_use]
    pub fn name(&self) -> &str {
        match self.model {
            Chosen::Listed(m) => &m.name,
            Chosen::Typed(id) => id,
        }
    }
}

impl<'a> From<&Choice<'a>> for Offered<'a> {
    fn from(choice: &Choice<'a>) -> Self {
        Self {
            service: choice.service,
            model: choice.model,
        }
    }
}

impl<'a> Cloud<'a> {
    /// Row `id` of the table.
    #[must_use]
    pub fn row(&self, id: &str) -> Option<&'a Service> {
        self.table.get(id)
    }

    /// The models `lang` may pick, in the Cloud page's order: each switched-on service's listed models that know it,
    /// then its typed Other id; none while cloud is off or in a hidden run. A key or an address may still be missing.
    #[must_use]
    pub fn models<'s>(&self, s: &'s VoiceSettings, lang: &str) -> Vec<Offered<'s>>
    where
        'a: 's,
    {
        let mut out = Vec::new();
        for service in self.offered(s) {
            let Some(on) = self.entry(s, service) else {
                continue;
            };
            let listed = service.models.iter().filter(|m| m.code(lang).is_some());
            out.extend(listed.map(|m| Offered {
                service,
                model: Chosen::Listed(m),
            }));
            if typed(service, on, &on.model, lang) {
                let model = Chosen::Typed(on.model.as_str());
                out.push(Offered { service, model });
            }
        }
        out
    }

    /// The model `lang`'s pick keeps while its offered service is Off, or its entry stale, and cloud is on (owner,
    /// 2026-11-21): a listed one, or the typed id its parked entry holds (ADR-0054); only that service's Confirm brings
    /// it back into use.
    #[must_use]
    pub fn waits<'s>(&self, s: &'s VoiceSettings, lang: &str) -> Option<Offered<'s>>
    where
        'a: 's,
    {
        let (id, model) = self.table.pick(s.picks.get(lang)?)?;
        let service = self.table.get(id).filter(|r| r.offered && self.on(s))?;
        if self.entry(s, service).is_some() {
            return None;
        }
        let model = if let Some(listed) = service.model(model) {
            listed.code(lang).map(|_| Chosen::Listed(listed))?
        } else {
            let held = |e: &&ServiceOn| entry_ok(service, e) && reaches(service, e);
            let parked = s.parked.get(id).filter(held)?;
            typed(service, parked, model, lang).then_some(Chosen::Typed(parked.model.as_str()))?
        };
        Some(Offered { service, model })
    }

    /// What service `id`'s `model` is for `lang` now, as Compare sends to it; `None` when the rules do not take it.
    #[must_use]
    pub fn model_for<'s>(
        &self,
        s: &'s VoiceSettings,
        lang: &str,
        id: &str,
        model: &str,
    ) -> Option<Choice<'s>>
    where
        'a: 's,
    {
        self.resolve(s, lang, id, model)
    }

    /// Why `service`'s `model` cannot be used now: a model every host of which keeps the voice, listed or typed by its
    /// id, while the row's keep-nothing switch is on (edge 17). Another typed id still sends the keep-nothing field,
    /// and with no host that keeps nothing its refusal reads as Strange until task 24 captures the real reply.
    #[must_use]
    pub fn unavailable(
        &self,
        s: &VoiceSettings,
        service: &Service,
        model: Chosen<'_>,
    ) -> Option<Unavailable> {
        let listed = match model {
            Chosen::Listed(m) => Some(m),
            Chosen::Typed(id) => service.model(id),
        };
        let keeps = listed.is_some_and(|m| m.keeps);
        let on = self.entry(s, service).is_some_and(|e| e.keep);
        (keeps && on && service.keep.is_some()).then_some(Unavailable::KeepNothing)
    }
}
