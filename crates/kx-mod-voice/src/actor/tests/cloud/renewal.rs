//! A used allowance's note through the actor (m6, N3): its renewal is read on the calendar as the fault comes, a clock
//! time that says whether it falls on the day the fault came.

use std::time::Duration;

use kx_module_api::ports::{BarHint, CloudFault, CloudWhy, Date, Renewal, VoiceBar};

use super::*;

/// The renewal the bar's note names after a used allowance that renews in `wait`, with the day the fault came.
fn note_after(wait: Duration) -> (Renewal, Date) {
    let mut rig = Rig::build(groq);
    let used = CloudFault::Allowance {
        renews_in: Some(wait),
    };
    rig.fake.cloud.script(vec![CloudAnswer::Fault(used)]);
    rig.dictate();
    let today = rig.actor.ports.calendar.now().date;
    let VoiceBar::Note(note) = rig.bar() else {
        panic!("a note shows");
    };
    let BarHint::Cloud {
        why: CloudWhy::Allowance {
            renews_at: Some(renews),
        },
        ..
    } = note.hint
    else {
        panic!("the used allowance's note, with its renewal");
    };
    (renews, today)
}

#[test]
fn a_used_allowances_note_names_its_renewal_and_whether_it_falls_on_the_day_it_came() {
    let (soon, today) = note_after(Duration::from_mins(90));
    let shown = (soon.at.date, soon.at.hour, soon.at.minute, soon.today);
    assert_eq!(shown, (today, 13, 30, true), "noon and 90 minutes");
    let (late, today) = note_after(Duration::from_hours(13));
    let next = Date::from_day_number(today.day_number() + 1).unwrap();
    let shown = (late.at.date, late.at.hour, late.at.minute, late.today);
    assert_eq!(shown, (next, 1, 0, false), "tomorrow at 01:00");
}
