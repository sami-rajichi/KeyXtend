//! The `lone` flag of a latch role: only the shipped Win key has it, a role without it is off, and a bad flag is
//! refused.

use super::*;

/// The shipped Win key's role as `board.toml` writes it.
const WIN_ROLE: &str = "latch = \"win\", lone = true";

#[test]
fn only_the_shipped_win_key_sends_itself_alone() {
    let s = spec();
    let lone = s
        .roles()
        .into_iter()
        .filter_map(|(id, role, _)| match role {
            Role::Latch(r) if r.lone => Some(id),
            _ => None,
        });
    assert_eq!(lone.collect::<Vec<_>>(), ["win"]);
    let win = LatchRole {
        latch: Latch::Win,
        lone: true,
    };
    assert_eq!(named(&s, "win").role, Role::Latch(win));
    assert!(BOARD.contains(WIN_ROLE));
}

#[test]
fn a_latch_role_without_the_flag_is_not_lone() {
    let text = BOARD.replace(WIN_ROLE, "latch = \"win\"");
    let s = BoardSpec::read(&text).unwrap();
    assert_eq!(
        named(&s, "win").role,
        Role::Latch(LatchRole::of(Latch::Win))
    );
    assert!(!LatchRole::of(Latch::Win).lone);
}

#[test]
fn a_bad_lone_flag_is_refused() {
    let bad = [
        "latch = \"win\", lone = \"yes\"",
        "latch = \"win\", alone = true",
        "lone = true",
    ];
    for b in bad {
        let text = BOARD.replace(WIN_ROLE, b);
        let read = BoardSpec::read(&text);
        assert!(matches!(read, Err(BoardError::Parse(_))), "{b}");
    }
}
