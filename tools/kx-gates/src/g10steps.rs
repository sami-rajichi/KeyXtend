//! G10's key steps on screen: each step's presses through UI Automation, then what target-window logged, whether it
//! kept the front, its focus and its layout, and whether a modifier read as latched and then let go.
#![cfg(windows)]

use kx_mod_keyboard::{KeyboardSettings, Modifiers, Nav, Role};
use kx_platform_windows::window::foreground;
use serde_json::{Value, json};
use spike_core::clock::now_us;

use crate::apps::{Ctx, Opened};
use crate::g10keys::{self, LangWant, Step};
use crate::keymap::Reader;
use crate::mouseguard;
use crate::product::{self, Keyboard};
use crate::win::{poll_until, sleep_ms};
use crate::{board, g25, langs, tlog};

/// Where G10 presses: the keyboard, target-window in front, and the layout target-window had at the start.
struct Desk<'a> {
    ctx: &'a Ctx<'a>,
    kb: &'a Keyboard,
    target: &'a Opened,
    reader: Reader,
    layout: Option<isize>,
}

/// What target-window shows after a step: the scan codes that went down, focus losses, and whether it stayed in front
/// with its first layout.
struct After {
    downs: Vec<u16>,
    lost: usize,
    front: bool,
    same_layout: bool,
}

impl After {
    /// True when no focus was lost and target-window stayed in front with its first layout.
    fn kept(&self) -> bool {
        self.lost == 0 && self.front && self.same_layout
    }

    /// The fields every step's result line carries.
    fn fields(&self) -> Value {
        let downs: Vec<String> = self.downs.iter().map(|d| format!("{d:04X}")).collect();
        json!({
            "downs": downs, "focus_lost": self.lost, "front": self.front, "same_layout": self.same_layout,
        })
    }
}

impl Desk<'_> {
    /// Presses board key `id` as a screen reader does, then waits G10's gap.
    fn press(&self, id: &str) -> Result<(), String> {
        mouseguard::allowed()?;
        self.restore(id)
    }

    /// Like `press`, but it still goes out after the mouse guard stopped the gate, to unlatch a key.
    fn restore(&self, id: &str) -> Result<(), String> {
        self.reader.restore(self.kb.hwnd, id)?;
        sleep_ms(self.ctx.cfg.g10.press_gap_ms);
        Ok(())
    }

    /// Whether `step`'s key reads as latched now; none when the step does not toggle or the state cannot be read.
    fn latched(&self, step: &Step) -> Result<Option<bool>, String> {
        if !step.toggles {
            return Ok(None);
        }
        self.reader.toggled(self.kb.hwnd, &step.key)
    }

    /// Whether latch key `id` reads as latched now; none when it cannot be read.
    fn on(&self, id: &str) -> Option<bool> {
        self.reader.toggled(self.kb.hwnd, id).ok().flatten()
    }

    /// What target-window shows once the step that started at `t0` (µs) has settled.
    fn after(&self, t0: i64) -> Result<After, String> {
        sleep_ms(self.ctx.cfg.g10.step_wait_ms);
        let path = &self.ctx.target.log;
        let log = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let downs = tlog::keys(&log).into_iter().filter(|k| k.us >= t0);
        Ok(After {
            downs: downs.map(|k| k.scan).collect(),
            lost: tlog::focus_lost(&log, t0),
            front: foreground() == self.target.hwnd,
            same_layout: langs::layout_of(self.target.hwnd) == self.layout,
        })
    }

    /// Waits until `f` holds, up to the shared read wait.
    fn until(&self, f: impl Fn() -> bool) -> bool {
        let t = &self.ctx.cfg.timing;
        poll_until(t.read_wait_ms, t.poll_ms, || f().then_some(())).is_some()
    }
}

/// Runs `step`: its result line, and whether its latch state could not be read.
fn one(d: &Desk, step: &Step) -> Result<(Value, bool), String> {
    if let Some(spare) = &step.spare {
        step.ready(d.reader.toggled(d.kb.hwnd, spare)?)?;
    }
    let t0 = now_us();
    let (first, rest) = step.presses.split_first().ok_or("a step presses nothing")?;
    d.press(first)?;
    let latched = d.latched(step)?;
    for (i, id) in rest.iter().enumerate() {
        // A last press that puts a mode back goes out even after the mouse guard stopped the gate.
        if step.puts_back && i + 1 == rest.len() {
            d.restore(id)?;
        } else {
            d.press(id)?;
        }
    }
    let still = d.latched(step)?;
    let after = d.after(t0)?;
    let keys_ok = step.expect.met(&after.downs);
    let unread = step.toggles && (latched.is_none() || still.is_none());
    let latch_ok = latched != Some(false) && still != Some(true);
    let mut v = after.fields();
    v["key"] = json!(step.key);
    v["keys_ok"] = json!(keys_ok);
    (v["latched"], v["let_go"]) = (json!(latched), json!(still.map(|on| !on)));
    v["ok"] = json!(keys_ok && latch_ok && after.kept());
    Ok((v, unread))
}

