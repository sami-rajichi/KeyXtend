//! The rig's backup fakes: the boxes, places with a cloud folder, a stick, two drives and the data folder, and a job
//! queue the test runs by hand, so a job's answer comes when the test says.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

use kx_module_api::SettingsHandle;
use kx_module_api::ports::{BoxConfig, Boxes, PlaceKind};
use kx_platform_fake::{FakeBoxes, FakePlaces};

use crate::backup::{Ask, Done, Job, Links, Post, Runner};

// The test places, with slashes, which every OS reads.

/// The suggested cloud folder.
pub(super) const CLOUD: &str = "drive-c/Users/sam/OneDrive";
/// The suggested stick.
pub(super) const STICK: &str = "usb-e";
/// A drive in the folder list, and a folder on it.
pub(super) const DRIVE: &str = "drive-d";
pub(super) const FOLDER: &str = "drive-d/Backups";
/// The system drive, which holds the data folder.
pub(super) const SYSTEM: &str = "drive-c";
pub(super) const DATA: &str = "drive-c/Users/sam/AppData/Local/KeyXtend";

/// The jobs the actor queued, which the test runs.
#[derive(Clone, Default)]
pub(super) struct Queue {
    asks: Arc<Mutex<VecDeque<Ask>>>,
    stops: Arc<Mutex<usize>>,
}

impl Post for Queue {
    fn post(&mut self, ask: Ask) {
        self.asks.lock().unwrap().push_back(ask);
    }

    fn stop(&mut self) {
        *self.stops.lock().unwrap() += 1;
    }
}

/// The backup's fakes and a runner over them, as the job thread holds.
pub(super) struct Fakes {
    pub boxes: FakeBoxes,
    pub places: FakePlaces,
    pub queue: Queue,
    runner: Runner,
}

impl Fakes {
    /// The fakes, with every place there and roomy.
    pub fn new(cfg: BoxConfig) -> Self {
        let places = FakePlaces::new();
        places.add(PlaceKind::Cloud, "OneDrive", Path::new(CLOUD));
        places.add(PlaceKind::Removable, "E: USB", Path::new(STICK));
        places.add(PlaceKind::Drive, "D:", Path::new(DRIVE));
        places.add(PlaceKind::Drive, "C:", Path::new(SYSTEM));
        places.add_folder(Path::new(FOLDER));
        places.add_folder(Path::new(DATA));
        places.set_data_dir(Path::new(DATA));
        let boxes = FakeBoxes::new();
        let runner = Runner::new(boxes.start(cfg), Arc::new(places.clone()));
        Self {
            boxes,
            places,
            queue: Queue::default(),
            runner,
        }
    }

    /// The links the actor's backup takes, writing settings through `handle`.
    pub fn links(&self, handle: SettingsHandle) -> Links {
        Links {
            jobs: Box::new(self.queue.clone()),
            handle,
        }
    }

    /// The next job queued, if any.
    pub fn take(&self) -> Option<Ask> {
        self.queue.asks.lock().unwrap().pop_front()
    }

    /// How many jobs wait.
    pub fn queued(&self) -> usize {
        self.queue.asks.lock().unwrap().len()
    }

    /// True while a write waits.
    pub fn writes_queued(&self) -> bool {
        let asks = self.queue.asks.lock().unwrap();
        asks.iter().any(|ask| matches!(ask.job, Job::Write(_)))
    }

    /// Runs `ask` as the job thread would.
    pub fn run(&mut self, ask: Ask) -> Done {
        let Ask { round, job } = ask;
        Done {
            round,
            answer: self.runner.answer(job),
        }
    }

    /// How many times the actor stopped the queue.
    pub fn stops(&self) -> usize {
        *self.queue.stops.lock().unwrap()
    }
}
