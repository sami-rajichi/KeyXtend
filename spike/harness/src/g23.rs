//! G23: Fill asks Windows Hello from the no-focus face; it types after Verified, and nothing after Cancel.

use std::time::Instant;

use serde_json::{Value, json};
use spike_core::config::ToolButton;
use spike_core::hello;
use spike_core::uia::{self, Uia};
use spike_core::window::foreground;

use crate::apps::{Ctx, Opened};
use crate::facetools::FaceWin;
use crate::win::{self, sleep_ms};
use crate::{g21, probe};

/// The login page: a user field, and a password field whose length the page keeps in its title, from 0 at load.
fn page(ctx: &Ctx) -> String {
    let (g, mark) = (&ctx.cfg.g21, &ctx.cfg.g23.length_mark);
    let len = format!(
        "oninput=\"document.title=document.title.split('{mark}')[0]+'{mark}'+this.value.length\""
    );
    let user = g21::input(ctx, &g.user_name, "text", "");
    let pass = g21::input(ctx, &g.pass_name, "password", &len);
    format!("{user}{pass}<script>document.title+='{mark}0'</script>")
}

/// The password length from the window title, which Chrome ends with its own name; `None` without the mark.
fn length_in(title: &str, mark: &str) -> Option<usize> {
    let (_, rest) = title.rsplit_once(mark)?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// True while a window of a Hello prompt program shows.
fn prompt_up(ctx: &Ctx) -> bool {
    let progs = &ctx.cfg.g23.prompt_programs;
    win::seen_windows()
        .into_iter()
        .filter_map(win::program_name)
        .any(|p| progs.iter().any(|q| q.eq_ignore_ascii_case(&p)))
}

/// Waits up to the Hello wait for `f`; returns the seconds it took.
fn within(ctx: &Ctx, f: impl Fn() -> bool) -> Option<f64> {
    let t0 = Instant::now();
    let wait = ctx.cfg.g23.hello_wait_ms;
    win::poll_until(wait, ctx.cfg.timing.poll_ms, || {
        f().then(|| t0.elapsed().as_secs_f64())
    })
}

/// Clicks the web field called `name`, then the face's `button`.
fn ask(
    ctx: &Ctx,
    uia: &Uia,
    fw: &FaceWin,
    app: &Opened,
    name: &str,
    button: ToolButton,
) -> Result<(), String> {
    g21::click_field(ctx, uia, app, name)?;
    fw.press(ctx, button)
}

/// The owner confirms: the test user name must arrive in the field in front.
fn confirmed(ctx: &Ctx, uia: &Uia, fw: &FaceWin, app: &Opened) -> Result<Value, String> {
    let (g, want) = (&ctx.cfg.g21, &ctx.spike.tools.test_user);
    ask(ctx, uia, fw, app, &g.user_name, ToolButton::FillUser)?;
    let prompt = within(ctx, || prompt_up(ctx));
    let typed = within(ctx, || {
        let el = uia.named(app.hwnd, &g.user_name);
        el.and_then(|e| uia::value(&e)).as_deref() == Some(want.as_str())
    });
    let back = foreground() == app.hwnd;
    let ok = prompt.is_some() && typed.is_some() && back;
    Ok(json!({ "ok": ok, "prompt_secs": prompt, "typed_secs": typed, "front_back": back }))
}

/// The owner cancels: the password field must stay empty.
fn cancelled(ctx: &Ctx, uia: &Uia, fw: &FaceWin, app: &Opened) -> Result<Value, String> {
    let mark = &ctx.cfg.g23.length_mark;
    ask(
        ctx,
        uia,
        fw,
        app,
        &ctx.cfg.g21.pass_name,
        ToolButton::FillPassword,
    )?;
    let prompt = within(ctx, || prompt_up(ctx));
    let closed = prompt.and_then(|_| within(ctx, || !prompt_up(ctx)));
    sleep_ms(
        ctx.spike
            .tools
            .fill_settle_ms
            .saturating_add(ctx.cfg.probes.settle_ms),
    );
    let len = length_in(&win::title(app.hwnd), mark);
    let ok = prompt.is_some() && closed.is_some() && len == Some(0);
    Ok(json!({ "ok": ok, "prompt_secs": prompt, "closed_secs": closed, "password_length": len }))
}

/// True if a Hello prompt shows within the refuse wait.
fn prompt_within_refuse(ctx: &Ctx) -> bool {
    let (wait, poll) = (ctx.cfg.g23.refuse_wait_ms, ctx.cfg.timing.poll_ms);
    win::poll_until(wait, poll, || prompt_up(ctx).then_some(())).is_some()
}

/// Hello is not set up: neither Fill may type anything, and no prompt may show.
fn refused(ctx: &Ctx, uia: &Uia, fw: &FaceWin, app: &Opened) -> Result<Value, String> {
    let g = &ctx.cfg.g21;
    ask(ctx, uia, fw, app, &g.user_name, ToolButton::FillUser)?;
    let prompt_user = prompt_within_refuse(ctx);
    let user = uia
        .named(app.hwnd, &g.user_name)
        .and_then(|e| uia::value(&e));
    ask(ctx, uia, fw, app, &g.pass_name, ToolButton::FillPassword)?;
    let prompt_pass = prompt_within_refuse(ctx);
    let len = length_in(&win::title(app.hwnd), &ctx.cfg.g23.length_mark);
    let user_chars = user.map(|u| u.chars().count());
    let ok = !prompt_user && !prompt_pass && user_chars == Some(0) && len == Some(0);
    Ok(json!({
        "ok": ok, "prompt_seen": prompt_user || prompt_pass,
        "user_chars": user_chars, "password_length": len,
    }))
}

/// With Hello: the owner confirms once, then cancels once. Without it: nothing may be typed.
fn drive(ctx: &Ctx, fw: &FaceWin, app: &Opened) -> Result<Value, String> {
    let uia = Uia::new()?;
    probe::front(ctx, app)?;
    if !hello::available()? {
        let r = refused(ctx, &uia, fw, app)?;
        return Ok(json!({ "hello": "unavailable", "pass": r["ok"], "refused": r }));
    }
    let yes = confirmed(ctx, &uia, fw, app)?;
    let no = cancelled(ctx, &uia, fw, app)?;
    let pass = yes["ok"] == true && no["ok"] == true;
    Ok(json!({ "hello": "available", "pass": pass, "confirmed": yes, "cancelled": no }))
}

/// Runs G23 with face `name` on a local Chrome login page.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let fw = FaceWin::parked(ctx, name)?;
    let doc = probe::open_page(ctx, &page(ctx))?;
    let (mut v, notes) = probe::run_on(ctx, doc, |doc| drive(ctx, &fw, &doc.app))?;
    v["gate"] = json!("G23");
    v["face"] = json!(name);
    v["clean_up"] = json!(notes);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_password_length_comes_from_the_title() {
        assert_eq!(length_in("KeyXtend probe 1 #15", " #"), Some(15));
        assert_eq!(
            length_in("KeyXtend probe 1 #0 - Google Chrome", " #"),
            Some(0)
        );
        assert_eq!(
            length_in("KeyXtend probe 1 - Google Chrome", " #"),
            None,
            "no mark means the page's counter never ran"
        );
        assert_eq!(length_in("KeyXtend probe 1 #x", " #"), None);
    }
}
