//! The address-bar table's checks.

use super::refused;

#[test]
fn an_address_bar_row_that_could_match_nothing_or_another_browser_is_refused() {
    refused(&[
        (|g| g.bars.clear(), "g19_product.bars"),
        (|g| g.bars[0].browsers.clear(), "g19_product.bars"),
        (
            |g| g.bars[0].browsers.push("other.exe".into()),
            "g19_product.bars",
        ),
        (
            |g| g.bars[1].browsers.push("Chrome.EXE".into()),
            "g19_product.bars",
        ),
        (
            |g| {
                let r = &mut g.bars[0];
                r.ids.clear();
                r.classes.clear();
                r.names.clear();
            },
            "g19_product.bars",
        ),
        (|g| g.bars[0].names.push(" ".into()), "g19_product.bars"),
    ]);
}
