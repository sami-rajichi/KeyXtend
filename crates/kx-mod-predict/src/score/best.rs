//! The best guesses so far: at most `limit` distinct texts, each with its best score and the stored spelling that gave
//! it, so a reading's thousands of completions never need sorting.

use std::cmp::Ordering;
use std::fmt;

/// One kept text: what the chip shows, its best score, and the stored spelling behind that score.
struct Item {
    text: String,
    score: f64,
    stored: String,
}

/// The best distinct texts offered so far.
pub(super) struct Best {
    /// The most texts kept.
    limit: usize,
    /// Each kept text.
    items: Vec<Item>,
}

impl fmt::Debug for Best {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Best")
            .field("limit", &self.limit)
            .field("kept", &self.items.len())
            .finish()
    }
}

impl Best {
    /// An empty list keeping at most `limit` texts.
    pub(super) fn new(limit: usize) -> Self {
        Self {
            limit,
            items: Vec::with_capacity(limit),
        }
    }

    /// True when a text scoring `score` could enter or raise its own score: room is left, or it scores at least the
    /// worst kept text, which a tie on score lets the text sorting first win.
    pub(super) fn wants(&self, score: f64) -> bool {
        self.limit > 0 && (self.items.len() < self.limit || score >= self.min().1)
    }

    /// Keeps `text`, stored as `stored`, at `score` when it beats the worst kept text, by score then by text sorting
    /// first, so the texts kept never depend on the order offered; a text kept already takes its best score's spelling.
    pub(super) fn offer(&mut self, text: String, score: f64, stored: &str) {
        if let Some(item) = self.items.iter_mut().find(|item| item.text == text) {
            if score > item.score {
                (item.score, item.stored) = (score, stored.to_owned());
            }
            return;
        }
        if self.limit == 0 {
            return;
        }
        if self.items.len() == self.limit {
            let (at, worst) = self.min();
            let kept = self.items.get(at).map(|item| (worst, item.text.as_str()));
            if kept.is_some_and(|(worst, held)| !beats((score, &text), (worst, held))) {
                return;
            }
            self.items.swap_remove(at);
        }
        let stored = stored.to_owned();
        self.items.push(Item {
            text,
            score,
            stored,
        });
    }

    /// The kept texts with their stored spellings, best first; equal scores go in text order.
    pub(super) fn into_texts(mut self) -> Vec<(String, String)> {
        self.items
            .sort_by(|a, b| rank((b.score, &b.text), (a.score, &a.text)));
        self.items.into_iter().map(|i| (i.text, i.stored)).collect()
    }

    /// The place and score of the worst kept text, by `rank`; a score no text loses to when none is kept.
    fn min(&self) -> (usize, f64) {
        let worst = (self.items.iter().enumerate())
            .min_by(|a, b| rank((a.1.score, &a.1.text), (b.1.score, &b.1.text)));
        worst.map_or((0, f64::NEG_INFINITY), |(at, item)| (at, item.score))
    }
}

/// How two scored texts rank: the higher score first, then the text sorting first.
fn rank(a: (f64, &str), b: (f64, &str)) -> Ordering {
    a.0.total_cmp(&b.0).then_with(|| b.1.cmp(a.1))
}

/// True when the scored text `a` ranks above `b`.
fn beats(a: (f64, &str), b: (f64, &str)) -> bool {
    rank(a, b) == Ordering::Greater
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The kept texts, best first.
    fn texts(best: Best) -> Vec<String> {
        best.into_texts().into_iter().map(|t| t.0).collect()
    }

    #[test]
    fn debug_shows_counts_never_the_guesses() {
        let mut best = Best::new(2);
        best.offer("secret".to_owned(), 1.0, "secret");
        let shown = format!("{best:?}");
        assert!(
            !shown.contains("secret") && shown.contains("kept: 1"),
            "{shown}"
        );
    }

    #[test]
    fn equal_scores_keep_the_texts_that_sort_first_in_any_order() {
        for order in [["b", "a", "c"], ["c", "b", "a"], ["a", "c", "b"]] {
            let mut best = Best::new(2);
            for text in order {
                best.offer(text.to_owned(), 1.0, text);
            }
            assert_eq!(texts(best), ["a", "b"], "{order:?}");
        }
    }

    #[test]
    fn a_text_offered_twice_keeps_the_spelling_of_its_better_score() {
        let mut best = Best::new(2);
        best.offer("May".to_owned(), 1.0, "may");
        best.offer("May".to_owned(), 2.0, "May");
        best.offer("May".to_owned(), 1.5, "may");
        assert_eq!(best.into_texts(), [("May".to_owned(), "May".to_owned())]);
    }
}
