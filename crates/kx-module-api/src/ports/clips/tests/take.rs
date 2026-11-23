//! Quick-fill's and the key page's take: it reads twice a field's characters within the longest field's room, and it
//! names its asker, which the answer names back, so only the module that asked takes the clipboard's text.

use super::*;
use crate::ModuleId;

/// The module that asks in these tests.
const ASKER: ModuleId = ModuleId::new("vault");
/// Another module, which must not take the answer.
const OTHER: ModuleId = ModuleId::new("voice");

#[test]
fn a_take_reads_twice_a_fields_characters_but_never_past_the_longest_fields_room() {
    let most = MAX_FIELD_CHARS * 2;
    let take = ClipTake::new(ASKER, 3, 10);
    assert_eq!((take.asker, take.round, take.max_chars), (ASKER, 3, 20));
    assert_eq!(ClipTake::new(ASKER, 1, 10).cap(), 20);
    assert_eq!(ClipTake::new(ASKER, 1, MAX_FIELD_CHARS).cap(), most);
    assert_eq!(ClipTake::new(ASKER, 1, usize::MAX).cap(), most);
    let wide = ClipTake {
        max_chars: most + 1,
        ..take
    };
    assert_eq!(wide.cap(), most);
}

#[test]
fn an_answer_names_its_takes_asker_and_round_so_no_other_module_takes_it() {
    let taken = ClipTake::new(ASKER, 3, 10).answer(ClipText::Empty);
    let want = ClipTaken {
        asker: ASKER,
        round: 3,
        text: ClipText::Empty,
    };
    assert_eq!(taken, want);
    assert!(taken.is_for(ASKER, 3));
    assert!(
        !taken.is_for(OTHER, 3),
        "another module's take of the same round"
    );
    assert!(!taken.is_for(ASKER, 4), "an older take");
}
