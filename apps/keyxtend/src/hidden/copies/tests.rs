use super::super::fixture::{APP, Bed, Held, hub};
use super::*;
use kx_platform::ui::HiddenError;
use kx_platform_fake::FakeHidden;
use kx_ui::Request;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

/// A package of `APP`, and a package of another app.
const PACKAGE: &str = "Claude_pzs8sxrjxfjjc";
const OTHER: &str = "Other_8wekyb3d8bbwe";
/// A second package of `APP`, as another edition of it would have.
const TWIN: &str = "Claude_twin";

/// The copy in package `package` of app `app`.
fn copy(package: &str, app: &str) -> HiddenCopy {
    HiddenCopy::new(PathBuf::from(package), package.into(), app.into())
}

/// A normal run over a data folder: its kernel booted, its hub, and a port listing `copies`.
struct NormalRun {
    bed: Bed,
    kernel: Kernel,
    hub: Arc<Hub>,
    requests: Receiver<Request>,
    port: Arc<FakeHidden>,
}

impl NormalRun {
    /// A run over a fresh data folder.
    fn new(copies: Vec<HiddenCopy>) -> Self {
        Self::over(Bed::new(), copies)
    }

    /// A run over `bed`, whose settings an earlier run may have saved.
    fn over(bed: Bed, copies: Vec<HiddenCopy>) -> Self {
        let mut kernel = bed.kernel();
        kernel.boot().unwrap();
        let (hub, requests) = hub();
        let port = Arc::new(FakeHidden::new());
        port.set_copies(Ok(copies));
        Self {
            bed,
            kernel,
            hub,
            requests,
            port,
        }
    }

    /// Asks about the copies, as the app does once the modules started.
    fn ask(&self) -> Asked {
        Asked::ask(self.port.clone(), &self.kernel, &self.hub)
    }

    /// Answers each question the banner shows now with `yes`, as a click does, and gives what reached the app's queue.
    fn answered(&self, yes: bool) -> Vec<Notice> {
        let shown = self.hub.take_notices();
        let answer = |id| {
            assert!(self.hub.answer(id, yes), "it asks");
            let Ok(Request::Answered { notice, .. }) = self.requests.try_recv() else {
                panic!("the answer reaches the app's queue");
            };
            notice
        };
        shown.iter().map(|n| answer(n.id)).collect()
    }

    /// Hands `asked` the answers to each question shown now, waits for its deletes, and counts them.
    fn answer_all(&mut self, asked: &mut Asked, yes: bool) -> usize {
        let answers = self.answered(yes);
        for notice in &answers {
            asked.answer(&mut self.kernel, notice, yes);
        }
        asked.deletes().settle();
        answers.len()
    }

    /// The packages the kernel's settings keep.
    fn kept(&self) -> Vec<String> {
        self.kernel.kernel_settings().unwrap().kept_copies
    }
}

#[test]
fn each_copy_not_kept_is_asked_about_once_by_its_apps_name() {
    let run = NormalRun::new(vec![copy(PACKAGE, APP), copy(OTHER, "Other")]);
    let _asked = run.ask();
    let shown = run.hub.take_notices();
    assert_eq!(shown.len(), 2);
    assert!(
        shown.iter().all(|n| n.ask.is_some()),
        "each one asks Yes or No"
    );
    assert!(shown[0].text.contains(APP), "{}", shown[0].text);
    assert!(run.hub.take_notices().is_empty(), "asked once");
}

#[test]
fn yes_deletes_the_copy_and_keeps_nothing() {
    let mut run = NormalRun::new(vec![copy(PACKAGE, APP)]);
    let mut asked = run.ask();
    assert_eq!(run.answer_all(&mut asked, true), 1);
    assert_eq!(run.port.removed(), [copy(PACKAGE, APP)]);
    assert!(run.hub.take_notices().is_empty() && run.kept().is_empty());
}

#[test]
fn a_failed_delete_says_so_and_keeps_nothing() {
    let mut run = NormalRun::new(vec![copy(PACKAGE, APP)]);
    run.port.set_remove_error(Some(HiddenError::Undeleted));
    let mut asked = run.ask();
    run.answer_all(&mut asked, true);
    let shown = run.hub.take_notices();
    assert_eq!(shown.len(), 1, "the failure notice");
    assert!(shown[0].ask.is_none() && shown[0].text.contains(APP));
    assert!(run.port.removed().is_empty() && run.kept().is_empty());
}

#[test]
fn no_keeps_the_copy_for_good_so_the_next_run_does_not_ask() {
    let mut run = NormalRun::new(vec![copy(PACKAGE, APP)]);
    let mut asked = run.ask();
    run.answer_all(&mut asked, false);
    assert_eq!(run.kept(), [PACKAGE]);
    assert!(run.port.removed().is_empty());
    let next = NormalRun::over(run.bed, vec![copy(PACKAGE, APP)]);
    let _asked = next.ask();
    assert!(
        next.hub.take_notices().is_empty(),
        "a kept copy is never asked about again"
    );
}

#[test]
fn an_answer_to_another_notice_changes_nothing() {
    let mut run = NormalRun::new(vec![copy(PACKAGE, APP)]);
    let mut asked = run.ask();
    let other = Notice {
        ask: Some(&NoticeAsk::YES_NO),
        ..Notice::plain(keys::KEYS_NOT_SENT)
    };
    asked.answer(&mut run.kernel, &other, true);
    asked.answer(&mut run.kernel, &other, false);
    assert!(run.port.removed().is_empty() && run.kept().is_empty());
    assert_eq!(
        run.answer_all(&mut asked, true),
        1,
        "its own question still waits"
    );
    assert_eq!(run.port.removed(), [copy(PACKAGE, APP)]);
}

#[test]
fn a_failed_list_asks_nothing() {
    let run = NormalRun::new(Vec::new());
    run.port.set_copies(Err(HiddenError::Unreadable));
    let _asked = run.ask();
    assert!(run.hub.take_notices().is_empty());
}

#[test]
fn two_copies_of_one_app_are_asked_about_in_turn() {
    let mut run = NormalRun::new(vec![copy(PACKAGE, APP), copy(TWIN, APP)]);
    let mut asked = run.ask();
    assert_eq!(
        run.answer_all(&mut asked, true),
        1,
        "one question shows at a time"
    );
    assert_eq!(run.answer_all(&mut asked, false), 1, "then the twin's");
    assert_eq!(run.port.removed(), [copy(PACKAGE, APP)]);
    assert_eq!(run.kept(), [TWIN]);
}

#[test]
fn a_package_listed_twice_is_kept_once() {
    let mut run = NormalRun::new(vec![copy(PACKAGE, APP), copy(PACKAGE, APP)]);
    let mut asked = run.ask();
    run.answer_all(&mut asked, false);
    assert_eq!(run.answer_all(&mut asked, false), 1, "asked in turn");
    assert_eq!(run.kept(), [PACKAGE]);
}

#[test]
fn yes_deletes_off_the_kernels_thread_and_settle_waits_for_it() {
    let mut run = NormalRun::new(Vec::new());
    let (held, open) = Held::new(vec![copy(PACKAGE, APP)]);
    let mut asked = Asked::ask(held.clone(), &run.kernel, &run.hub);
    for notice in run.answered(true) {
        asked.answer(&mut run.kernel, &notice, true);
    }
    assert!(held.fake.removed().is_empty(), "the answer did not wait");
    open.send(()).unwrap();
    asked.deletes().settle();
    assert_eq!(held.fake.removed(), [copy(PACKAGE, APP)]);
}
