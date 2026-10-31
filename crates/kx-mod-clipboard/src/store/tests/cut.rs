//! Cut writes: a shelf that stops writing at a chosen point, as a crash or a failing disk would. At every point an
//! entry is whole or gone, never half, a list is old or new, never mixed, and a failed write finishes later.

use std::sync::atomic::{AtomicUsize, Ordering};

use kx_module_api::Redacted;
use kx_module_api::ports::Room;

use super::*;

/// The most write points a test walks; each test's work takes fewer.
const MAX_POINTS: usize = 80;
/// The text of the entry the tests move or delete: several small blobs.
const X: &str = "x takes several small blobs";

/// A fake shelf whose writes stop after `left` of them: every later one, as after a crash, or only the next one.
struct Cut {
    /// The shelf written to.
    inner: FakeShelf,
    /// The writes that go through before the cut.
    left: usize,
    /// Every write after the cut fails too.
    crash: bool,
    /// The writes asked so far.
    asked: AtomicUsize,
}

impl Cut {
    /// A shelf over `inner` that fails every write after the first `left`.
    fn crash(inner: &FakeShelf, left: usize) -> Arc<Self> {
        let asked = AtomicUsize::new(0);
        let inner = inner.clone();
        Arc::new(Self {
            inner,
            left,
            crash: true,
            asked,
        })
    }

    /// A shelf over `inner` that fails only the write after the first `left`.
    fn once(inner: &FakeShelf, left: usize) -> Arc<Self> {
        let asked = AtomicUsize::new(0);
        let inner = inner.clone();
        Arc::new(Self {
            inner,
            left,
            crash: false,
            asked,
        })
    }

    /// Counts a write and says whether it goes through.
    fn pass(&self) -> Result<(), SealedError> {
        let n = self.asked.fetch_add(1, Ordering::SeqCst);
        let cut = n == self.left || (self.crash && n > self.left);
        if cut { Err(SealedError::Io) } else { Ok(()) }
    }

    /// True when a write was cut.
    fn hit(&self) -> bool {
        self.asked.load(Ordering::SeqCst) > self.left
    }
}

impl SealedShelf for Cut {
    fn load(&self, folder: &BlobName, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        self.inner.load(folder, name)
    }

    fn save(&self, folder: &BlobName, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        self.pass()?;
        self.inner.save(folder, name, bytes)
    }

