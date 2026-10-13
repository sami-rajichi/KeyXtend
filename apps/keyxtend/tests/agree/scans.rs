//! Every character key on the keyboard's board has its scan code in the layouts module's table, so each one gets a
//! legend and types in every layout that has it.

use kx_mod_keyboard::BoardSpec;
use kx_mod_keyboard::spec::RowKey;
use kx_mod_layouts::scans;

#[test]
fn every_character_key_is_in_the_scan_table() {
    let spec = BoardSpec::shipped().unwrap();
    let table = scans().unwrap();
    let keys = (0..spec.rows.len()).flat_map(|r| spec.row_keys(r));
    let chars: Vec<_> = keys
        .filter_map(|k| match k {
            RowKey::Char { sc, .. } => Some(sc),
            RowKey::Named(_) => None,
        })
        .collect();
    assert!(!chars.is_empty(), "the board has character keys");
    for sc in chars {
        assert!(
            table.contains(&sc),
            "{:#06X} is not in scans.toml",
            sc.code()
        );
    }
}
