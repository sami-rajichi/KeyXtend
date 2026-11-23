//! G22's cloud line: each offered service with a sealed key in the owner's daily data folder dictates the clips its
//! model knows on a copy of its own, with that key borrowed still sealed and only that service on. A service without a
//! key is skipped, the line is skipped while `[g22_cloud] daily_dir` is unset, and it fails when that names no data
//! folder.

use std::collections::BTreeMap;
use std::path::Path;

use kx_mod_voice::services::table;
use kx_mod_voice::{ID, SPEC, ServiceOn, VoiceSettings};
use kx_module_api::Redacted;
use kx_module_api::ports::Calendar;
use kx_platform::ui::{Hidden, HiddenError, Probe};
use kx_platform_windows::{WinCalendar, WinHidden};
use kx_settings::{Files, peek, peek_section};
use serde_json::{Value, json};

use super::cloudset::{self, Plan, Planned, Typed};
use super::line::{self, Line};
use super::{Case, GATE, borrow, say, table_of};
use crate::apps::Ctx;
use crate::freshcopy::Fresh;
use crate::g22;
use crate::g22cfg::{DAILY, G22Cloud};
use crate::product::NAME;

/// The cloud line of the gate build at `program` on `cases`, judged as the local line by `s`: its fields, with `pass`
/// none when no service ran.
pub(super) fn run(ctx: &Ctx, program: &Path, cases: &[Case], s: &VoiceSettings) -> Value {
    let Some(daily) = ctx.cfg.g22_cloud.daily() else {
        let why = G22Cloud::unset();
        say!("{GATE} {NAME} cloud: skipped, {why}");
        return json!({ "skipped": why });
    };
    if !borrow::is_data(daily) {
        let why = format!(
            "{DAILY} holds no settings file and no voice data key, so it is no data folder"
        );
        say!("{GATE} {NAME} cloud: {why}");
        return json!({ "error": why, "pass": false });
    }
    let services = match table() {
        Ok(t) => t,
        Err(e) => return json!({ "error": e.to_string(), "pass": false }),
    };
    let langs: Vec<&str> = cases.iter().map(|c| c.lang.as_str()).collect();
    let plans = cloudset::plan(services, |id| borrow::has(daily, id), &langs, typed(daily));
    let clips: Vec<(&str, u32)> = cases.iter().map(|c| (c.lang.as_str(), c.clip_ms)).collect();
    let runs = plans.iter().filter_map(Plan::run).count();
    let asks = cloudset::requests(&plans, &clips);
    say!("{GATE} {NAME} cloud: {runs} services, {asks} requests");
    let lines: Vec<Value> = plans
        .iter()
        .map(|p| match p {
            Plan::Run(run) => service(ctx, program, (cases, s), run, daily),
            Plan::Skip { why, .. } => json!({ "service": p.id(), "skipped": why }),
        })
        .collect();
    let mut ran = lines.iter().filter(|l| l.get("skipped").is_none());
    let pass = (runs > 0).then(|| ran.all(|l| l["pass"] == true));
    json!({ "services": lines, "requests": asks, "pass": pass })
}

/// The typed address and model each service holds in the daily settings, when it holds both, and nothing else of
/// them; it reads that file and never writes it.
pub(super) fn typed(daily: &Path) -> impl Fn(&str) -> Option<Typed> {
    let file = peek(&Files::new(daily));
    let read = peek_section::<VoiceSettings>(file.as_ref(), ID, &SPEC).ok();
    let both = |(_, on): &(String, ServiceOn)| !on.address.is_empty() && !on.model.is_empty();
    let lines: BTreeMap<String, Typed> = (read.into_iter().flat_map(|s| s.services))
        .filter(both)
        .map(|(id, on)| {
            let (address, model) = (Redacted::new(on.address), Redacted::new(on.model));
            (id, Typed { address, model })
        })
        .collect();
    move |id| lines.get(id).cloned()
}

/// Service `p` on the cases its model knows, on a copy with its key borrowed from `daily` and only it on.
fn service(
    ctx: &Ctx,
    program: &Path,
    (cases, s): (&[Case], &VoiceSettings),
    p: &Planned,
    daily: &Path,
) -> Value {
    let id = p.row.id.as_str();
    eprintln!("{} {NAME} cloud: {id}", g22::GATE);
    let mine: Vec<&Case> = cases.iter().filter(|c| p.langs.contains(&c.lang)).collect();
    let line = Line::cloud(ctx, id);
    let ready = |f: &Fresh| {
        hidden_why(&WinHidden::new().probe(f.data()))?;
        for file in borrow::copy(daily, f.data(), id)? {
            f.lend(file)?;
        }
        let voice = cloudset::voice(p, WinCalendar.now().date)?;
        f.write(&table_of(&voice, &ctx.cfg.g22_product)?)
    };
    let got = line::on_copy(ctx, program, (&mine, s, &line), ready);
    let mut out = got.unwrap_or_else(|e| json!({ "error": e, "pass": false }));
    out["service"] = json!(id);
    out
}

/// Refuses a copy whose saves `probe` says would be hidden (ADR-0037), which keeps cloud off; it never names the app.
pub(super) fn hidden_why(probe: &Result<Probe, HiddenError>) -> Result<(), String> {
    match probe {
        Ok(Probe::Normal) => Ok(()),
        Ok(Probe::Hidden { .. }) => {
            Err("the copy would be a hidden run (ADR-0037), which keeps cloud off".into())
        }
        Err(e) => Err(format!("the copy's data folder could not be probed: {e}")),
    }
}
