//! Every end (edge 17): the keyboard comes back and the key goes dark after Copy, Save, Close, Cancel and a fault; one
//! snip runs at a time; Quit cancels a recording; new settings set the next snip.

use super::*;

/// The keyboard's last step and the key's last light in `outs`.
fn last(outs: &[Out]) -> (Option<SnipAside>, Option<SnipLit>) {
    let aside = outs.iter().rev().find_map(|o| match o {
        Out::Aside(a) => Some(*a),
        _ => None,
    });
    let lit = outs.iter().rev().find_map(|o| match o {
        Out::Lit(l) => Some(*l),
        _ => None,
    });
    (aside, lit)
}

/// A named path a test drives, giving the steps of its last input.
type Path = (&'static str, fn(&mut Rig) -> Vec<Out>);
/// A named place a test reaches, and whether the Snip key there leads to a new snip.
type Place = (&'static str, fn(&mut Rig), bool);

/// Each way a snip ends, from the Snip key on; each gives the steps of its end.
fn ends() -> Vec<Path> {
    vec![
        ("copy", |rig| {
            rig.picture();
            rig.on(In::Strip(StripKey::Copy));
            rig.on(In::Kept(Outcome::Done));
            rig.wait(rig.shown())
        }),
        ("save", |rig| {
            rig.picture();
            rig.on(In::Strip(StripKey::Save));
            rig.on(In::Kept(Outcome::Done));
            rig.wait(rig.shown())
        }),
        ("close", |rig| {
            rig.picture();
            rig.on(In::Strip(StripKey::Close));
            rig.on(In::Ask(AskKey::Drop))
        }),
        ("cancel", |rig| {
            rig.recording();
            rig.on(In::Record(RecKey::Cancel))
        }),
        ("fault", |rig| {
            let video = rig.recording();
            let note = RecordNote::Fault(RecordError::Gone);
            rig.on(In::Note { video, note })
        }),
        ("picking", |rig| {
            rig.picking();
            rig.on(In::Bar(BarKey::Close))
        }),
    ]
}

#[test]
fn edge_17_the_keyboard_comes_back_after_every_end() {
    for keyboard in [SnipKeyboard::Bubble, SnipKeyboard::Hidden] {
        for (name, end) in ends() {
            let mut rig = Rig::with(|s| s.keyboard = keyboard);
            let outs = end(&mut rig);
            assert!(outs.contains(&Out::End), "{name}");
            let back = (Some(SnipAside::Back), Some(SnipLit::Off));
            assert_eq!(last(&outs), back, "{name} {keyboard:?}");
            assert_eq!(rig.stage(), &Stage::Idle, "{name}");
        }
    }
}

#[test]
fn edge_17_the_keyboard_stays_back_after_the_editor() {
    let mut rig = Rig::new();
    rig.editor();
    rig.on(In::Edit(EditKey::Close));
    let outs = rig.on(In::Ask(AskKey::Drop));
    assert_eq!(outs, [Out::End]);
    assert_eq!(last(&outs), (None, None));
}

fn freezing(rig: &mut Rig) {
    rig.on(In::Key);
}

fn counting(rig: &mut Rig) {
    rig.ready();
    rig.on(In::Strip(StripKey::Record));
}

fn recording(rig: &mut Rig) {
    rig.recording();
}

fn stopped(rig: &mut Rig) {
    rig.stopped();
}

fn kept_editor(rig: &mut Rig) {
    rig.editor();
    rig.flat_kept(Keep::Copy);
}

fn sped(rig: &mut Rig) {
    rig.stopped();
    rig.on(In::Strip(StripKey::Speed));
}

/// Each place a snip can stand, reached from the Snip key, and whether the key there leads to a new snip.
fn places() -> Vec<Place> {
    vec![
        ("freezing", freezing, false),
        ("picking", Rig::picking, false),
        ("strip", Rig::picture, false),
        ("ready", Rig::ready, false),
        ("count", counting, false),
        ("recording", recording, false),
        ("video", stopped, false),
        ("editor", Rig::editor, true),
        ("kept editor", kept_editor, true),
        ("sped video", sped, false),
    ]
}

#[test]
fn one_snip_at_a_time() {
    for (name, place, starts) in places() {
        let mut rig = Rig::new();
        place(&mut rig);
        let mut outs = rig.on(In::Key);
        if rig.flow.asking() {
            outs.extend(rig.on(In::Ask(AskKey::Drop)));
        }
        let start = at(&outs, &Out::Lit(SnipLit::On));
        assert_eq!(start.is_some(), starts, "{name}: {outs:?}");
        if let Some(start) = start {
            let end = at(&outs, &Out::End);
            assert!(end.is_some_and(|end| end < start), "{name}: {outs:?}");
        }
    }
}

#[test]
fn quit_cancels_the_recording_and_ends() {
    let mut rig = Rig::new();
    let video = rig.recording();
    let outs = rig.on(In::Quit);
    let cancel = Out::Run {
        video,
        call: Call::Cancel,
    };
    assert_eq!(outs.first(), Some(&cancel));
    assert!(ended(&outs));
}

#[test]
fn quit_deletes_an_unkept_video() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    assert!(rig.on(In::Quit).contains(&Out::Delete(video)));
}

#[test]
fn quit_while_idle_does_nothing() {
    let mut rig = Rig::new();
    assert_eq!(rig.on(In::Quit), []);
}

#[test]
fn quit_keeps_a_video_whose_copy_is_in_flight() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(In::Strip(StripKey::Copy));
    let outs = rig.on(In::Quit);
    assert!(!outs.contains(&Out::Delete(video)), "{outs:?}");
    assert!(ended(&outs));
}

