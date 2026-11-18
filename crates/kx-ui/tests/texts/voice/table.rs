//! Every fact of the services table's rows has its text in each language with its values filled, and a service's own
//! words end the sentence after a colon, as `own` checks for its samples.

use crate::all;
use kx_mod_voice::services::{Facts, table};
use kx_module_api::ports::{Fact, FactValue};
use kx_ui::i18n::keys::{
    FACT_BILLS, FACT_FREE, FACT_KEEPS, FACT_PRICE, FACT_TRAINS, FACT_WARN, FACT_WHERE, fact_key,
};

/// A row's facts, each with the group its text is in, as its card and its Cloud page row word them.
fn grouped(f: &Facts) -> Vec<(&'static str, &Fact)> {
    let mut out = vec![
        (FACT_WHERE, &f.place),
        (FACT_TRAINS, &f.trains),
        (FACT_KEEPS, &f.keeps),
        (FACT_FREE, &f.free),
        (FACT_PRICE, &f.light),
        (FACT_PRICE, &f.heavy),
    ];
    out.extend(f.warn.iter().map(|w| (FACT_WARN, w)));
    out.extend(f.bills.iter().map(|b| (FACT_BILLS, b)));
    out
}

#[test]
fn every_fact_of_the_table_has_its_text_in_each_language_with_its_values_filled() {
    let rows = table().expect("the services table reads").rows();
    let facts = rows.iter().flat_map(|r| grouped(&r.facts));
    let facts: Vec<(&str, &Fact)> = facts.collect();
    for t in all() {
        let tag = t.lang().tag;
        for &(group, fact) in &facts {
            let key = fact_key(group, &fact.id);
            assert!(t.has(&key), "{tag}: {key}");
            let said = t.voice_fact(group, fact);
            assert!(!said.text.contains('{'), "{tag} {key}: {}", said.text);
            let own = fact
                .args
                .iter()
                .any(|a| matches!(a.value, FactValue::Text(_)));
            if own {
                let [lead, last] = &said.parts[..] else {
                    panic!("{tag} {key}: {:?}", said.parts);
                };
                let colon = lead.text.trim_end().ends_with(':');
                assert!(last.own && colon, "{tag} {key}: {}", said.text);
            }
        }
    }
}
