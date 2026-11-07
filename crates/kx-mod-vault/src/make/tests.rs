//! Make one: the set length, every group present, only the table's characters, an even spread over many draws, a
//! source that fails or sticks, and the table's own rules.

use super::*;
use crate::settings::MAKE_CHARS;
use kx_module_api::ports::Random;
use kx_platform_fake::FakeRandom;
use std::collections::HashSet;

/// The made password as text.
fn made(len: usize, random: &dyn Random) -> String {
    make(len, random).unwrap().expose().to_string()
}

/// A source that gives the same bytes every time.
struct Same(u8);

impl Random for Same {
    fn fill(&self, buf: &mut [u8]) -> Result<(), RandomError> {
        buf.fill(self.0);
        Ok(())
    }
}

#[test]
fn a_password_has_the_set_length_and_every_group() {
    let random = FakeRandom::new(9);
    for len in [*MAKE_CHARS.start(), 20, *MAKE_CHARS.end()] {
        for _ in 0..50 {
            let password = made(len, &random);
            assert_eq!(password.chars().count(), len);
            for group in groups() {
                assert!(password.chars().any(|c| group.contains(&c)), "{password}");
            }
        }
    }
}

#[test]
fn a_password_holds_only_the_tables_characters() {
    let pool: HashSet<char> = groups().iter().flatten().copied().collect();
    let random = FakeRandom::new(4);
    for _ in 0..200 {
        assert!(made(20, &random).chars().all(|c| pool.contains(&c)));
    }
}

#[test]
fn every_character_turns_up_over_many_draws() {
    let pool: HashSet<char> = groups().iter().flatten().copied().collect();
    let random = FakeRandom::new(17);
    let seen: HashSet<char> = (0..300)
        .flat_map(|_| made(20, &random).chars().collect::<Vec<_>>())
        .collect();
    assert_eq!(seen, pool);
}

#[test]
fn two_passwords_differ() {
    let random = FakeRandom::new(2);
    assert_ne!(made(20, &random), made(20, &random));
}

#[test]
fn a_failed_or_stuck_source_makes_nothing() {
    let random = FakeRandom::new(1);
    random.fail_next();
    assert_eq!(make(20, &random).err(), Some(MakeError::NoRandom));
    assert_eq!(
        make(20, &Same(0)).err(),
        Some(MakeError::NoRandom),
        "only one group's character"
    );
    assert_eq!(
        make(20, &Same(u8::MAX)).err(),
        Some(MakeError::NoRandom),
        "every draw rejected"
    );
}

#[test]
fn a_length_shorter_than_the_groups_is_refused() {
    let short = groups().len() - 1;
    assert_eq!(
        make(short, &FakeRandom::new(1)).err(),
        Some(MakeError::Short)
    );
}

#[test]
fn the_shipped_table_holds_three_groups_that_fit_the_shortest_password() {
    let parsed = parse(TABLE).unwrap();
    assert_eq!(parsed.len(), 3);
    assert!(parsed.len() <= *MAKE_CHARS.start());
    let letters: String = parsed[0].iter().collect();
    assert!(letters.contains('a') && letters.contains('Z'));
    assert_eq!(parsed[1].iter().collect::<String>(), "0123456789");
}

#[test]
fn a_table_with_an_empty_group_a_shared_or_hidden_character_is_refused() {
    let group = |chars: &str| format!("[[group]]\nname = \"g\"\nchars = \"{chars}\"\n");
    assert!(parse(&group("ab")).is_ok());
    assert!(parse(&group("")).is_err());
    assert!(parse(&[group("ab"), group("bc")].concat()).is_err());
    assert!(parse(&group("aa")).is_err());
    for bad in [" ", "\\t", "\\u200e", "\\u200b"] {
        assert!(parse(&group(&format!("a{bad}"))).is_err(), "{bad}");
    }
    assert!(parse("").is_err(), "no group at all");
}
