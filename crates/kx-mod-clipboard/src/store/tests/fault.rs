//! A fault-making shelf over the fake one: writes stop at a chosen point, as a crash or a failing disk would, and
//! one named blob fails its next remove or load.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use kx_module_api::ports::Room;

use super::*;

/// A blob by its folder and name, or none.
type Spot = Mutex<Option<(BlobName, BlobName)>>;

/// A fake shelf whose writes, once armed, stop after `left` of them: every later one, as after a crash, or only the
/// next one.
pub(super) struct Cut {
    /// The shelf written to.
    inner: FakeShelf,
    /// The writes that go through before the cut.
    left: AtomicUsize,
    /// Every write after the cut fails too.
    crash: AtomicBool,
    /// Writes are counted and cut.
    armed: AtomicBool,
    /// The writes asked since it was armed.
    asked: AtomicUsize,
    /// A blob whose next remove fails.
    snag: Spot,
    /// A blob whose next load is damaged, which sets it aside as the real shelf does.
    spoil: Spot,
}

impl Cut {
    /// A shelf over `inner` that cuts nothing until armed.
    pub(super) fn idle(inner: &FakeShelf) -> Arc<Self> {
        Arc::new(Self {
            inner: inner.clone(),
            left: AtomicUsize::new(0),
            crash: AtomicBool::new(false),
            armed: AtomicBool::new(false),
            asked: AtomicUsize::new(0),
            snag: Mutex::new(None),
            spoil: Mutex::new(None),
        })
    }

    /// A shelf over `inner` that fails every write after the first `left`.
    pub(super) fn crash(inner: &FakeShelf, left: usize) -> Arc<Self> {
        let cut = Self::idle(inner);
        cut.arm(left, true);
        cut
    }

    /// A shelf over `inner` that fails only the write after the first `left`.
    pub(super) fn once(inner: &FakeShelf, left: usize) -> Arc<Self> {
        let cut = Self::idle(inner);
        cut.arm(left, false);
        cut
    }

    /// Counts writes from now and cuts after the first `left`: every later one when `crash`, else only the next.
    pub(super) fn arm(&self, left: usize, crash: bool) {
        self.left.store(left, Ordering::SeqCst);
        self.crash.store(crash, Ordering::SeqCst);
        self.asked.store(0, Ordering::SeqCst);
        self.armed.store(true, Ordering::SeqCst);
    }

    /// Fails the next remove of `name` in `folder`.
    pub(super) fn snag(&self, folder: &BlobName, name: &BlobName) {
        *self.snag.lock().unwrap() = Some((folder.clone(), name.clone()));
    }

    /// Damages `name` in `folder` at its next load.
    pub(super) fn spoil(&self, folder: &BlobName, name: &BlobName) {
        *self.spoil.lock().unwrap() = Some((folder.clone(), name.clone()));
    }

    /// True when a write was cut.
    pub(super) fn hit(&self) -> bool {
        let armed = self.armed.load(Ordering::SeqCst);
        armed && self.asked.load(Ordering::SeqCst) > self.left.load(Ordering::SeqCst)
    }

    /// Counts a write and says whether it goes through.
    fn pass(&self) -> Result<(), SealedError> {
        if !self.armed.load(Ordering::SeqCst) {
            return Ok(());
        }
        let n = self.asked.fetch_add(1, Ordering::SeqCst);
        let left = self.left.load(Ordering::SeqCst);
        let cut = n == left || (self.crash.load(Ordering::SeqCst) && n > left);
        if cut { Err(SealedError::Io) } else { Ok(()) }
    }

    /// True, once, when `spot` names `name` in `folder`.
    fn takes(spot: &Spot, folder: &BlobName, name: &BlobName) -> bool {
        let mut spot = spot.lock().unwrap();
        let hit = spot.as_ref().is_some_and(|(f, n)| f == folder && n == name);
        if hit {
            *spot = None;
        }
        hit
    }
}

impl SealedShelf for Cut {
    fn load(&self, folder: &BlobName, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        if Self::takes(&self.spoil, folder, name) {
            self.inner.remove(folder, name)?;
            return Err(SealedError::Damaged);
        }
        self.inner.load(folder, name)
    }

    fn save(&self, folder: &BlobName, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        self.pass()?;
        self.inner.save(folder, name, bytes)
    }

    fn remove(&self, folder: &BlobName, name: &BlobName) -> Result<(), SealedError> {
        if Self::takes(&self.snag, folder, name) {
            return Err(SealedError::Io);
        }
        self.pass()?;
        self.inner.remove(folder, name)
    }

    fn drop_folder(&self, folder: &BlobName) -> Result<(), SealedError> {
        self.pass()?;
        self.inner.drop_folder(folder)
    }

    fn folders(&self) -> Result<Vec<BlobName>, SealedError> {
        self.inner.folders()
    }

    fn names(&self, folder: &BlobName) -> Result<Vec<BlobName>, SealedError> {
        self.inner.names(folder)
    }

    fn room(&self) -> Result<Room, SealedError> {
        self.inner.room()
    }

    fn free(&self) -> Result<u64, SealedError> {
        self.inner.free()
    }
}