    fn remove(&self, folder: &BlobName, name: &BlobName) -> Result<(), SealedError> {
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
}

/// A small-blob store over `shelf`.
fn small(shelf: &FakeShelf) -> Store {
    Store::new(Arc::new(shelf.clone()), SMALL)
}

/// A shelf holding two day-1 entries in small blobs: `x`, in several parts with a thumbnail, and `y`.
fn two() -> (FakeShelf, u64, u64) {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (small(&shelf), History::default());
    let x = keep(&mut h, &mut s, X, &noon(1));
    let y = keep(&mut h, &mut s, "y", &noon(1));
    s.put_thumb(h.get(x).unwrap(), &grey(1)).unwrap();
    h.update(x, |e| e.thumb = true);
    s.save(&h).unwrap();
    (shelf, x, y)
}

/// A small-blob store over `cut`, and the history it loads.
fn cut_open(cut: &Arc<Cut>) -> (Store, History) {
    let mut s = Store::new(cut.clone(), SMALL);
    let h = s.load().unwrap().history;
    (s, h)
}

/// What the next run makes of `shelf`: it loads, saves and loads the same again, and no blob of an entry is left
/// that its folder's list does not name.
fn settle(shelf: &FakeShelf) -> (Store, History) {
    let mut s = small(shelf);
    let h = s.load().unwrap().history;
    s.save(&h).unwrap();
    assert_eq!(small(shelf).load().unwrap().history, h);
    for f in shelf.folders().unwrap() {
        let place = place_of(&f);
        for name in shelf.names(&f).unwrap() {
            let Some((id, blob)) = parts::owner(&name) else {
                continue;
            };
            let named = h
                .get(id)
                .filter(|e| Some(e.place) == place)
                .is_some_and(|e| match blob {
                    parts::Blob::Part(n) => n < usize::from(e.parts),
                    parts::Blob::Thumb => e.thumb,
                });
            assert!(named, "{} left in {}", name.as_str(), f.as_str());
        }
    }
    (s, h)
}

#[test]
fn a_crash_anywhere_in_a_pin_leaves_the_entry_whole_once() {
    for left in 0..MAX_POINTS {
        let (shelf, x, y) = two();
        let cut = Cut::crash(&shelf, left);
        let (mut s, mut h) = cut_open(&cut);
        h.pin(x);
        let _ = s.moved(&h, x, on(1));
        let (s, after) = settle(&shelf);
        let mut kept = ids(&after);
        kept.sort_unstable();
        assert_eq!(kept, [x, y], "cut at {left}");
        let whole = s.parts(after.get(x).unwrap());
        assert_eq!(whole, Ok(Some(vec![text(X)])), "cut at {left}");
        if !cut.hit() {
            assert!(after.get(x).unwrap().pinned());
            return;
        }
    }
    panic!("a pin took over {MAX_POINTS} writes");
}

#[test]
fn a_crash_anywhere_in_a_split_list_keeps_the_old_list_or_the_new() {
    for left in 0..MAX_POINTS {
        let (shelf, _, _) = two();
        let cut = Cut::crash(&shelf, left);
        let (mut s, old) = cut_open(&cut);
        let mut new = old.clone();
        for id in ids(&old) {
            new.update(id, |e| {
                e.title = Redacted::new(format!("a longer title {id}"));
            });
        }
        let _ = s.save(&new);
        let loaded = small(&shelf).load().unwrap();
        assert!(loaded.aside.is_empty(), "cut at {left}");
        assert!(
            loaded.history == old || loaded.history == new,
            "cut at {left}"
        );
        if !cut.hit() {
            assert_eq!(loaded.history, new);
            return;
        }
    }
    panic!("a list save took over {MAX_POINTS} writes");
}

#[test]
fn a_crash_anywhere_in_a_delete_leaves_the_entry_whole_or_gone() {
    for left in 0..MAX_POINTS {
        let (shelf, x, y) = two();
        let cut = Cut::crash(&shelf, left);
        let (mut s, mut h) = cut_open(&cut);
        let entry = h.remove(x).unwrap();
        let _ = s.delete(&entry);
        let _ = s.save(&h);
        let (s, after) = settle(&shelf);
        if let Some(e) = after.get(x) {
            assert_eq!(s.parts(e), Ok(Some(vec![text(X)])), "cut at {left}");
        }
        assert!(after.get(y).is_some(), "cut at {left}");
        if !cut.hit() {
            assert!(after.get(x).is_none());
            return;
        }
    }
    panic!("a delete took over {MAX_POINTS} writes");
}

#[test]
fn a_failed_write_anywhere_in_a_pin_finishes_at_the_next_save() {
    for left in 0..MAX_POINTS {
        let (shelf, x, _) = two();
        let cut = Cut::once(&shelf, left);
        let (mut s, mut h) = cut_open(&cut);
        h.pin(x);
        let first = s.moved(&h, x, on(1));
        let read = s.parts(h.get(x).unwrap());
        assert_eq!(read, Ok(Some(vec![text(X)])), "read between, cut at {left}");
        if first.is_err() {
            s.save(&h).unwrap();
        }
        assert_eq!(small(&shelf).load().unwrap().history, h, "cut at {left}");
        let mine = format!("{x:x}-");
        let old = names(&shelf, on(1));
        assert!(old.iter().all(|n| !n.starts_with(&mine)), "cut at {left}");
        if !cut.hit() {
            return;
        }
    }
    panic!("a pin took over {MAX_POINTS} writes");
}

#[test]
fn a_failed_put_removes_the_blobs_it_saved() {
    let shelf = FakeShelf::new();
    let mut s = Store::new(Cut::once(&shelf, 1), SMALL);
    let mut h = History::default();
    let id = h.add(copied(X, &noon(1)));
    assert_eq!(s.put(h.get(id).unwrap(), &[text(X)]), Err(SealedError::Io));
    assert!(names(&shelf, on(1)).is_empty());
}
