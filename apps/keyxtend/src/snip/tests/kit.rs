//! The ports a kit makes for each run: a hidden run gets no Snips folder, every port but the freeze works over the one
//! folder, and the recorder holds the app's seal.

use super::{dirs, folder, marked};
use crate::hidden::Run;
use crate::snip::{Guarded, Kit, Ports};
use kx_module_api::Mono;
use kx_module_api::ports::{
    BASE_DPI, FileAsk, MediaOpen, MicPasses, MicSeal, Point, Recorder, Rect, ScreenFreeze,
    SnipFolder,
};
use kx_platform_fake::{
    FakeClock, FakeFileAsk, FakeMediaOpen, FakeRecorder, FakeScreenFreeze, FakeSnipFolder,
};
use kx_test_support::tempdir::TempDir;
use std::sync::{Arc, Mutex};

/// The one screen.
const SCREEN: Rect = Rect {
    x: 0,
    y: 0,
    w: 32,
    h: 18,
};

/// Fakes over a normal and a hidden folder, keeping the folder and the seal each port was made with.
struct Probe {
    /// A normal run's folder, which the ports' fakes work in.
    normal: FakeSnipFolder,
    /// A hidden run's: none.
    hidden: FakeSnipFolder,
    /// The folder each port was made over.
    given: Mutex<Vec<Arc<dyn SnipFolder>>>,
    /// The seal each recorder was made with.
    seals: Mutex<Vec<MicSeal>>,
}

impl Kit for Probe {
    fn freeze(&self) -> Arc<dyn ScreenFreeze> {
        let pointer = Point { x: 0, y: 0 };
        Arc::new(FakeScreenFreeze::new(vec![(SCREEN, BASE_DPI)], pointer))
    }

    fn folder(&self, hidden: bool) -> Arc<dyn SnipFolder> {
        Arc::new(if hidden { &self.hidden } else { &self.normal }.clone())
    }

    fn recorder(&self, seal: MicSeal, folder: Arc<dyn SnipFolder>) -> Arc<dyn Recorder> {
        self.given.lock().unwrap().push(folder);
        self.seals.lock().unwrap().push(seal.clone());
        Arc::new(FakeRecorder::new(seal, &self.normal))
    }

    fn dialogs(&self, folder: Arc<dyn SnipFolder>) -> Arc<dyn FileAsk> {
        self.given.lock().unwrap().push(folder);
        Arc::new(FakeFileAsk::new(&self.normal))
    }

    fn player(&self, folder: Arc<dyn SnipFolder>) -> Arc<dyn MediaOpen> {
        self.given.lock().unwrap().push(folder);
        Arc::new(FakeMediaOpen::new(&self.normal))
    }

    fn guarded(&self) -> Guarded {
        marked
    }
}

#[test]
fn a_hidden_run_gets_no_snips_folder_and_each_port_works_over_the_one_folder() {
    let user = TempDir::new("snip-kit").unwrap();
    let clock = Arc::new(FakeClock::new(Mono::from_us(0)));
    let kit = Probe {
        normal: folder(&dirs(&user)),
        hidden: FakeSnipFolder::hidden(clock),
        given: Mutex::default(),
        seals: Mutex::default(),
    };
    for run in [Run::Normal, Run::hidden("app".into())] {
        let ports = Ports::of(&kit, &run, MicSeal::new());
        assert_eq!(ports.folder.snips().is_none(), run.is_hidden(), "{run:?}");
        let given: Vec<_> = kit.given.lock().unwrap().drain(..).collect();
        assert_eq!(given.len(), 3, "the recorder, the dialogs and the player");
        assert!(
            given.iter().all(|f| Arc::ptr_eq(f, &ports.folder)),
            "{run:?}"
        );
        let seal = kit.seals.lock().unwrap().pop().unwrap();
        assert!(
            seal.opens(&ports.seal.pass()),
            "the recorder holds the app's seal"
        );
    }
}
