//! What the actor logs when a port fails: fixed texts and error kinds, never a dictated text, a folder or a cloud key.

use std::cell::RefCell;
use std::fmt::{self, Write as _};
use std::sync::Once;

use kx_module_api::ports::{
    BarButton, ClipError, FolderError, PanelButton, SealedError, VoiceClick, VoiceTab,
};
use kx_platform_fake::{CloudAnswer, ShelfCall};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

use super::rig::Rig;
use super::{Msg, cloud};

/// A dictated text that appears nowhere else.
const SECRET: &str = "zebra-quartz";
/// A part of the models folder's path, which no line may show.
const FOLDER: &str = "KeyXtend";

thread_local! {
    /// Every line logged on this thread while `lines_of` runs here.
    static SEEN: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

/// Installs the capture for the whole test binary, once.
static INSTALL: Once = Once::new();

/// Each line `f` logs on this thread, its fields joined.
fn lines_of(f: impl FnOnce()) -> Vec<String> {
    INSTALL.call_once(|| {
        // Only this file sets a capture in this test binary, so the first call installs it.
        let _ = tracing::subscriber::set_global_default(Lines);
    });
    SEEN.with(|seen| *seen.borrow_mut() = Some(Vec::new()));
    f();
    SEEN.with(|seen| seen.borrow_mut().take().unwrap_or_default())
}

/// Hands each line to the thread that logs it.
struct Lines;

/// Joins a line's fields, each as its debug text.
#[derive(Default)]
struct Recorder(String);

impl Visit for Recorder {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let _ = write!(self.0, "{}={value:?} ", field.name());
    }
}

impl Subscriber for Lines {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut line = Recorder::default();
        event.record(&mut line);
        SEEN.with(|seen| seen.borrow_mut().as_mut().map(|l| l.push(line.0.clone())));
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}

#[test]
fn a_failed_copy_save_reveal_or_run_logs_no_text_or_folder() {
    let lines = lines_of(|| {
        let mut rig = Rig::new();
        rig.fake.shelf.fail(ShelfCall::Save, SealedError::Io);
        rig.speak(SECRET);
        rig.fake.clip.queue(Err(ClipError::Busy));
        rig.act(VoiceClick::Bar(BarButton::Copy));
        rig.fake.reveal.set_error(Some(FolderError::Missing));
        rig.send(Msg::Open);
        rig.act(VoiceClick::Open(VoiceTab::Models));
        rig.act(VoiceClick::Button(PanelButton::Open));
        rig.listen();
        assert!(rig.fake.speech.die());
        rig.pump();
    });
    assert!(lines.len() >= 3, "{lines:?}");
    for line in &lines {
        assert!(!line.contains(SECRET) && !line.contains(FOLDER), "{line}");
    }
}

#[test]
fn a_cloud_dictation_logs_no_key_and_no_reply() {
    let lines = lines_of(|| {
        let mut rig = Rig::build(cloud::groq);
        rig.fake.shelf.fail(ShelfCall::Load, SealedError::Damaged);
        rig.dictate();
        cloud::set_key(&rig.fake, &cloud::key());
        rig.fake.cloud.script(vec![CloudAnswer::text(SECRET); 2]);
        rig.act(VoiceClick::Bar(BarButton::Again));
        assert_eq!(rig.typed_texts(), [format!("{SECRET} {SECRET}")]);
    });
    assert!(!lines.is_empty(), "the unread key is logged by kind");
    for line in &lines {
        assert!(
            !line.contains(SECRET) && !line.contains(&cloud::key()),
            "{line}"
        );
    }
}
