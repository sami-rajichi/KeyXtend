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

use crate::apps::{self, Ctx, Opened};
use crate::g27lane::Lane;
use crate::g27type::{self, Typist};
use crate::owncopy::{Ended, Session};
use crate::p7cfg::Case;
use crate::product::{self, Keyboard};
use crate::win::sleep_ms;
use crate::{g21, launch, probe};

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

/// The kind `watch` tells once it is `want`, or the last it told within `[timing] read_wait_ms`.
fn told(ctx: &Ctx, watch: &dyn FieldRun, want: FieldKind) -> FieldKind {
    let t = &ctx.cfg.timing;
    let mut kind = watch.kind();
    crate::win::poll_until(t.read_wait_ms, t.poll_ms, || {
        kind = watch.kind();
        (kind == want).then_some(())
    });
    kind
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

/// `app`'s focused box judged: the kind the watch tells, the lane after the case's prefix, the layout put back.
fn judge(ctx: &Ctx, watch: &dyn FieldRun, app: &Opened, f: Field) -> Result<Value, String> {
    let want = if f.password() {
        FieldKind::Password
    } else {
        FieldKind::Text
    };
    let kind = told(ctx, watch, want);
    let g = &ctx.cfg;
    let case = g
        .g27
        .case(&g.g21_product.case)
        .ok_or("g21_product.case names no g27 case")?;
    let t = Typist::new(ctx, app)?;
    let lane = typed_lane(ctx, &t, case, f.password());
    let back = t.put_back();
    let lane = lane?;
    let ok = box_ok(f.password(), kind, &lane) && back;
    let kind = format!("{kind:?}").to_lowercase();
    Ok(json!({
        "field": f.name(), "kind": kind, "locked": lane.locked, "chips": lane.chips, "shown": lane.shown,
        "layout_put_back": back, "ok": ok,
    }))
}

/// target-window's box, a password box for `f`'s password, judged and then closed.
fn target(ctx: &Ctx, watch: &dyn FieldRun, f: Field) -> Result<Value, String> {
    let password = std::slice::from_ref(&ctx.target.password_arg);
    let extra = if f.password() { password } else { &[] };
    let app = launch::to_front(ctx, launch::start_target_with(ctx, extra)?)?;
    let got = g27type::focus_target(ctx, &app).and_then(|()| judge(ctx, watch, &app, f));
    let mut left = probe::released();
    left.extend(apps::clean_up(ctx, app));
    got.map(|mut v| {
        v["clean_up"] = json!(left);
        v
    })
}

/// G21's Chrome page, its password field clicked, judged, then closed.
fn chrome(ctx: &Ctx, watch: &dyn FieldRun) -> Result<Value, String> {
    let g = &ctx.cfg.g21;
    let body =
        g21::input(ctx, &g.user_name, "text", "") + &g21::input(ctx, &g.pass_name, "password", "");
    let doc = probe::open_page(ctx, &body)?;
    let (mut v, left) = probe::run_on(ctx, doc, |doc| {
        let uia = Uia::new().map_err(|e| e.to_string())?;
        probe::front(ctx, &doc.app)?;
        g21::click_field(ctx, &uia, &doc.app, &g.pass_name)?;
        judge(ctx, watch, &doc.app, Field::ChromePassword)
    })?;
    v["clean_up"] = json!(left);
    Ok(v)
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
