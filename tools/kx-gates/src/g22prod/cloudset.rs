//! The cloud line's plan: each offered service with a key on this PC, its first model on the clip languages it knows,
//! a typed row's address and model from the daily settings, and the copy's `[voice]` with only that service on.

use kx_mod_voice::cloud::pick_text;
use kx_mod_voice::services::{Address, Service, Services, piece_s};
use kx_mod_voice::{ServiceOn, VoiceSettings};
use kx_module_api::ports::{CloudAsk, CloudLimits, Date, fits_iso};
use kx_module_api::{MS_PER_S, Redacted};

/// Why a service is skipped.
const NO_KEY: &str = "no sealed key in the daily data folder";
const NO_LINES: &str = "the daily settings hold no typed address and model for it";
const BAD_ADDRESS: &str = "the daily typed address is not one the row takes";
pub(super) const BAD_MODEL: &str = "the daily typed model is not one the row takes";
pub(super) const NO_LANG: &str = "its model knows none of the clip languages";

/// A typed row's address and model, as the daily settings hold them; never shown.
#[derive(Clone, Debug)]
pub(super) struct Typed {
    pub(super) address: Redacted<String>,
    pub(super) model: Redacted<String>,
}

/// A service the cloud line runs: its row, its model, the clip languages it dictates, and a typed row's address.
#[derive(Debug)]
pub(super) struct Planned<'a> {
    pub(super) row: &'a Service,
    pub(super) model: Redacted<String>,
    pub(super) langs: Vec<String>,
    address: Option<Redacted<String>>,
}

/// One offered service's place in the cloud line.
#[derive(Debug)]
pub(super) enum Plan<'a> {
    /// It runs.
    Run(Planned<'a>),
    /// It is skipped, and why.
    Skip { id: &'a str, why: &'static str },
}

impl<'a> Plan<'a> {
    /// The service's id.
    pub(super) fn id(&self) -> &'a str {
        match self {
            Self::Run(p) => &p.row.id,
            Self::Skip { id, .. } => id,
        }
    }

    /// What it runs; none when skipped.
    pub(super) fn run(&self) -> Option<&Planned<'a>> {
        match self {
            Self::Run(p) => Some(p),
            Self::Skip { .. } => None,
        }
    }
}

/// Every offered row of `table` in order, run when `has` its key, on `langs`, a typed row with its `typed` lines.
pub(super) fn plan<'a>(
    table: &'a Services,
    has: impl Fn(&str) -> bool,
    langs: &[&str],
    typed: impl Fn(&str) -> Option<Typed>,
) -> Vec<Plan<'a>> {
    let each = |row: &'a Service| match planned(row, &has, langs, &typed) {
        Ok(p) => Plan::Run(p),
        Err(why) => Plan::Skip { id: &row.id, why },
    };
    table.offered().map(each).collect()
}

/// `row`'s run, or why it is skipped.
fn planned<'a>(
    row: &'a Service,
    has: &impl Fn(&str) -> bool,
    langs: &[&str],
    typed: &impl Fn(&str) -> Option<Typed>,
) -> Result<Planned<'a>, &'static str> {
    if !has(&row.id) {
        return Err(NO_KEY);
    }
    let typed_row = matches!(row.address, Address::Typed { .. });
    let lines = if typed_row || row.models.is_empty() {
        Some(typed(&row.id).ok_or(NO_LINES)?)
    } else {
        None
    };
    let address = match &lines {
        Some(t) if typed_row && row.url(t.address.expose()).is_some() => Some(t.address.clone()),
        _ if typed_row => return Err(BAD_ADDRESS),
        _ => None,
    };
    let (model, langs) = model_of(row, lines, langs)?;
    if langs.is_empty() {
        return Err(NO_LANG);
    }
    Ok(Planned {
        row,
        model,
        langs,
        address,
    })
}

/// `row`'s first model and the clip `langs` it has a code for, or the typed model on the `langs` that go as they are.
fn model_of(
    row: &Service,
    lines: Option<Typed>,
    langs: &[&str],
) -> Result<(Redacted<String>, Vec<String>), &'static str> {
    let keep = |fits: &dyn Fn(&str) -> bool| {
        let kept = langs.iter().filter(|l| fits_iso(l) && fits(l));
        kept.map(|l| (*l).to_owned()).collect()
    };
    if let Some(first) = row.models.first() {
        return Ok((
            Redacted::new(first.id.clone()),
            keep(&|l| first.code(l).is_some()),
        ));
    }
    let t = lines.ok_or(NO_LINES)?;
    if !row.other || !CloudAsk::model_fits(t.model.expose()) {
        return Err(BAD_MODEL);
    }
    Ok((t.model, keep(&CloudAsk::lang_fits)))
}

/// The copy's `[voice]` for `p`: cloud on, only its service on, confirmed on `day` at its terms with the warning seen,
/// and each of its languages picking its model.
pub(super) fn voice(p: &Planned, day: Date) -> Result<VoiceSettings, String> {
    let mut s = VoiceSettings::from_defaults().map_err(|e| format!("the voice defaults: {e}"))?;
    let mut on = ServiceOn::new(day, p.row.terms);
    on.warned = p.row.terms;
    if let Some(address) = &p.address {
        on.address.clone_from(address.expose());
    }
    if p.row.models.is_empty() {
        on.model.clone_from(p.model.expose());
    }
    s.cloud = true;
    s.services = [(p.row.id.clone(), on)].into();
    let pick = pick_text(&p.row.id, p.model.expose());
    s.picks
        .extend(p.langs.iter().map(|l| (l.clone(), pick.clone())));
    Ok(s)
}

/// The requests a clip of `clip_ms` takes within `limits`, as a listen-only run cuts it (`piece_s`): at least one.
pub(super) fn pieces(clip_ms: u32, limits: &CloudLimits) -> u64 {
    let piece_ms = u64::from(piece_s(limits)) * u64::from(MS_PER_S);
    u64::from(clip_ms).div_ceil(piece_ms).max(1)
}

/// The requests the runs of `plans` send for `clips`, each a language and its length in ms.
pub(super) fn requests(plans: &[Plan], clips: &[(&str, u32)]) -> u64 {
    let of = |p: &Planned| {
        let mine = clips.iter().filter(|(l, _)| p.langs.iter().any(|m| m == l));
        mine.map(|(_, ms)| pieces(*ms, &p.row.limits)).sum::<u64>()
    };
    plans.iter().filter_map(Plan::run).map(of).sum()
}