/// After a failed `step`, lets its latch go so the next step goes out alone: Win by `g10keys::undo` with its spare, any
/// other by one more press while it reads as latched; none of these sends a key, so they go out even after the mouse
/// guard's stop. The key's and the spare's states after, `None` when unread.
fn let_go(d: &Desk, step: &Step) -> (Option<bool>, Option<bool>) {
    if let Some(spare) = &step.spare {
        let keys = (step.key.as_str(), spare.as_str());
        return g10keys::undo(keys, |id| d.on(id), |id| d.restore(id).is_ok());
    }
    let on = d.on(&step.key);
    if !step.left_latched(on) || d.restore(&step.key).is_err() {
        return (on, None);
    }
    (d.on(&step.key), None)
}

/// Runs `step` and, when it failed, lets its latches go: its result line, with each key the owner checks by hand
/// added to `owner`.
fn checked(d: &Desk, step: &Step, owner: &mut Vec<Value>) -> Value {
    let (mut v, unread) = one(d, step)
        .unwrap_or_else(|e| (json!({ "key": step.key, "ok": false, "error": e }), false));
    let failed = v["ok"] != true;
    let (mut after, mut spare) = (None, None);
    if failed && step.toggles {
        (after, spare) = let_go(d, step);
        v["latched_after"] = json!(after);
        if step.spare.is_some() {
            v["spare_after"] = json!(spare);
        }
    }
    let why = step
        .owner_why(unread, failed, after)
        .map(|why| (step.key.as_str(), why));
    for (key, why) in why.into_iter().chain(step.spare_why(failed, spare)) {
        owner.push(json!({ "key": key, "why": why }));
    }
    v
}

/// Carousel or Cycle: target-window steps from its first layout to the next, which G10 then puts back.
fn stepped(d: &Desk) -> bool {
    let Some(start) = d.layout else {
        return false;
    };
    let next = langs::step_from(&langs::installed(), start, 1);
    let stepped = next.is_some_and(|n| g25::shows(d.ctx, d.target, n));
    let back = g25::switch_to(d.ctx, d.target, start);
    stepped && back
}

/// List: the list opens as one more product window, and the panel's Esc key closes it again, even after the mouse
/// guard stopped the gate.
fn listed(d: &Desk, before: usize) -> Result<bool, String> {
    let opened = d.until(|| product::shown().len() > before);
    d.restore(&board::nav_key(Nav::Esc)?)?;
    let closed = d.until(|| product::shown().len() <= before);
    Ok(opened && closed)
}

/// The language key's step, done last: it does what `settings` and the installed layouts say, and no key reaches
/// target-window.
fn lang(d: &Desk, settings: &KeyboardSettings) -> Result<Value, String> {
    let want = g10keys::lang_want(settings.lang_key, langs::installed().len());
    let key = board::id_of(Role::Lang)?;
    let (t0, before) = (now_us(), product::shown().len());
    d.press(&key)?;
    let did = match want {
        LangWant::Nothing => true,
        LangWant::Steps => stepped(d),
        LangWant::Opens => listed(d, before)?,
    };
    let after = d.after(t0)?;
    let mut v = after.fields();
    v["key"] = json!(key);
    v["want"] = json!(format!("{want:?}"));
    v["did"] = json!(did);
    v["ok"] = json!(did && after.downs.is_empty() && after.kept());
    Ok(v)
}

/// Presses every key but the character keys, in board order, then the language key: the steps, the keys
/// left to the owner's Narrator and NVDA check, and `ok`.
pub fn run(ctx: &Ctx, kb: &Keyboard, target: &Opened) -> Result<Value, String> {
    let settings = product::keyboard()?;
    let stay = settings.modifiers == Modifiers::Stay;
    let plan = g10keys::plan(&board::roles()?, &ctx.cfg.g10.probe, stay)?;
    let layout = langs::layout_of(target.hwnd);
    let reader = Reader::new()?;
    let d = Desk {
        ctx,
        kb,
        target,
        reader,
        layout,
    };
    let mut owner = Vec::new();
    let steps: Vec<Value> = plan.iter().map(|s| checked(&d, s, &mut owner)).collect();
    let lang = lang(&d, &settings).unwrap_or_else(|e| json!({ "ok": false, "error": e }));
    let ok = steps.iter().all(|s| s["ok"] == true) && lang["ok"] == true;
    Ok(json!({ "steps": steps, "lang": lang, "owner_checks": owner, "ok": ok }))
}
