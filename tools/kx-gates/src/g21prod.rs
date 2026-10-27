//! G21's product half, on a copy of the product it starts with learning paused: in target-window's password box,
//! the Chrome page's password field and target-window's plain box, the product's own field watch (`WinFieldWatch`)
//! tells each box's kind, the product's keys type `[g21_product]`'s case, and the lane must show the lock and no chip
//! in a password box, and chips in the plain one.
#![cfg(windows)]

use kx_mod_predict::PredictSettings;
use kx_module_api::ports::{FieldConfig, FieldKind, FieldRun, FieldWatch};
use kx_platform_windows::WinFieldWatch;
use kx_platform_windows::uia::Uia;
use kx_settings::peek_section;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::copyend::Ended;
use crate::g27lane::Lane;
use crate::g27type::Typist;
use crate::owncopy::Session;
use crate::p7cfg::Case;
use crate::product::{self, Keyboard};
use crate::win::sleep_ms;
use crate::{g21, probe, typebox};

/// A box the half types in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    /// target-window started as a password box.
    Win32Password,
    /// The password field of G21's Chrome page.
    ChromePassword,
    /// target-window's plain box.
    Win32Plain,
}

/// The boxes in the order they are typed in.
const FIELDS: [Field; 3] = [
    Field::Win32Password,
    Field::ChromePassword,
    Field::Win32Plain,
];
/// How many boxes the product half types in, for its run-length check.
pub(crate) const BOXES: usize = FIELDS.len();

impl Field {
    /// Its name in the result.
    fn name(self) -> &'static str {
        match self {
            Self::Win32Password => "win32-password",
            Self::ChromePassword => "chrome-password",
            Self::Win32Plain => "win32-plain",
        }
    }

    /// True for a password box.
    fn password(self) -> bool {
        self != Self::Win32Plain
    }
}

/// True when a box passed: a password box is told as one and shows the lock with no chip in the lane's tree; a plain
/// box is told as text and shows chips and no lock.
pub(crate) fn box_ok(password: bool, kind: FieldKind, lane: &Lane) -> bool {
    if password {
        kind == FieldKind::Password && lane.locked && lane.chips == 0
    } else {
        kind == FieldKind::Text && !lane.locked && lane.shown > 0
    }
}

/// How the running copy's predict module reads boxes, so the gate's watch reads them the same way.
fn field_config() -> Result<FieldConfig, String> {
    let file = product::running_file()?;
    let id = kx_mod_predict::ID;
    let s: PredictSettings = peek_section(file.as_ref(), id, &kx_mod_predict::SPEC)
        .map_err(|e| format!("the predict settings: {e}"))?;
    Ok(s.field_config())
}

/// The lane after the case's prefix: a password box's after the whole chip wait, as a chip may come late; a plain
/// box's once a chip shows.
fn typed_lane(ctx: &Ctx, t: &Typist, case: &Case, password: bool) -> Result<Lane, String> {
    let layout = t.switch(&case.lang)?;
    t.type_in(layout, &case.prefix)?;
    if password {
        sleep_ms(ctx.cfg.g27.chip_wait_ms);
        return t.lane();
    }
    t.lane_until(|l| l.shown > 0).map_or_else(|| t.lane(), Ok)
}

/// The focused box judged with typist `t`: the kind the watch tells, and the lane after the case's prefix.
fn judge(ctx: &Ctx, watch: &dyn FieldRun, t: &Typist, f: Field) -> Result<Value, String> {
    let kind = g21::told(ctx, watch, g21::wanted(f.password()));
    let g = &ctx.cfg;
    let case = g
        .g27
        .case(&g.g21_product.case)
        .ok_or("g21_product.case names no g27 case")?;
    let lane = typed_lane(ctx, t, case, f.password())?;
    let ok = box_ok(f.password(), kind, &lane);
    Ok(json!({
        "field": f.name(), "kind": g21::kind_name(kind), "locked": lane.locked, "chips": lane.chips,
        "shown": lane.shown, "ok": ok,
    }))
}

/// A box's line `v` with whether its layout came `back` and what clean-up `left`; it passes only with the layout back.
fn finished(mut v: Value, back: bool, left: &[String]) -> Value {
    v["ok"] = json!(v["ok"] == true && back);
    v["layout_put_back"] = json!(back);
    v["clean_up"] = json!(left);
    v
}

/// target-window's box, a password box for `f`'s password, judged in a fresh window.
fn target(ctx: &Ctx, watch: &dyn FieldRun, f: Field) -> Result<Value, String> {
    let extra = typebox::password_args(ctx, f.password());
    let b = typebox::in_target(ctx, extra, |t| judge(ctx, watch, t, f));
    let left = b.left;
    b.got.map(|v| finished(v, b.back, &left))
}

