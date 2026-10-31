//! The largest shapes: a 32 MB entry, and a list of 5,000 entries at the longest title and search text, each split
//! under the port's blob size and joined back.

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipPart, ImageCodec, MAX_BLOB_BYTES};

use super::*;
use crate::fixture::{CFG, LIMITS};
use crate::settings::MAX_ENTRIES;

/// The longest character in UTF-8, 4 bytes.
const WIDE: char = '😀';

#[test]
fn a_32_mb_entry_splits_and_joins() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let size = usize::try_from(LIMITS.max_entry_bytes).unwrap();
    let bytes: Vec<u8> = (0..=u8::MAX).cycle().take(size).collect();
    let codec = ImageCodec::Png;
    let parts = vec![ClipPart::Image {
        codec,
        bytes: Redacted::new(bytes),
    }];
    let id = h.add(copied("image", &noon(1)));
    let n = s.put(h.get(id).unwrap(), &parts).unwrap();
    assert_eq!(
        usize::from(n),
        size / MAX_BLOB_BYTES + 1,
        "the header spills into one more"
    );
    for i in 0..usize::from(n) {
        let blob = shelf
            .blob(&folder(on(1)), &part_name(id, i).unwrap())
            .unwrap();
        assert!(blob.len() <= MAX_BLOB_BYTES);
    }
    h.update(id, |e| e.parts = n);
    assert_eq!(s.parts(h.get(id).unwrap()), Ok(Some(parts)));
}

#[test]
fn a_list_of_5000_entries_at_the_longest_texts_splits_and_joins() {
    let shelf = FakeShelf::new();
    let day1 = folder(on(1));
    let title = WIDE.to_string().repeat(CFG.title_chars);
    let search = WIDE.to_string().repeat(CFG.search_chars);
    let most = u64::try_from(*MAX_ENTRIES.end()).unwrap();
    let entries = (0..most).map(|id| {
        shelf.plant(&day1, &part_name(id, 0).unwrap(), vec![]);
        let mut e = copied("x", &noon(1));
        (e.id, e.seq, e.parts) = (id, id, 1);
        e.title = Redacted::new(title.clone());
        e.search = Redacted::new(search.clone());
        e
    });
    let h = History::new(entries.collect(), most);
    open(&shelf).save(&h).unwrap();
    let lists: Vec<String> = names(&shelf, on(1))
        .into_iter()
        .filter(|n| n.starts_with("list"))
        .collect();
    assert!(lists.len() > 1, "{lists:?}");
    for name in &lists {
        let blob = shelf.blob(&day1, &BlobName::new(name).unwrap()).unwrap();
        assert!(blob.len() <= MAX_BLOB_BYTES, "{name}");
    }
    assert_eq!(reload(&shelf).history, h);
}
