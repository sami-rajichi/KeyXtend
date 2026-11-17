//! The cloud models a language may pick (design "Choosing"): each switched-on service's listed models that know it,
//! then its typed Other id, and why one cannot be used now. The Models page's Cloud side and Compare list them.

use kx_module_api::ports::Unavailable;

use super::{Choice, Chosen, Cloud, typed};
use crate::services::Service;
use crate::settings::VoiceSettings;

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
        match self.model {
            Chosen::Listed(m) => &m.id,
            Chosen::Typed(id) => id,
        }
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

    /// Why `service`'s `model` cannot be used now: a model every host of which keeps the voice, while the row's
    /// keep-nothing switch is on (edge 17). A typed id's hosts are checked later, so it is never unavailable here.
    #[must_use]
    pub fn unavailable(
        &self,
        s: &VoiceSettings,
        service: &Service,
        model: Chosen<'_>,
    ) -> Option<Unavailable> {
        let keeps = matches!(model, Chosen::Listed(m) if m.keeps);
        let on = self.entry(s, service).is_some_and(|e| e.keep);
        (keeps && on && service.keep.is_some()).then_some(Unavailable::KeepNothing)
    }
}
