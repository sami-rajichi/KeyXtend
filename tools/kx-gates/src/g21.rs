//! G21: after a click into a field, the focused element says whether it is a password box.
#![cfg(windows)]

use serde_json::{Value, json};
use spike_core::uia::{self, Uia};

use crate::apps::{self, AppKind, Ctx, Opened};
use crate::{launch, probe, simuser, win};

/// Run on a web page in Chrome.
const CHROME: &str = "chrome";
/// Run on target-window's Win32 edit box.
const WIN32: &str = "win32";
/// G21's two runs.
pub const NAMES: [&str; 2] = [CHROME, WIN32];
/// The Win32 run's boxes: target-window plain, and with its password arguments.
const WIN32_BOXES: [(&str, bool); 2] = [("plain", false), ("password", true)];

/// One field's result.
fn field(name: &str, want: bool, got: Result<bool, String>) -> Value {
    match got {
        Ok(p) => json!({ "field": name, "password": p, "expected": want, "ok": p == want }),
        Err(e) => json!({ "field": name, "expected": want, "ok": false, "error": e }),
    }
}

/// A web field called `name` of HTML type `kind`, with any `extra` attributes.
pub(crate) fn input(ctx: &Ctx, name: &str, kind: &str, extra: &str) -> String {
    let css = &ctx.cfg.g21.field_css;
    format!(
        "<input aria-label=\"{name}\" type=\"{kind}\" autocomplete=\"off\" style=\"{css}\" {extra}>"
    )
}

/// Clicks the centre of the web field called `name`, once UI Automation shows it.
pub(crate) fn click_field(ctx: &Ctx, uia: &Uia, app: &Opened, name: &str) -> Result<(), String> {
    let t = &ctx.cfg.timing;
    let el = win::poll_until(t.read_wait_ms, t.poll_ms, || uia.named(app.hwnd, name))
        .ok_or_else(|| format!("no field {name} through UI Automation"))?;
    simuser::click(ctx, app, uia::centre(&uia::rect(&el)?))
}

/// Clicks the web field called `name` and reads the password flag of the element that took the focus.
fn web_field(ctx: &Ctx, uia: &Uia, app: &Opened, name: &str) -> Result<bool, String> {
    let t = &ctx.cfg.timing;
    click_field(ctx, uia, app, name)?;
    let has_focus = || uia.focused().ok().filter(|f| uia::name(f) == name);
    let focused = win::poll_until(t.read_wait_ms, t.poll_ms, has_focus)
        .ok_or_else(|| format!("{name} did not take the focus"))?;
    Ok(uia::is_password(&focused))
}

/// Chrome on a page with a text field and a password field.
fn chrome(ctx: &Ctx) -> Result<Value, String> {
    let g = &ctx.cfg.g21;
    let body = input(ctx, &g.user_name, "text", "") + &input(ctx, &g.pass_name, "password", "");
    let doc = probe::open_page(ctx, &body)?;
    let (fields, notes) = probe::run_on(ctx, doc, |doc| {
        let uia = Uia::new()?;
        probe::front(ctx, &doc.app)?;
        let cases = [(g.user_name.as_str(), false), (g.pass_name.as_str(), true)];
        Ok(cases.map(|(n, want)| field(n, want, web_field(ctx, &uia, &doc.app, n))))
    })?;
    Ok(result(CHROME, &fields, notes))
}

/// Starts target-window, with a password box if `password`, clicks its box and reads the focused element's flag.
/// Also returns what clean-up left open.
fn win32_box(ctx: &Ctx, password: bool) -> (Result<bool, String>, Vec<String>) {
    let extra = if password {
        std::slice::from_ref(&ctx.target.password_arg)
    } else {
        &[]
    };
    let app = match launch::start_target_with(ctx, extra) {
        Ok(app) => app,
        Err(e) => return (Err(e), Vec::new()),
    };
    let got = (|| {
        let uia = Uia::new()?;
        probe::front(ctx, &app)?;
        simuser::click(ctx, &app, uia::centre(&win::rect(app.hwnd)?))?;
        let edit = ctx
            .cfg
            .app(AppKind::Target.name())?
            .focus_class
            .clone()
            .unwrap_or_default();
        if win::class(win::focus(app.hwnd)) != edit {
            return Err(format!("the {edit} box did not take the focus"));
        }
        Ok(uia::is_password(&uia.focused()?))
    })();
    (got, apps::clean_up(ctx, app))
}

/// target-window's box, plain and as a password box.
fn win32(ctx: &Ctx) -> Value {
    let mut left = Vec::new();
    let fields = WIN32_BOXES.map(|(n, want)| {
        let (got, notes) = win32_box(ctx, want);
        left.extend(notes);
        field(n, want, got)
    });
    result(WIN32, &fields, left)
}

/// The run's JSON: it passes when every field's flag is as expected.
fn result(name: &str, fields: &[Value], clean_up: Vec<String>) -> Value {
    let pass = fields.iter().all(|f| f["ok"] == true);
    json!({ "gate": "G21", "app": name, "pass": pass, "fields": fields, "clean_up": clean_up })
}

/// Runs G21 on `name`.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    match name {
        CHROME => chrome(ctx),
        WIN32 => Ok(win32(ctx)),
        _ => Err(format!("G21 runs {}, not {name}", NAMES.join(", "))),
    }
}
