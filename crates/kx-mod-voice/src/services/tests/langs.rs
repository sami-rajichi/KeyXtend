//! The language sets live in their own file, read and checked with the table; the codes they hold, as the pages give
//! them, are pinned in `research::codes`.

use super::*;

#[test]
fn the_language_sets_come_from_their_own_file_alone() {
    let set = "\n[langs]\nmore = { en = \"en\" }\n";
    let inline = parse(&format!("{TEXT}{set}"));
    assert!(matches!(inline, Err(ServicesError::Parse(_))), "{inline:?}");
    let odd = Services::parse(TEXT, OWN, "whisper = 1");
    assert!(matches!(odd, Err(ServicesError::Langs(_))), "{odd:?}");
    let none = match Services::parse(TEXT, OWN, "") {
        Err(ServicesError::Row {
            why: ServiceError::Model { why, .. },
            ..
        }) => Some(why),
        _ => None,
    };
    assert_eq!(none, Some(ModelError::Set), "a set the file does not name");
}
