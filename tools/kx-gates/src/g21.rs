//! G21: after a click into a field, the product's own field rules (`WinFieldWatch`) must tell a password box as one;
//! `g21 keyxtend` then checks the product's lane in those boxes (`g21prod`).
#![cfg(windows)]

use kx_mod_predict::PredictSettings;
use kx_module_api::ports::{FieldKind, FieldRun, FieldWatch};
use kx_platform_windows::WinFieldWatch;
use kx_platform_windows::uia::{self, Uia};
use serde_json::{Value, json};
use spike_core::geom;

use crate::apps::{self, Ctx, Opened};
use crate::{g21prod, launch, probe, product, simuser, typebox, win};

/// Run on a web page in Chrome.
const CHROME: &str = "chrome";
/// Run on target-window's Win32 edit box.
const WIN32: &str = "win32";
/// G21's runs: the two field-kind checks, then the product half.
pub const NAMES: [&str; 3] = [CHROME, WIN32, product::NAME];
/// The Win32 run's boxes: target-window plain, and with its password arguments.
const WIN32_BOXES: [(&str, bool); 2] = [("plain", false), ("password", true)];

/// A field kind's name in a result line.
pub(crate) fn kind_name(kind: FieldKind) -> &'static str {
    match kind {
        FieldKind::Text => "text",
        FieldKind::Password => "password",
        FieldKind::Unknown => "unknown",
        FieldKind::None => "none",
    }
}

/// The kind a box should be told as: a password box when `password`, else text.
pub(crate) fn wanted(password: bool) -> FieldKind {
    if password {
        FieldKind::Password
    } else {
        FieldKind::Text
    }
}

/// The kind `watch` tells once it is `want`, or the last it told within `[timing] read_wait_ms`.
pub(crate) fn told(ctx: &Ctx, watch: &dyn FieldRun, want: FieldKind) -> FieldKind {
    let t = &ctx.cfg.timing;
    let mut kind = watch.kind();
    win::poll_until(t.read_wait_ms, t.poll_ms, || {
        kind = watch.kind();
        (kind == want).then_some(())
    });
    kind
}

/// One field's result: it passes when the kind told is a password box exactly when one is expected.
fn field(name: &str, want: bool, got: Result<FieldKind, String>) -> Value {
    match got {
        Ok(kind) => {
            let p = kind == FieldKind::Password;
            json!({ "field": name, "kind": kind_name(kind), "password": p, "expected": want, "ok": p == want })
        }
        Err(e) => json!({ "field": name, "expected": want, "ok": false, "error": e }),
    }
}

/// A field watch with the product's rules, reading boxes as the predict defaults say.
fn watch() -> Result<Box<dyn FieldRun>, String> {
    let s = PredictSettings::from_defaults().map_err(|e| format!("the predict defaults: {e}"))?;
    let tell = Box::new(|_| {});
    WinFieldWatch
        .watch(s.field_config(), tell)
        .map_err(|e| e.to_string())
}

/// A web field called `name` of HTML type `kind`, with any `extra` attributes.
pub(crate) fn input(ctx: &Ctx, name: &str, kind: &str, extra: &str) -> String {
    let css = &ctx.cfg.g21.field_css;
    format!(
        "<input aria-label=\"{name}\" type=\"{kind}\" autocomplete=\"off\" style=\"{css}\" {extra}>"
    )
}

/// G21's Chrome page body: a text field, then a password field.
pub(crate) fn page(ctx: &Ctx) -> String {
    let g = &ctx.cfg.g21;
    input(ctx, &g.user_name, "text", "") + &input(ctx, &g.pass_name, "password", "")
}

/// Clicks the centre of the web field called `name`, once UI Automation shows it.
pub(crate) fn click_field(ctx: &Ctx, uia: &Uia, app: &Opened, name: &str) -> Result<(), String> {
    let t = &ctx.cfg.timing;
    let el = win::poll_until(t.read_wait_ms, t.poll_ms, || uia.named(app.hwnd, name))
        .ok_or_else(|| format!("no field {name} through UI Automation"))?;
    simuser::click(
        ctx,
        app,
        geom::centre(&uia::rect(&el).map_err(|e| e.to_string())?),
    )
}