/// G21's Chrome page, its password field clicked, judged, then closed with the page file G21 wrote.
fn chrome(ctx: &Ctx, watch: &dyn FieldRun) -> Result<Value, String> {
    let g = &ctx.cfg.g21;
    let doc = probe::open_page(ctx, &g21::page(ctx))?;
    let ((v, back), left) = probe::run_on(ctx, doc, |doc| {
        let uia = Uia::new().map_err(|e| e.to_string())?;
        probe::front(ctx, &doc.app)?;
        g21::click_field(ctx, &uia, &doc.app, &g.pass_name)?;
        let judged = |t: &Typist| judge(ctx, watch, t, Field::ChromePassword);
        let (got, back) = typebox::typed_in(ctx, &doc.app, judged);
        got.map(|v| (v, back))
    })?;
    Ok(finished(v, back, &left))
}

/// Each box judged on the running copy, with one field watch for them all.
fn boxes(ctx: &Ctx) -> Result<Vec<Value>, String> {
    Keyboard::find()?;
    let tell = Box::new(|_| {});
    let watch = WinFieldWatch
        .watch(field_config()?, tell)
        .map_err(|e| e.to_string())?;
    let one = |f: Field| {
        let got = match f {
            Field::ChromePassword => chrome(ctx, &*watch),
            _ => target(ctx, &*watch, f),
        };
        got.unwrap_or_else(|e| json!({ "field": f.name(), "ok": false, "error": e }))
    };
    Ok(FIELDS.into_iter().map(one).collect())
}

/// Runs the product half on a copy of the product it starts with learning paused, which ends before the owner's
/// settings file comes back; refused while the product runs.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    let session = Session::open(ctx)?;
    let (fields, copy) = session.with_copy(&[], || boxes(ctx));
    let settings = session.close();
    let fields = fields.unwrap_or_else(|e| vec![json!({ "ok": false, "error": e })]);
    let ended = copy.as_ref().is_some_and(Ended::ok);
    let typed = fields.iter().all(|f| f["ok"] == true);
    Ok(json!({
        "gate": "G21", "app": product::NAME, "fields": fields, "copy": copy.map(|c| c.line()),
        "settings_put_back": settings.is_ok(), "settings_error": settings.as_ref().err(),
        "pass": typed && ended && settings.is_ok(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lane(locked: bool, chips: usize, shown: usize) -> Lane {
        Lane {
            locked,
            chips,
            shown,
            first: None,
        }
    }

    #[test]
    fn a_password_box_passes_with_the_lock_shown_and_no_chip() {
        assert!(box_ok(true, FieldKind::Password, &lane(true, 0, 0)));
    }

    #[test]
    fn a_chip_under_a_password_box_fails_even_one_not_shown() {
        assert!(!box_ok(true, FieldKind::Password, &lane(true, 1, 1)));
        assert!(
            !box_ok(true, FieldKind::Password, &lane(true, 1, 0)),
            "in the tree"
        );
    }

    #[test]
    fn a_password_box_without_the_lock_or_told_otherwise_fails() {
        assert!(
            !box_ok(true, FieldKind::Password, &lane(false, 0, 0)),
            "no lock"
        );
        for kind in [FieldKind::Text, FieldKind::Unknown, FieldKind::None] {
            assert!(!box_ok(true, kind, &lane(true, 0, 0)), "{kind:?}");
        }
    }

    #[test]
    fn a_plain_box_passes_only_with_chips_shown_no_lock_and_told_as_text() {
        assert!(box_ok(false, FieldKind::Text, &lane(false, 3, 3)));
        assert!(
            !box_ok(false, FieldKind::Text, &lane(false, 0, 0)),
            "no chips"
        );
        assert!(
            !box_ok(false, FieldKind::Text, &lane(false, 2, 0)),
            "none shown"
        );
        assert!(!box_ok(false, FieldKind::Text, &lane(true, 3, 3)), "a lock");
        assert!(!box_ok(false, FieldKind::Unknown, &lane(false, 3, 3)));
    }

    #[test]
    fn two_password_boxes_and_one_plain_box_are_typed_in() {
        let passwords = FIELDS.iter().filter(|f| f.password()).count();
        assert_eq!((FIELDS.len(), passwords), (3, 2));
        let names: Vec<&str> = FIELDS.iter().map(|f| f.name()).collect();
        assert_eq!(names, ["win32-password", "chrome-password", "win32-plain"]);
    }
}
