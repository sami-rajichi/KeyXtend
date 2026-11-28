//! The video strip, the countdown and recording: the switches, a sound that fails turning off with a note,
//! `countdown_s` on the clock, Cancel during the countdown making no file (edge 10), Stop or the Snip key keeping the
//! video, Cancel deleting it with no question.

use super::*;

/// The countdown's stage with `left_s` seconds left of the default video.
fn left(rig: &Rig) -> Option<u8> {
    match rig.stage() {
        Stage::Count(count) => Some(count.left_s),
        _ => None,
    }
}

/// The recording's stage.
fn rec(rig: &Rig) -> Rec {
    match rig.stage() {
        Stage::Record(rec) => rec.clone(),
        other => panic!("not recording: {other:?}"),
    }
}

#[test]
fn mic_and_sound_switch_on_the_strip() {
    let mut rig = Rig::new();
    rig.ready();
    rig.on(In::Strip(StripKey::Mic));
    rig.on(In::Strip(StripKey::Sound));
    rig.on(In::Strip(StripKey::Sound));
    let sounds = Sounds {
        mic: true,
        sound: false,
    };
    let Stage::Strip(strip) = rig.stage() else {
        panic!("no strip");
    };
    assert_eq!(strip.held, Held::Ready(sounds));
}

#[test]
fn record_counts_down_on_the_live_screen() {
    let mut rig = Rig::new();
    let count = rig.settings.countdown_s;
    rig.ready();
    assert_eq!(rig.on(In::Strip(StripKey::Record)), []);
    assert_eq!(left(&rig), Some(count));
    assert_eq!(rig.flow.frozen(), None);
    assert_eq!(rig.flow.deadline(), Some(rig.now().saturating_add(SECOND)));
    assert_eq!(rig.wait(SECOND.saturating_sub(MS)), []);
    assert_eq!(left(&rig), Some(count));
    assert_eq!(rig.wait(MS), []);
    assert_eq!(left(&rig), Some(count - 1));
    for left_s in (1..count - 1).rev() {
        assert_eq!(rig.wait(SECOND), []);
        assert_eq!(left(&rig), Some(left_s));
    }
    let outs = rig.wait(SECOND);
    let video = recorded(&outs).unwrap();
    let start = Out::Record {
        video,
        part: part(),
        mic: false,
        sound: false,
    };
    assert_eq!(outs, [start]);
    assert_eq!(rec(&rig).video, video);
    assert_eq!(rig.flow.deadline(), None);
}

#[test]
fn a_late_tick_counts_every_second_passed() {
    let mut rig = Rig::new();
    rig.ready();
    rig.on(In::Strip(StripKey::Record));
    let late = rig.countdown() + SECOND;
    assert!(recorded(&rig.wait(late)).is_some());
}

#[test]
fn the_switches_go_with_the_recording() {
    let mut rig = Rig::with(|s| s.countdown_s = 0);
    rig.ready();
    rig.on(In::Strip(StripKey::Sound));
    let outs = rig.on(In::Strip(StripKey::Record));
    let video = recorded(&outs).unwrap();
    let start = Out::Record {
        video,
        part: part(),
        mic: false,
        sound: true,
    };
    assert_eq!(outs, [start]);
}

#[test]
fn edge_10_cancel_during_the_countdown_makes_no_file() {
    for input in [In::Record(RecKey::Cancel), In::Key, In::Esc] {
        let mut rig = Rig::new();
        rig.ready();
        rig.on(In::Strip(StripKey::Record));
        rig.wait(SECOND);
        let outs = rig.on(input.clone());
        assert!(ended(&outs), "{input:?}");
        assert!(!kept_any(&outs), "{input:?}");
        assert_eq!(rig.flow.deadline(), None);
        let past = rig.countdown() + SECOND;
        assert_eq!(rig.wait(past), [], "{input:?}");
    }
}

#[test]
fn time_notes_show_the_time() {
    let mut rig = Rig::new();
    let video = rig.recording();
    assert!(rec(&rig).live);
    let note = RecordNote::Time { ms: 2_000 };
    assert_eq!(rig.on(In::Note { video, note }), []);
    assert_eq!(rec(&rig).ms, 2_000);
}

#[test]
fn switches_go_to_the_run() {
    let mut rig = Rig::new();
    let video = rig.recording();
    let mic = Out::Run {
        video,
        call: Call::Mic(true),
    };
    assert_eq!(rig.on(In::Record(RecKey::Mic)), [mic]);
    let sound = Out::Run {
        video,
        call: Call::Sound(true),
    };
    assert_eq!(rig.on(In::Record(RecKey::Sound)), [sound]);
    let both = Sounds {
        mic: true,
        sound: true,
    };
    assert_eq!(rec(&rig).film.sounds, both);
}

