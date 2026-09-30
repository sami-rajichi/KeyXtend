use super::*;
use proptest::prelude::*;

const EN: LayoutId = LayoutId::new(0x0409_0409);
const FR: LayoutId = LayoutId::new(0x040C_040C);
const AR: LayoutId = LayoutId::new(0x0401_0401);

fn label(id: LayoutId, short: &str, parts: &[&str], arabic: bool) -> LangLabel {
    LangLabel {
        id,
        short: short.into(),
        name: parts.iter().map(|&p| p.to_owned()).collect(),
        rtl: arabic,
        arabic,
    }
}

fn three() -> [LangLabel; 3] {
    [
        label(EN, "EN", &["English", "US"], false),
        label(FR, "FR", &["français", "France"], false),
        label(AR, "ع", &["العربية", "101"], true),
    ]
}

#[test]
fn the_carousel_goes_back_on_its_back_part_and_on_elsewhere() {
    assert_eq!(click(LangKey::Carousel, true), LangClick::Step(-1));
    assert_eq!(click(LangKey::Carousel, false), LangClick::Step(1));
}

#[test]
fn the_cycle_always_steps_on_and_the_list_opens() {
    assert_eq!(click(LangKey::Cycle, true), LangClick::Step(1));
    assert_eq!(click(LangKey::Cycle, false), LangClick::Step(1));
    assert_eq!(click(LangKey::List, true), LangClick::List);
    assert_eq!(click(LangKey::List, false), LangClick::List);
}

#[test]
fn the_carousel_shows_the_layouts_before_and_after_wrapping_as_a_step_does() {
    let list = three();
    let v = lang_view(&list, &list[0], 0);
    let short = |n: &Option<LangName>| n.as_ref().map(|n| n.short.clone());
    assert_eq!(short(&v.prev).as_deref(), Some("ع"), "EN wraps back to AR");
    assert_eq!(v.cur.short, "EN");
    assert_eq!(short(&v.next).as_deref(), Some("FR"));
    let prev = v.prev.as_ref().unwrap();
    assert!(
        prev.rtl && prev.arabic,
        "the Arabic name takes the Arabic font"
    );
    let v = lang_view(&list, &list[2], 0);
    assert_eq!(short(&v.next).as_deref(), Some("EN"), "AR wraps on to EN");
}

#[test]
fn the_cycle_dots_count_the_layouts_and_light_the_current_one() {
    let list = three();
    let v = lang_view(&list, &list[1], 0);
    assert_eq!((v.at, v.count), (Some(1), 3));
    assert_eq!(v.space, ["français", "France"]);
    let v = lang_view(&list, &label(LayoutId::new(9), "XX", &[], false), 0);
    assert_eq!(v.at, None, "a layout off the list lights no dot");
}

#[test]
fn a_lone_layout_or_an_empty_list_shows_only_the_current_name() {
    let list = three();
    let v = lang_view(&list[..1], &list[0], 0);
    assert!(v.prev.is_none() && v.next.is_none());
    let v = lang_view(&[], &LangLabel::default(), 0);
    assert!(v.prev.is_none() && v.next.is_none());
    assert_eq!((v.at, v.count), (None, 0));
}

#[test]
fn the_names_turn_the_way_the_key_stepped_once_the_switch_lands() {
    let ids = [EN, FR, AR];
    let mut t = Turn::new(EN);
    t.asked(-1);
    assert!(!t.seen(EN, &ids), "not switched yet");
    assert!(t.seen(AR, &ids));
    assert_eq!(t.turn(), -1, "from the left");
    t.asked(1);
    assert!(t.seen(EN, &ids));
    assert_eq!(t.turn(), 1);
}

#[test]
fn a_switch_made_another_way_does_not_turn() {
    let ids = [EN, FR, AR];
    let mut t = Turn::new(EN);
    t.asked(1);
    assert!(t.seen(AR, &ids), "the user picked AR, not the next layout");
    assert_eq!(t.turn(), 0);
    t.asked(1);
    t.clear();
    assert!(t.seen(EN, &ids));
    assert_eq!(t.turn(), 0, "a cleared ask never turns");
}

#[test]
fn a_step_asked_is_used_once() {
    let ids = [EN, FR, AR];
    let mut t = Turn::new(EN);
    t.asked(1);
    t.seen(FR, &ids);
    assert!(t.seen(AR, &ids), "a later switch the key did not ask");
    assert_eq!(t.turn(), 0);
}

/// One event at the key: a step asked, an ask cleared by a pick, or the app showing layout `n`; 3 is not listed.
#[derive(Clone, Debug)]
enum Event {
    Ask(i32),
    Clear,
    Show(usize),
}

fn event() -> impl Strategy<Value = Event> {
    prop_oneof![
        (-3_i32..=3).prop_map(Event::Ask),
        Just(Event::Clear),
        (0_usize..4).prop_map(Event::Show),
    ]
}

proptest! {
    /// The names turn only on the change where the step last asked lands, the way it asked, and only once.
    #[test]
    fn the_names_turn_only_where_the_asked_step_lands(events in prop::collection::vec(event(), 0..40)) {
        let ids = [EN, FR, AR];
        let at = |n: usize| ids.get(n).copied().unwrap_or(LayoutId::new(1));
        let mut t = Turn::new(EN);
        let (mut shown, mut asked) = (EN, 0);
        for e in events {
            match e {
                Event::Ask(by) => {
                    t.asked(by);
                    asked = by;
                }
                Event::Clear => {
                    t.clear();
                    asked = 0;
                }
                Event::Show(n) => {
                    let (now, before) = (at(n), t.turn());
                    prop_assert_eq!(t.seen(now, &ids), now != shown);
                    if now == shown {
                        prop_assert_eq!(t.turn(), before, "no change, no new turn");
                        continue;
                    }
                    let lands = asked != 0 && step_to(&ids, Some(shown), asked) == Some(now);
                    prop_assert_eq!(i32::from(t.turn()), if lands { asked.signum() } else { 0 });
                    (shown, asked) = (now, 0);
                }
            }
        }
    }
}
