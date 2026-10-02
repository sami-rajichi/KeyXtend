//! G10's key steps on screen: each step's presses through UI Automation, then what target-window logged, whether it
//! kept the front, its focus and its layout, and whether a modifier read as latched and then let go.
#![cfg(windows)]

use kx_mod_keyboard::{KeyboardSettings, Modifiers, Nav, Role};
use kx_platform_windows::window::foreground;
use serde_json::{Value, json};
use spike_core::clock::now_us;
use spike_core::langkey;

use crate::apps::{Ctx, Opened};
use crate::g10keys::{self, LangWant, Step};
use crate::keymap::Reader;
use crate::product::{self, Keyboard};
use crate::win::{poll_until, sleep_ms};
use crate::{board, g25, tlog};

/// Why a key is left to the owner's Narrator and NVDA check.
const NO_TOGGLE: &str = "its latch shows no toggle state through UI Automation";

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
        self.reader.invoke(self.kb.hwnd, id)?;
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
            same_layout: langkey::layout_of(self.target.hwnd) == self.layout,
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
    let t0 = now_us();
    let (first, rest) = step.presses.split_first().ok_or("a step presses nothing")?;
    d.press(first)?;
    let latched = d.latched(step)?;
    for id in rest {
        d.press(id)?;
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

/// Carousel or Cycle: target-window steps from its first layout to the next, which G10 then puts back.
fn stepped(d: &Desk) -> bool {
    let Some(start) = d.layout else {
        return false;
    };
    let next = langkey::step_from(&langkey::installed(), start, 1);
    let stepped = next.is_some_and(|n| g25::shows(d.ctx, d.target, n));
    let back = g25::switch_to(d.ctx, d.target, start);
    stepped && back
}

/// List: the list opens as one more product window, and the panel's Esc key closes it again.
fn listed(d: &Desk, before: usize) -> Result<bool, String> {
    let opened = d.until(|| product::shown().len() > before);
    d.press(&board::nav_key(Nav::Esc)?)?;
    let closed = d.until(|| product::shown().len() <= before);
    Ok(opened && closed)
}

/// The language key's step, done last: it does what `settings` and the installed layouts say, and no key reaches
/// target-window.
fn lang(d: &Desk, settings: &KeyboardSettings) -> Result<Value, String> {
    let want = g10keys::lang_want(settings.lang_key, langkey::installed().len());
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

/// Presses every key that types no character at once, in board order, then the language key: the steps, the keys
/// left to the owner's Narrator and NVDA check, and `ok`.
pub fn run(ctx: &Ctx, kb: &Keyboard, target: &Opened) -> Result<Value, String> {
    let settings = product::keyboard()?;
    let stay = settings.modifiers == Modifiers::Stay;
    let plan = g10keys::plan(&board::roles()?, &ctx.cfg.g10.probe, stay)?;
    let layout = langkey::layout_of(target.hwnd);
    let reader = Reader::new()?;
    let d = Desk {
        ctx,
        kb,
        target,
        reader,
        layout,
    };
    let (mut steps, mut owner) = (Vec::new(), Vec::new());
    for step in &plan {
        let (v, unread) = one(&d, step)
            .unwrap_or_else(|e| (json!({ "key": step.key, "ok": false, "error": e }), false));
        if unread {
            owner.push(json!({ "key": step.key, "why": NO_TOGGLE }));
        }
        steps.push(v);
    }
    let lang = lang(&d, &settings).unwrap_or_else(|e| json!({ "ok": false, "error": e }));
    let ok = steps.iter().all(|s| s["ok"] == true) && lang["ok"] == true;
    Ok(json!({ "steps": steps, "lang": lang, "owner_checks": owner, "ok": ok }))
}
