//! Snip over the whole app (ADR-0056): the boot makes the Snips folder and its unsaved part, a hidden run neither
//! (ADR-0037); the Snip key starts a snip whose view, step aside and light the ui hears; a hidden run dims Video and
//! Save while Copy and Edit work (edge 21); the recorder the app registers opens only its own seal's passes; and Esc
//! ends a snip over its frozen screens (ADR-0059).

mod esc;
mod seal;

use crate::rig::snip::SCREEN;
use crate::rig::{APP, Rig, WAIT, Wakes, hear, watch};
use keyxtend::boot::Boot;
use keyxtend::hidden::Run;
use keyxtend::snip::{SNIP_DIR, UNSAVED_DIR};
use kx_module_api::Subscription;
use kx_module_api::ports::{
    BarKey, Dim, FaceAction, KeyId, SnipAside, SnipLit, SnipStage, SnipView, StripKey,
};
use kx_platform::PORTABLE_DIR;
use kx_platform_fake::WriteKind;
use kx_ui::Wake;
use kx_ui::i18n::keys::{snip_bar_id, snip_dim_id, snip_strip_id};
use kx_ui::snip_acts::{BAR, STRIP};
use serde_json::Value;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

/// The keyboard's Snip key.
const SNIP_KEY: &str = "snip";

/// The ui's view as JSON.
fn view(boot: &Boot) -> Value {
    serde_json::from_str(&boot.hub.snip_json()).unwrap()
}

/// Each key's id with why it dims, in `keys`' order.
fn dims(keys: &Value) -> Vec<(String, Option<String>)> {
    let key = |k: &Value| {
        (
            k["key"].as_str().unwrap().into(),
            k["dim"].as_str().map(Into::into),
        )
    };
    keys.as_array().unwrap().iter().map(key).collect()
}

/// The Snip key's light in the face's state, which QML reads at `Wake::Keys`.
fn snip_lit(boot: &Boot) -> Value {
    let state: Value = serde_json::from_str(&boot.hub.state_json()).unwrap();
    state["keys"][SNIP_KEY]["on"].clone()
}

/// Waits until a view `is` heard; the ui heard it first.
fn until(views: &Receiver<SnipView>, is: fn(&SnipStage) -> bool) {
    while !is(&views.recv_timeout(WAIT).unwrap().0.stage) {}
}

/// Clicks the key `id` of kind `kind` on the ui's newest view.
fn click(boot: &Boot, kind: &str, id: &str) {
    let round = view(boot)["round"].as_i64().unwrap();
    boot.hub.snip_act(kind, id, 0, round);
}

/// Taps the keyboard's Snip key.
fn tap(boot: &Boot) {
    let tap = FaceAction::Tap {
        id: KeyId::new(SNIP_KEY),
        back: false,
    };
    let _ = boot.hub.act(tap);
}

/// Starts a snip on a started `boot` and answers its step aside, as Qt does; it gives the views heard from then on
/// while the subscription lives.
fn snip(boot: &Boot) -> (Subscription, Receiver<SnipView>) {
    let bus = boot.kernel.bus();
    let ((_aside, asides), (ear, views)) = (hear::<SnipAside>(&bus), hear::<SnipView>(&bus));
    tap(boot);
    assert_eq!(asides.recv_timeout(WAIT).unwrap(), SnipAside::Bubble);
    let mode: Value = serde_json::from_str(&boot.hub.mode_json()).unwrap();
    assert_eq!(mode["aside"], "bubble", "the ui heard the step aside");
    boot.hub.aside_shown(true);
    until(&views, |s| matches!(s, SnipStage::Pick(_)));
    (ear, views)
}

/// Takes the whole screen in the snip whose views come on `views`, as a click on Full, then on the screen, does.
fn take(boot: &Boot, views: &Receiver<SnipView>) {
    click(boot, BAR, snip_bar_id(BarKey::Full));
    until(
        views,
        |s| matches!(s, SnipStage::Pick(p) if p.keys.iter().any(|k| k.key == BarKey::Full && k.on)),
    );
    let round = view(boot)["round"].as_i64().unwrap();
    boot.hub.snip_spot(0, 1.0, 1.0, round);
    until(views, |s| matches!(s, SnipStage::Strip(_)));
}

/// Each light heard, with how many wakes the ui had given then.
type Lights = Arc<Mutex<Vec<(SnipLit, usize)>>>;

/// Keeps each `SnipLit` heard on `boot`'s bus with how many wakes `wakes` held then.
fn lights(boot: &Boot, wakes: &Wakes) -> (Subscription, Lights) {
    let (seen, wakes) = (Lights::default(), Arc::clone(wakes));
    let kept = Arc::clone(&seen);
    let ear = boot.kernel.bus().subscribe(move |lit: &SnipLit| {
        let at = wakes.lock().unwrap().len();
        kept.lock().unwrap().push((*lit, at));
    });
    (ear.unwrap(), seen)
}

/// Where the wakes stood when `lit` was first heard in `seen`.
fn at(seen: &Mutex<Vec<(SnipLit, usize)>>, lit: SnipLit) -> usize {
    let seen = seen.lock().unwrap();
    seen.iter().find(|(l, _)| *l == lit).unwrap().1
}

