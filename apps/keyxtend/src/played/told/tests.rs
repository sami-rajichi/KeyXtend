use super::*;
use kx_kernel::KernelSettings;
use kx_module_api::ports::SpeechDone;
use kx_settings::Files;
use kx_test_support::tempdir::TempDir;

/// What this run's log file holds, at the kernel's default level, once `f` has logged.
fn logged(f: impl FnOnce()) -> String {
    let dir = TempDir::new("played-told").unwrap();
    let files = Files::new(dir.path());
    let settings = KernelSettings::from_defaults().unwrap();
    let log = crate::log::open(&settings, &files).unwrap();
    tracing::dispatcher::with_default(&log.dispatch, f);
    std::fs::read_to_string(&files.log).unwrap()
}

/// The device ids of `text`'s written-dictation lines, in order.
fn devices(text: &str) -> Vec<String> {
    let device = |l: &str| {
        let word = l.split_whitespace().find_map(|w| w.strip_prefix("device="));
        word.map(str::to_owned)
    };
    text.lines()
        .filter(|l| l.contains(WROTE_LINE))
        .filter_map(device)
        .collect()
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
    use kx_module_api::Redacted;
    use kx_module_api::ports::{CardUse, SpeechModel};
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
}
