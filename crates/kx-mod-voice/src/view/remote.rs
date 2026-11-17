//! Cloud models as the Models page's Cloud side and Compare list them (design "Choosing"): each switched-on service's
//! models that know the language in front, with its service, its mark and why one cannot be used now.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    CloudBand, CloudRow, CompareCloud, ModelRow, ModelState, PanelButton, ServiceMark, ServiceName,
};

use super::Facts;
use crate::cloud::Offered;
use crate::panel::{Key, Panel};

impl Panel {
    /// The Cloud side's rows for the language in front, each keyed by its service and pick id, unticked.
    pub(crate) fn cloud_list(facts: &Facts<'_>) -> Vec<(Key, ModelRow)> {
        let Some(cloud) = facts.cloud else {
            return Vec::new();
        };
        let s = facts.settings;
        let used = cloud
            .choice(s, facts.iso())
            .map(|c| key_of(Offered::from(&c)));
        let row = |o: Offered<'_>| {
            let cloud = CloudRow {
                service: o.service.shown(),
                mark: cloud.mark(s, o.service, facts.keyed.contains(&o.service.id)),
                unavailable: cloud.unavailable(s, o.service, o.model),
            };
            let key = key_of(o);
            let model = ModelRow {
                name: Redacted::new(o.name().to_owned()),
                bytes: 0,
                default: false,
                state: ModelState::Ready,
                used: used.as_ref() == Some(&key),
                tick: false,
                fetch: None,
                cloud: Some(cloud),
            };
            (key, model)
        };
        cloud.models(s, facts.iso()).into_iter().map(row).collect()
    }

    /// The Cloud side's band: cloud off, no service switched on yet, or the models listed.
    pub(crate) fn cloud_band(facts: &Facts<'_>) -> CloudBand {
        let on = |mark: &ServiceMark| *mark != ServiceMark::Off;
        match facts.cloud.filter(|c| c.on(facts.settings)) {
            None => CloudBand::Off,
            Some(_) if !Self::services(facts).iter().any(|(_, m)| on(m)) => CloudBand::NoneOn,
            Some(_) => CloudBand::Listed,
        }
    }

    /// The Cloud side's buttons: Use on a ticked model whose service is ready, that can be used and is not in use.
    pub(crate) fn cloud_side_acts(&self, facts: &Facts<'_>) -> Vec<PanelButton> {
        let ticked = self.ticked();
        let list = Self::cloud_list(facts);
        let row = list.iter().find(|(k, _)| Some(k) == ticked).map(|(_, r)| r);
        let usable = row.is_some_and(|r| !r.used && ready(r));
        usable.then_some(PanelButton::Use).into_iter().collect()
    }

    /// Compare's cloud models: the ready ones that can be used, ticked or not; and the ticked ones' services, once each.
    pub(crate) fn compare_clouds(
        &self,
        facts: &Facts<'_>,
    ) -> (Vec<(Key, CompareCloud)>, Vec<ServiceName>) {
        let mut send: Vec<ServiceName> = Vec::new();
        let mut list = Vec::new();
        for (key, row) in Self::cloud_list(facts) {
            let Some(cloud) = row.cloud.clone().filter(|_| ready(&row)) else {
                continue;
            };
            let tick = self.ticks.contains(&key);
            if tick && !send.contains(&cloud.service) {
                send.push(cloud.service.clone());
            }
            let shown = CompareCloud {
                name: row.name,
                service: cloud.service,
                tick,
            };
            list.push((key, shown));
        }
        (list, send)
    }
}

/// True for a cloud model whose service is ready and which can be used now.
fn ready(row: &ModelRow) -> bool {
    row.cloud
        .as_ref()
        .is_some_and(|c| c.mark == ServiceMark::Ready && c.unavailable.is_none())
}

/// The key a cloud model's row is ticked by.
fn key_of(o: Offered<'_>) -> Key {
    Key::Cloud {
        service: o.service.id.clone(),
        model: Redacted::new(o.id().to_owned()),
    }
}