/// Clicks the web field called `name`, a password box if `want`, and reads the kind `watch` tells once it has focus.
fn web_field(
    ctx: &Ctx,
    uia: &Uia,
    app: &Opened,
    watch: &dyn FieldRun,
    (name, want): (&str, bool),
) -> Result<FieldKind, String> {
    let t = &ctx.cfg.timing;
    click_field(ctx, uia, app, name)?;
    let has_focus = || uia.focused().ok().filter(|f| uia::name(f) == name);
    win::poll_until(t.read_wait_ms, t.poll_ms, has_focus)
        .ok_or_else(|| format!("{name} did not take the focus"))?;
    Ok(told(ctx, watch, wanted(want)))
}

/// Chrome on a page with a text field and a password field, with one field watch for both.
fn chrome(ctx: &Ctx) -> Result<Value, String> {
    let g = &ctx.cfg.g21;
    let doc = probe::open_page(ctx, &page(ctx))?;
    let (fields, notes) = probe::run_on(ctx, doc, |doc| {
        let (uia, watch) = (Uia::new().map_err(|e| e.to_string())?, watch()?);
        probe::front(ctx, &doc.app)?;
        let cases = [(g.user_name.as_str(), false), (g.pass_name.as_str(), true)];
        let one = |c: (&str, bool)| field(c.0, c.1, web_field(ctx, &uia, &doc.app, &*watch, c));
        Ok(cases.map(one))
    })?;
    Ok(result(CHROME, &fields, &notes))
}

/// Starts target-window, with a password box if `password`, clicks its box and reads the kind `watch` tells. Also
/// returns what clean-up left open.
fn win32_box(
    ctx: &Ctx,
    watch: &dyn FieldRun,
    password: bool,
) -> (Result<FieldKind, String>, Vec<String>) {
    let app = match launch::start_target_with(ctx, typebox::password_args(ctx, password)) {
        Ok(app) => app,
        Err(e) => return (Err(e), Vec::new()),
    };
    let got = probe::front(ctx, &app)
        .and_then(|()| typebox::focus_target(ctx, &app))
        .map(|()| told(ctx, watch, wanted(password)));
    (got, apps::clean_up(ctx, app))
}

/// target-window's box, plain and as a password box, with one field watch for both.
fn win32(ctx: &Ctx) -> Result<Value, String> {
    let watch = watch()?;
    let mut left = Vec::new();
    let fields = WIN32_BOXES.map(|(n, want)| {
        let (got, notes) = win32_box(ctx, &*watch, want);
        left.extend(notes);
        field(n, want, got)
    });
    Ok(result(WIN32, &fields, &left))
}

/// The run's JSON: it passes when every field's flag is as expected.
fn result(name: &str, fields: &[Value], clean_up: &[String]) -> Value {
    let pass = fields.iter().all(|f| f["ok"] == true);
    json!({ "gate": "G21", "app": name, "pass": pass, "fields": fields, "clean_up": clean_up })
}

/// Runs G21 on `name`.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    match name {
        CHROME => chrome(ctx),
        WIN32 => win32(ctx),
        product::NAME => g21prod::run(ctx),
        _ => Err(format!("G21 runs {}, not {name}", NAMES.join(", "))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every kind, once.
    const KINDS: [FieldKind; 4] = [
        FieldKind::Text,
        FieldKind::Password,
        FieldKind::Unknown,
        FieldKind::None,
    ];

    #[test]
    fn each_kind_has_its_own_name() {
        let names: std::collections::BTreeSet<_> = KINDS.into_iter().map(kind_name).collect();
        assert_eq!(names.len(), KINDS.len());
        assert_eq!(kind_name(FieldKind::Password), "password");
    }

    #[test]
    fn a_field_passes_only_when_told_as_a_password_box_exactly_when_expected() {
        let ok = |want, kind| field("f", want, Ok(kind))["ok"] == true;
        assert!(ok(true, FieldKind::Password));
        for kind in [FieldKind::Text, FieldKind::Unknown, FieldKind::None] {
            assert!(!ok(true, kind), "{kind:?}");
            assert!(ok(false, kind), "{kind:?} is no password box");
        }
        assert!(!ok(false, FieldKind::Password));
        let line = field("f", true, Ok(FieldKind::Unknown));
        assert_eq!(line["kind"], "unknown");
        assert_eq!(field("f", true, Err("gone".into()))["ok"], false);
    }

    #[test]
    fn a_password_box_is_wanted_as_one_and_a_plain_box_as_text() {
        assert_eq!(wanted(true), FieldKind::Password);
        assert_eq!(wanted(false), FieldKind::Text);
    }
}
