//! The emails the lane offers: the typed ones first and the notebook's after, one both typed and saved shown once as
//! saved, the start of a saved email, More past the lane, and the paged list, each with its kind.

use super::*;
use crate::email::saved::tests::notebook;
use crate::email::tests::{cfg, known, seen};
use ChipKind::{Email, More, Saved as Kept};

#[test]
fn the_notebooks_emails_follow_the_typed_ones_and_one_both_typed_and_saved_shows_once_as_saved() {
    let typed = known(&["nour@y.com", "Alex@x.org"]);
    let saved = notebook(&["alex@x.org", "lea@z.fr"]);
    let offer = Offer::new(&typed, Some(&saved));
    let want = [
        ("alex@x.org", Kept),
        ("nour@y.com", Email),
        ("lea@z.fr", Kept),
    ];
    assert_eq!(seen(&offer.all(5)), want, "the notebook's spelling wins");
    assert_eq!(offer.len(), 3);
    let alone = Offer::new(&typed, None);
    assert_eq!(
        seen(&alone.all(5)),
        [("Alex@x.org", Email), ("nour@y.com", Email)]
    );
}

#[test]
fn the_start_of_a_saved_email_shows_it_as_saved() {
    let cfg = cfg();
    let typed = known(&["leo@y.com"]);
    let saved = notebook(&["lea@z.fr"]);
    let offer = Offer::new(&typed, Some(&saved));
    assert_eq!(
        seen(&offer.starting("LE", &cfg, 5)),
        [("leo@y.com", Email), ("lea@z.fr", Kept)]
    );
    assert!(offer.starting("lea@", &cfg, 5).is_empty());
    assert!(offer.starting("l", &cfg, 5).is_empty());
}

#[test]
fn past_the_lane_saved_emails_wait_behind_more_and_the_list_pages_both_kinds() {
    let typed = known(&["a@x.co", "b@x.co"]);
    let saved = notebook(&["c@x.co", "d@x.co"]);
    let offer = Offer::new(&typed, Some(&saved));
    assert_eq!(
        seen(&offer.all(3)),
        [("b@x.co", Email), ("a@x.co", Email), ("", More)]
    );
    assert_eq!(seen(&offer.first(3)).last(), Some(&("c@x.co", Kept)));
    let page = offer.page(1, 2);
    assert_eq!((page.page, page.pages), (1, 2));
    assert_eq!(seen(&page.items), [("c@x.co", Kept), ("d@x.co", Kept)]);
    assert_eq!(offer.get(2), Some(("c@x.co", Kept)));
    assert_eq!(offer.get(4), None);
}

#[test]
fn kept_keeps_each_kind_in_order_and_an_empty_offer_has_one_page() {
    let typed = known(&["a@x.co", "b@x.co"]);
    let saved = notebook(&["c@x.co"]);
    let kept = Offer::new(&typed, Some(&saved)).kept(|e| e != "a@x.co");
    assert_eq!(seen(&kept.all(5)), [("b@x.co", Email), ("c@x.co", Kept)]);
    let empty = Emails::default();
    let none = Offer::new(&empty, None);
    assert!(none.is_empty());
    let page = none.page(0, 5);
    assert_eq!((page.page, page.pages, page.items.len()), (0, 1, 0));
}

#[test]
fn shown_leaves_the_notebooks_emails_out_only_in_forget_mode() {
    let typed = known(&["a@x.co"]);
    let saved = notebook(&["c@x.co"]);
    let lane = Offer::shown(&typed, &saved, false);
    assert_eq!(seen(&lane.all(5)), [("a@x.co", Email), ("c@x.co", Kept)]);
    let forgetting = Offer::shown(&typed, &saved, true);
    assert_eq!(
        seen(&forgetting.all(5)),
        [("a@x.co", Email)],
        "not ours to forget"
    );
}
