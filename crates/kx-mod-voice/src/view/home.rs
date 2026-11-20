//! The Voice panel's home list: the model and device the language in front gets, and how many services are ready.

use kx_module_api::Redacted;
use kx_module_api::ports::{ServiceMark, VoiceHome, VoiceTab};

use super::{Facts, device, short, shown_name};
use crate::cloud::Offered;
use crate::flow::Supply;
use crate::panel::{Panel, options};

/// The home list's values for the language in front, with `tabs`, the pages its shown page lists; a cloud pick names
/// its model, and the device stays the one picked.
pub(super) fn home(panel: &Panel, facts: &Facts<'_>, tabs: Vec<VoiceTab>) -> VoiceHome {
    let cloud = facts.cloud_choice();
    let ready = (cloud.is_none())
        .then(|| panel.stock(facts).supply(facts.iso()).ok())
        .flatten();
    let named = cloud.map(|c| Redacted::new(Offered::from(&c).name().to_owned()));
    let compared = facts.scores.get(facts.iso()).map(|b| b.day);
    VoiceHome {
        model: named.or_else(|| ready.as_ref().map(|r| shown_name(&r.name))),
        device: device::label(facts, ready.as_ref()),
        compared,
        history: u32::try_from(facts.history.len()).unwrap_or(u32::MAX),
        changed: options::changed(facts.settings),
        cloud: cloud_home(facts),
        tabs,
    }
}

/// The home's Cloud value: how many services are ready while cloud is on; `None` while off.
fn cloud_home(facts: &Facts<'_>) -> Option<u16> {
    let cloud = facts.cloud.filter(|c| c.on(facts.settings))?;
    let s = facts.settings;
    let ready = cloud.offered(s).into_iter().filter(|row| {
        let key = facts.keyed.contains(&row.id);
        cloud.mark(s, row, key) == ServiceMark::Ready
    });
    Some(short(ready.count()))
}
