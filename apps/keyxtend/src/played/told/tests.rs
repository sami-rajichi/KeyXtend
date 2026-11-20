use super::*;
use crate::played::logged;
use crate::played::stage::{
    self, INDEX, LOAD, OPEN, PIECE, STAGE, STAGE_LINE, START, STOP, TOOK_MS,
};
use kx_module_api::ports::SpeechDone;

/// The level the stage lines take.
const DEBUG: Option<&str> = Some("debug");

/// What this run's log file holds, at the kernel's default level, once `f` has logged.
fn logged(f: impl FnOnce()) -> String {
    logged::at(None, f)
}

/// The device ids of `text`'s written-dictation lines, in order.
fn devices(text: &str) -> Vec<String> {
    logged::values(text, WROTE_LINE, kx_ipc::gate::DEVICE)
}

fn card(id: &str) -> CardId {
    CardId::new(id).unwrap()
}

#[test]
fn a_card_wrote_it_only_when_it_was_asked_and_served_to_the_end() {
    let vk = card("Vulkan0");
    assert_eq!(device(Some(&vk), true), "Vulkan0");
    assert_eq!(
        device(Some(&vk), false),
        DevicePick::CPU,
        "the processor took over"
    );
    assert_eq!(device(None, false), DevicePick::CPU);
}

#[test]
fn the_line_holds_the_device_id_as_one_word_at_the_default_level() {
    let text = logged(|| wrote("Vulkan0"));
    assert_eq!(devices(&text), ["Vulkan0"], "{text}");
}

#[cfg(feature = "mod-voice")]
mod runs {
    use super::*;
    use kx_module_api::ports::{CardUse, SpeechModel, SpeechStage};
    use kx_module_api::{Redacted, Zeroizing};
    use kx_platform_fake::FakeSpeech;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A dictation's ask on `card`, or on the processor, with `[voice]`'s defaults.
    fn ask(card: Option<CardId>) -> SpeechAsk {
        let model = SpeechModel {
            file: Redacted::new(PathBuf::from(r"D:\models\m.gguf")),
            lang: "en".into(),
            runner: "transcribe".into(),
            cpu_mb: *SpeechModel::CPU_MB.start(),
        };
        let card = card.map(|id| CardUse {
            id,
            need_mb: *CardUse::NEED_MB.start(),
            margin_mb: *CardUse::MARGIN_MB.start(),
        });
        let settings = kx_mod_voice::VoiceSettings::from_defaults().unwrap();
        settings.ask(Some(model), card)
    }

    /// A run's end, `card` saying whether a card served to the end.
    fn done(card: bool) -> SpeechNote {
        SpeechNote::Done(SpeechDone {
            pieces: 1,
            sound_ms: 0,
            wait_ms: 0,
            load_ms: 0,
            memory_mb: 0,
            card,
        })
    }

    #[test]
    fn each_written_dictation_logs_its_device_and_again_follows_the_new_card() {
        let (fake, heard) = (FakeSpeech::new(), Arc::new(AtomicUsize::new(0)));
        let count = Arc::clone(&heard);
        let tell = Box::new(move |_| {
            count.fetch_add(1, Ordering::SeqCst);
        });
        let told = Told(fake.clone());
        let text = logged(|| {
            let mut run = told.open(ask(Some(card("Vulkan0"))), tell).unwrap();
            assert!(fake.tell(done(true)));
            run.again(ask(Some(card("Vulkan1")))).unwrap();
            assert!(fake.tell(SpeechNote::Heard), "another note");
            assert!(fake.tell(done(true)));
            let mut lone = ask(Some(card("Vulkan9")));
            lone.model = None;
            assert!(
                run.again(lone).is_err(),
                "a refused ask keeps the card before"
            );
            assert!(fake.tell(done(true)));
            run.again(ask(None)).unwrap();
            assert!(fake.tell(done(false)));
        });
        let want = ["Vulkan0", "Vulkan1", "Vulkan1", DevicePick::CPU];
        assert_eq!(devices(&text), want, "{text}");
        assert_eq!(heard.load(Ordering::SeqCst), 5, "every note goes on");
    }

    #[test]
    fn a_listen_only_run_hands_its_speech_on_and_logs_no_device() {
        let fake = FakeSpeech::new();
        fake.set_speech(vec![vec![1, 0, 2, 0]]);
        let told = Told(fake.clone());
        let mut lone = ask(None);
        lone.model = None;
        let text = logged(|| {
            let mut run = told.open(lone, Box::new(|_| {})).unwrap();
            assert!(fake.tell(done(false)));
            let pieces = run.pieces().expect("its speech for the service");
            let bytes: Vec<&[u8]> = pieces.iter().map(|p| p.expose().as_slice()).collect();
            assert_eq!(bytes, [[1, 0, 2, 0].as_slice()]);
        });
        assert!(devices(&text).is_empty(), "no model wrote it: {text}");
    }

    /// What the stage test's piece says, which no line may hold.
    const SAID: &str = "Tunis in spring";
    /// The stage test's load time, in ms.
    const LOAD_MS: u32 = 1234;

    /// A local run's end after a load of `load_ms`.
    fn loaded(load_ms: u32) -> SpeechNote {
        let SpeechNote::Done(done) = done(false) else {
            unreachable!("done is an end")
        };
        SpeechNote::Done(SpeechDone { load_ms, ..done })
    }

    #[test]
    fn each_stage_of_a_dictation_logs_one_debug_line_and_never_its_text() {
        let fake = FakeSpeech::new();
        let told = Told(fake.clone());
        let text = logged::at(DEBUG, || {
            let mut run = told.open(ask(None), Box::new(|_| {})).unwrap();
            assert!(fake.tell(SpeechNote::Stage(SpeechStage::Open)));
            run.stop().unwrap();
            let said = Redacted::new(Zeroizing::new(SAID.to_owned()));
            assert!(fake.tell(SpeechNote::Piece {
                index: 0,
                text: said
            }));
            assert!(fake.tell(loaded(LOAD_MS)));
        });
        let stages = logged::values(&text, STAGE_LINE, STAGE);
        assert_eq!(stages, [START, OPEN, STOP, PIECE, LOAD], "{text}");
        assert_eq!(logged::values(&text, STAGE_LINE, INDEX), ["0"]);
        let took = logged::values(&text, STAGE_LINE, TOOK_MS);
        assert_eq!(took, [LOAD_MS.to_string()]);
        assert!(!text.contains(SAID), "never the text: {text}");
        let quiet = logged(|| stage::stage(START));
        assert!(!quiet.contains(STAGE_LINE), "debug lines only: {quiet}");
    }

    #[test]
    fn a_listen_only_run_logs_its_start_open_and_stop_but_no_load() {
        let fake = FakeSpeech::new();
        let told = Told(fake.clone());
        let mut lone = ask(None);
        lone.model = None;
        let text = logged::at(DEBUG, || {
            let mut run = told.open(lone, Box::new(|_| {})).unwrap();
            assert!(fake.tell(SpeechNote::Stage(SpeechStage::Open)));
            run.stop().unwrap();
            assert!(fake.tell(loaded(LOAD_MS)));
        });
        let stages = logged::values(&text, STAGE_LINE, STAGE);
        assert_eq!(stages, [START, OPEN, STOP], "{text}");
    }
}
