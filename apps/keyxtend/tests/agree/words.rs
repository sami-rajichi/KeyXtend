//! The keyboard and predict agree on picked words: the longest word with chips, and its gap, fit the keyboard's
//! smallest text cap, so no setting makes it refuse a chip it was shown.

#[test]
fn the_longest_word_with_chips_and_its_gap_fit_the_smallest_text_cap() {
    let longest = *kx_mod_predict::MAX_WORD.end();
    let floor = usize::from(*kx_mod_keyboard::TEXT_CAP.start());
    assert!(longest < floor, "{longest} and its gap must fit {floor}");
}
