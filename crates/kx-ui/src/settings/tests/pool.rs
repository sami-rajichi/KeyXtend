//! The renderer's buffer pool (ADR-0028): none kept by default, Qt's own size one setting away, and nothing past the
//! range or below 0; a refused value goes back to its default (the app's agree test loads it through the store).

use super::*;

#[test]
fn the_renderer_keeps_no_spare_buffers_unless_set() {
    let s = UiSettings::from_defaults().unwrap();
    assert_eq!(s.buffer_pool_bytes, 0, "ADR-0028");
    let qt = with("buffer_pool_bytes", i64::from(QT_POOL_BYTES));
    assert_eq!(
        validate_as::<UiSettings>(&qt),
        Ok(()),
        "Qt's own stays one setting away"
    );
    let qt: UiSettings = parse_as(&qt).unwrap();
    assert_eq!(qt.buffer_pool_bytes, QT_POOL_BYTES);
}

#[test]
fn the_buffer_pool_stays_in_its_range() {
    let end = i64::from(*BUFFER_POOL_BYTES.end());
    for edge in [i64::from(*BUFFER_POOL_BYTES.start()), end] {
        assert_eq!(
            validate_as::<UiSettings>(&with("buffer_pool_bytes", edge)),
            Ok(()),
            "{edge}"
        );
    }
    assert!(invalid(&with("buffer_pool_bytes", end + 1)));
    assert!(unparsed(&with("buffer_pool_bytes", -1)));
    assert!(unparsed(&with("buffer_pool_bytes", "none")));
}