#[test]
fn a_lost_sound_switches_off_with_a_note() {
    for (source, note, (mic, sound)) in [
        (Source::Mic, SnipNote::MicFailed, (false, true)),
        (Source::Sound, SnipNote::SoundFailed, (true, false)),
    ] {
        let mut rig = Rig::with(|s| (s.mic_on, s.sound_on) = (true, true));
        let video = rig.recording();
        let lost = RecordNote::Lost(source);
        assert_eq!(rig.on(In::Note { video, note: lost }), [Out::Note(note)]);
        assert_eq!(rec(&rig).film.sounds, Sounds { mic, sound });
    }
}

/// Both sounds on, Record pressed and counted down, with no note heard yet.
fn starting_with_sounds() -> (Rig, Video) {
    let mut rig = Rig::with(|s| (s.mic_on, s.sound_on) = (true, true));
    rig.ready();
    let mut outs = rig.on(In::Strip(StripKey::Record));
    outs.extend(rig.wait(rig.countdown()));
    let video = recorded(&outs).unwrap();
    (rig, video)
}

#[test]
fn a_listed_device_failing_at_the_start_shows_the_strip_again_with_its_switch_off() {
    for (error, note, sounds) in [
        (RecordError::NoMic, SnipNote::MicFailed, (false, true)),
        (RecordError::MicRefused, SnipNote::MicFailed, (false, true)),
        (RecordError::NoSound, SnipNote::SoundFailed, (true, false)),
    ] {
        let (mut rig, video) = starting_with_sounds();
        let fault = RecordNote::Fault(error);
        assert_eq!(
            rig.on(In::Note { video, note: fault }),
            [Out::Note(note)],
            "{error:?}"
        );
        let (mic, sound) = sounds;
        let strip = Strip {
            part: part(),
            home: home(),
            held: Held::Ready(Sounds { mic, sound }),
            done: None,
            frame: None,
        };
        assert_eq!(rig.stage(), &Stage::Strip(strip), "{error:?}");
        let mut outs = rig.on(In::Strip(StripKey::Record));
        outs.extend(rig.wait(rig.countdown()));
        let again =
            |o: &Out| matches!(o, Out::Record { mic, sound, .. } if (*mic, *sound) == sounds);
        assert!(outs.iter().any(again), "{error:?} records again without it");
    }
}

#[test]
fn a_device_fault_once_live_ends_with_the_general_note() {
    let mut rig = Rig::with(|s| s.mic_on = true);
    let video = rig.recording();
    let fault = RecordNote::Fault(RecordError::NoMic);
    let outs = rig.on(In::Note { video, note: fault });
    assert!(outs.contains(&Out::Note(SnipNote::RecordFailed)));
    assert!(ended(&outs));
}

#[test]
fn stop_asks_once_then_waits_for_the_end() {
    let mut rig = Rig::new();
    let video = rig.recording();
    let stop = Out::Run {
        video,
        call: Call::Stop,
    };
    assert_eq!(rig.on(In::Record(RecKey::Stop)), [stop]);
    assert!(rec(&rig).stopping);
    for input in [In::Record(RecKey::Stop), In::Key, In::Record(RecKey::Mic)] {
        assert_eq!(rig.on(input.clone()), [], "{input:?}");
    }
}

#[test]
fn the_key_or_esc_stops_the_recording() {
    for input in [In::Key, In::Esc] {
        let mut rig = Rig::new();
        let video = rig.recording();
        let stop = Out::Run {
            video,
            call: Call::Stop,
        };
        assert_eq!(rig.on(input.clone()), [stop], "{input:?}");
    }
}

#[test]
fn the_end_shows_the_video_strip_still_lit() {
    let mut rig = Rig::new();
    let video = rig.recording();
    rig.on(In::Record(RecKey::Stop));
    let outs = rig.on(In::Note {
        video,
        note: done(RecordEnd::Stopped),
    });
    assert_eq!(outs, []);
    let strip = Strip {
        part: part(),
        home: home(),
        held: reel(video, Speed::PLAIN),
        done: None,
        frame: None,
    };
    assert_eq!(rig.stage(), &Stage::Strip(strip));
}

#[test]
fn a_stop_by_itself_keeps_the_video_with_a_note() {
    for (end, note) in [
        (RecordEnd::Longest, SnipNote::Longest),
        (RecordEnd::Full, SnipNote::DiskLow),
    ] {
        let mut rig = Rig::new();
        let video = rig.recording();
        let outs = rig.on(In::Note {
            video,
            note: done(end),
        });
        assert_eq!(outs, [Out::Note(note)]);
        assert!(matches!(rig.stage(), Stage::Strip(_)));
    }
}

#[test]
fn cancel_ends_and_deletes_with_no_question() {
    let mut rig = Rig::new();
    let video = rig.recording();
    let outs = rig.on(In::Record(RecKey::Cancel));
    let cancel = Out::Run {
        video,
        call: Call::Cancel,
    };
    assert_eq!(outs.first(), Some(&cancel));
    assert!(ended(&outs));
    assert!(!rig.flow.asking());
    assert!(!outs.contains(&Out::Delete(video)));
}
