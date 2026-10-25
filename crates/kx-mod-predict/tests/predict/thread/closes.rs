//! The thread opens only the language in front's pack, and closes the old one before it opens the next.

use super::{Live, hear, lane_of, lang};
use kx_mod_predict::{Flags, PackWriter};
use kx_module_api::ports::{PackError, PackView, Suggested, WordPacks};
use kx_platform_fake::{FakeFieldWatch, FakeSealedStore, FakeWordPacks};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Packs that count the views alive, and the most alive at once.
struct Counted {
    packs: FakeWordPacks,
    live: Arc<AtomicUsize>,
    most: Arc<AtomicUsize>,
}

/// A view that says when it closes.
struct View(Box<dyn PackView>, Arc<AtomicUsize>);

impl PackView for View {
    fn bytes(&self) -> &[u8] {
        self.0.bytes()
    }
}

impl Drop for View {
    fn drop(&mut self) {
        self.1.fetch_sub(1, Ordering::SeqCst);
    }
}

impl WordPacks for Counted {
    fn open(&self, iso: &str) -> Result<Option<Box<dyn PackView>>, PackError> {
        let Some(view) = self.packs.open(iso)? else {
            return Ok(None);
        };
        let live = self.live.fetch_add(1, Ordering::SeqCst) + 1;
        self.most.fetch_max(live, Ordering::SeqCst);
        Ok(Some(Box::new(View(view, Arc::clone(&self.live)))))
    }
}

/// Each test pack's one word, which the lane shows when its language arrives.
fn word(iso: &str) -> &'static str {
    match iso {
        "en" => "abc",
        "fr" => "abd",
        _ => "باب",
    }
}

#[test]
fn the_thread_closes_one_pack_before_it_opens_the_next() {
    let pack = |iso: &str| {
        let mut w = PackWriter::new(iso, Flags::default(), 1).unwrap();
        w.add_word(word(iso), 100).unwrap();
        w.finish().unwrap()
    };
    let fake = FakeWordPacks::new();
    for iso in ["en", "fr", "ar"] {
        fake.add(iso, pack(iso));
    }
    let (live, most) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let packs = Counted {
        packs: fake.clone(),
        live: Arc::clone(&live),
        most: Arc::clone(&most),
    };
    let (field, store) = (FakeFieldWatch::new(), FakeSealedStore::new());
    let (mut run, (_l, lanes)) = Live::start(&field, Arc::new(packs), &store, hear::<Suggested>);
    for iso in ["en", "fr", "ar", "en"] {
        run.send(&lang(iso));
    }
    for iso in ["en", "fr", "ar", "en"] {
        lane_of(&lanes, &[word(iso)]);
    }
    assert_eq!(
        (fake.opens("en"), fake.opens("fr"), fake.opens("ar")),
        (2, 1, 1)
    );
    assert_eq!(most.load(Ordering::SeqCst), 1, "one pack open at a time");
    run.stop();
    assert_eq!(live.load(Ordering::SeqCst), 0, "the stop closed it");
}
