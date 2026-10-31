//! A row whose size is unknown, as a share path the clipboard module never probes, says so in every language.

use super::{clip, heard, page, row, shown};
use crate::words::speaking;

#[test]
fn a_row_of_unknown_size_says_so_in_every_language() {
    let said = [
        ("en", "Size unknown"),
        ("fr", "Taille inconnue"),
        ("ar", "الحجم غير معروف"),
    ];
    for (iso, want) in said {
        let (rig, _host) = speaking(iso, 5);
        let (bus, acts, _subs) = heard(&rig);
        let mut far = row(9, "plan.pdf");
        far.bytes = None;
        shown(&rig, &bus, &acts, page(1, vec![far]));
        let v = clip(&rig);
        assert_eq!(v["rows"][0]["size"], want, "{iso}");
        let meta = v["rows"][0]["meta"].as_str().unwrap_or_default();
        assert!(meta.contains(want), "{iso}: {meta}");
    }
}