#[test]
fn quit_keeps_the_retimed_copy_in_flight_and_deletes_the_recording() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(In::Strip(StripKey::Speed));
    rig.on(In::Strip(StripKey::Copy));
    let outs = rig.on(In::Quit);
    assert_eq!(outs.first(), Some(&Out::Delete(video)));
    assert_eq!(
        outs.iter().filter(|o| matches!(o, Out::Delete(_))).count(),
        1
    );
    assert!(ended(&outs));
}

#[test]
fn quit_deletes_a_video_whose_save_is_in_flight() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(In::Strip(StripKey::Save));
    assert!(rig.on(In::Quit).contains(&Out::Delete(video)));
}

/// Settings unlike the defaults in every value a snip reads.
fn other() -> SnipSettings {
    let mut other = SnipSettings::from_defaults().unwrap();
    other.keyboard = SnipKeyboard::Hidden;
    other.countdown_s = 0;
    other.mic_on = true;
    other.sound_on = true;
    other
}

/// A video part taken, then Record; whether it records at once, and the strip's switches before.
fn shoot(rig: &mut Rig) -> (bool, Option<Held>) {
    rig.on(In::Bar(BarKey::VideoRegion));
    rig.take();
    let held = match rig.stage() {
        Stage::Strip(strip) => Some(strip.held.clone()),
        _ => None,
    };
    let now = recorded(&rig.on(In::Strip(StripKey::Record))).is_some();
    (now, held)
}

#[test]
fn new_settings_spare_the_running_snip() {
    let mut rig = Rig::new();
    let old = rig.settings.clone();
    rig.on(In::Key);
    rig.on(In::Settings(Box::new(other())));
    rig.on(In::Aside);
    rig.on(In::Frozen(Ok(frozen())));
    let (now, held) = shoot(&mut rig);
    let sounds = Sounds {
        mic: old.mic_on,
        sound: old.sound_on,
    };
    assert_eq!(held, Some(Held::Ready(sounds)));
    assert_eq!(now, old.countdown_s == 0);
    assert!(
        rig.on(In::Record(RecKey::Cancel))
            .contains(&Out::Aside(SnipAside::Back))
    );
}

#[test]
fn new_settings_set_the_next_snip() {
    let mut rig = Rig::new();
    rig.picking();
    rig.on(In::Settings(Box::new(other())));
    rig.on(In::Esc);
    let outs = rig.on(In::Key);
    assert!(outs.contains(&Out::Aside(SnipAside::Hidden)), "{outs:?}");
    rig.on(In::Aside);
    rig.on(In::Frozen(Ok(frozen())));
    let (now, held) = shoot(&mut rig);
    let both = Sounds {
        mic: true,
        sound: true,
    };
    assert_eq!(held, Some(Held::Ready(both)));
    assert!(now);
}

#[test]
fn new_settings_wait_for_the_editor_to_end_then_start_the_next_snip() {
    let mut rig = Rig::new();
    rig.editor();
    rig.on(In::Settings(Box::new(other())));
    rig.on(In::Key);
    let outs = rig.on(In::Ask(AskKey::Drop));
    assert!(outs.contains(&Out::Aside(SnipAside::Hidden)), "{outs:?}");
}
