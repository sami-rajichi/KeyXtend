//! A form's management-key fields: paths of names parted by dots, sent after its region code and read back, at most
//! `MAX_ADMIN`, and refused both ways when bad or too many.

use super::*;
use crate::MAX_ADMIN;
use crate::rules::field_path;
use IpcError::{BadField, Inconsistent, TooLong};
use net_wire::URL;

/// A test of a one-letter form whose key test reads `paths`.
fn test(paths: &[&'static str]) -> NetAsk<'static> {
    let shape = admined(form("l", false, Vec::new()), paths);
    NetAsk::Test {
        id: 1,
        call: call(shape, NetHeader::Bearer),
    }
}

/// Where `part` starts in `bytes`.
fn at(bytes: &[u8], part: &[u8]) -> usize {
    bytes.windows(part.len()).position(|w| w == part).unwrap()
}

#[test]
fn a_reply_path_is_names_parted_by_dots() {
    for good in ["data", ADMIN[0], ADMIN[1], "a-b.c_d.E1"] {
        assert_eq!(field_path(good), Ok(()), "{good}");
    }
    let dots = ["", ".", "data.", ".data", "a..b"];
    let marks = ["a b", "a/b", "a:b", "data.clé", "a\0"];
    for text in dots.into_iter().chain(marks) {
        assert_eq!(field_path(text), Err(BadField), "{text:?}");
    }
}

#[test]
fn a_forms_paths_follow_its_region_code_each_after_its_length_and_read_back() {
    let (none, both) = (net_ask_bytes(&test(&[])), net_ask_bytes(&test(&ADMIN)));
    // A form with no paths ends with their count, 0, just before the address's length.
    let count = at(&none, ADDRESS.as_bytes()) - URL.width - 1;
    assert_eq!(none[count], 0);
    let mut want = none[HEADER_LEN..count].to_vec();
    want.push(u8::try_from(ADMIN.len()).unwrap());
    for p in ADMIN {
        want.push(u8::try_from(p.len()).unwrap());
        want.extend(p.as_bytes());
    }
    assert_eq!(both[HEADER_LEN..HEADER_LEN + want.len()], want[..]);
    assert_eq!(NetAsk::read(&mut &both[..]), Ok(test(&ADMIN)));
}

#[test]
fn a_bad_empty_or_extra_path_is_refused_both_ways() {
    refused(&test(&["a b"]), BadField);
    refused(&test(&["a..b"]), BadField);
    refused(&test(&[""]), Inconsistent);
    refused(&test(&["a"; MAX_ADMIN + 1]), TooLong);
    let good = net_ask_bytes(&test(&["a.b"]));
    let name = at(&good, b"a.b") - HEADER_LEN;
    refused_reads(&good, &[(name + 1, b" "), (name + 1, b"/")], BadField);
    let extra = [u8::try_from(MAX_ADMIN + 1).unwrap()];
    refused_reads(&good, &[(name - 2, &extra)], TooLong);
}
