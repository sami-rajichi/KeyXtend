//! Paged lists (ADR-0008): no scrolling, the same number of rows on every page.

use serde::Serialize;

/// One page of a list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Page<T> {
    /// The rows on this page; `None` pads the last page to full size.
    pub rows: Vec<Option<T>>,
    /// This page, from 0.
    pub at: usize,
    /// How many pages there are; an empty list has one.
    pub count: usize,
}

/// Page `at` of `items` with `rows` rows each; a page past the end shows the last one.
pub fn page<T: Clone>(items: &[T], at: usize, rows: usize) -> Page<T> {
    let rows = rows.max(1);
    let count = items.len().div_ceil(rows).max(1);
    let at = at.min(count - 1);
    Page {
        rows: (0..rows)
            .map(|i| items.get(at * rows + i).cloned())
            .collect(),
        at,
        count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROWS: usize = 5;

    fn seven() -> Vec<u8> {
        (1..=7).collect()
    }

    #[test]
    fn seven_rows_make_two_pages_and_the_last_is_padded_to_five() {
        let first = page(&seven(), 0, ROWS);
        assert_eq!(first.count, 2);
        assert_eq!(first.rows, [1, 2, 3, 4, 5].map(Some));
        let last = page(&seven(), 1, ROWS);
        assert_eq!(last.rows, [Some(6), Some(7), None, None, None]);
    }

    #[test]
    fn a_page_past_the_end_shows_the_last_page() {
        let p = page(&seven(), 9, ROWS);
        assert_eq!((p.at, p.count), (1, 2));
    }

    #[test]
    fn an_empty_list_has_one_empty_page() {
        let p = page::<u8>(&[], 0, ROWS);
        assert_eq!((p.at, p.count), (0, 1));
        assert_eq!(p.rows, [None; ROWS]);
    }

    #[test]
    fn zero_rows_still_gives_one_row_per_page() {
        let p = page(&seven(), 0, 0);
        assert_eq!((p.rows.len(), p.count), (1, 7));
    }
}