#[test]
fn the_boot_makes_the_snips_folder_and_its_unsaved_part_and_a_hidden_run_neither() {
    let rig = Rig::new();
    let boot = rig.load();
    let snips = rig.exe.path().join(PORTABLE_DIR).join(SNIP_DIR);
    assert!(snips.join(UNSAVED_DIR).is_dir(), "recordings have a place");
    assert_eq!(boot.no_snip_dir, None);
    let rig = Rig::new();
    let boot = rig.load_as(&Run::hidden(APP.into()));
    assert!(!rig.exe.path().join(PORTABLE_DIR).join(SNIP_DIR).exists());
    assert_eq!(boot.no_snip_dir, None, "nothing was tried");
}

#[test]
fn the_snip_key_starts_a_snip_whose_view_step_aside_and_light_the_ui_hears() {
    let rig = Rig::new();
    let mut boot = rig.load();
    let wakes = watch(&boot.hub);
    let (_before, before) = lights(&boot, &wakes);
    boot.start().unwrap();
    // Handlers run in subscription order: the ui's light handler, made at the start, runs between these two.
    let (_after, after) = lights(&boot, &wakes);
    let (_ear, views) = snip(&boot);
    let shown = view(&boot);
    assert_eq!(
        (
            shown["stage"].as_str(),
            shown["frozen"][0]["rect"]["w"].as_i64()
        ),
        (Some("pick"), Some(SCREEN.w.into()))
    );
    assert_eq!(snip_lit(&boot), Value::Bool(true), "the key is lit");
    let heard = at(&before, SnipLit::On)..at(&after, SnipLit::On);
    // A copy, so the lock is free before the actor's next light.
    let woke = wakes.lock().unwrap()[heard].to_vec();
    assert!(
        woke.contains(&Wake::Keys),
        "the ui heard the light: {woke:?}"
    );
    let bus = boot.kernel.bus();
    let ((_aside, asides), (_lit, lits)) = (hear::<SnipAside>(&bus), hear::<SnipLit>(&bus));
    click(&boot, BAR, snip_bar_id(BarKey::Close));
    until(&views, |s| matches!(s, SnipStage::Idle));
    assert_eq!(view(&boot)["stage"], "idle");
    assert_eq!(asides.recv_timeout(WAIT).unwrap(), SnipAside::Back);
    let mode: Value = serde_json::from_str(&boot.hub.mode_json()).unwrap();
    assert_eq!(mode["aside"], Value::Null, "the keyboard is back");
    assert_eq!(lits.recv_timeout(WAIT).unwrap(), SnipLit::Off);
    assert_eq!(snip_lit(&boot), Value::Bool(false), "and its key dark");
    boot.kernel.stop_all();
}

/// A hidden run's app, started, in a snip at its Pick view, with the views heard from then on.
fn hidden_snip(rig: &Rig) -> (Boot, Subscription, Receiver<SnipView>) {
    let mut boot = rig.load_as(&Run::hidden(APP.into()));
    boot.start().unwrap();
    let (ear, views) = snip(&boot);
    (boot, ear, views)
}

/// Why a key dims in a hidden run, as the ui names it.
fn hidden_dim() -> String {
    snip_dim_id(Dim::Hidden).to_owned()
}

#[test]
fn a_hidden_run_dims_video_and_save_while_copy_works() {
    let rig = Rig::new();
    let (mut boot, _ear, views) = hidden_snip(&rig);
    let video = [BarKey::VideoRegion, BarKey::VideoFull].map(|k| snip_bar_id(k).to_owned());
    for (key, dim) in dims(&view(&boot)["pick"]["keys"]) {
        let want = video.contains(&key).then(hidden_dim);
        assert_eq!(dim, want, "{key}");
    }
    take(&boot, &views);
    let strip = dims(&view(&boot)["strip"]["keys"]);
    let want = [
        (StripKey::Copy, None),
        (StripKey::Edit, None),
        (StripKey::Save, Some(hidden_dim())),
    ];
    for (key, dim) in want {
        assert!(
            strip.contains(&(snip_strip_id(key).to_owned(), dim)),
            "{key:?}: {strip:?}"
        );
    }
    click(&boot, STRIP, snip_strip_id(StripKey::Save));
    click(&boot, STRIP, snip_strip_id(StripKey::Copy));
    assert!(rig.writer.wait_tries(1, WAIT), "Copy works");
    assert_eq!(
        rig.writer.kinds(),
        [WriteKind::Kept],
        "both histories keep it"
    );
    until(
        &views,
        |s| matches!(s, SnipStage::Strip(t) if t.done.is_some()),
    );
    assert!(
        rig.snip.hidden.dialogs.saves().is_empty(),
        "Save asked nothing"
    );
    boot.kernel.stop_all();
    assert!(!rig.exe.path().join(PORTABLE_DIR).join(SNIP_DIR).exists());
}

#[test]
fn a_hidden_runs_edit_works_with_save_dimmed() {
    let rig = Rig::new();
    let (mut boot, _ear, views) = hidden_snip(&rig);
    take(&boot, &views);
    click(&boot, STRIP, snip_strip_id(StripKey::Edit));
    until(&views, |s| matches!(s, SnipStage::Edit(_)));
    let save = view(&boot)["edit"]["save_dim"].as_str().map(str::to_owned);
    assert_eq!(save, Some(hidden_dim()), "the editor's Save dims too");
    boot.kernel.stop_all();
}
