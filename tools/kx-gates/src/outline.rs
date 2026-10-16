//! The scroll outline's edge windows as G8 and G9 find them: the product's shown windows titled with kx-ui's outline
//! title in a shipped language, and whether they lie just under the keyboard in the z-order.
#![cfg(windows)]

use windows::Win32::Foundation::HWND;

use crate::apps::Ctx;
use crate::{ftl, product, win};

/// The outline's title in every shipped language.
pub fn titles(ctx: &Ctx) -> Result<Vec<String>, String> {
    ftl::every(&ctx.cfg.outline.title)
}

/// Those of `windows`, with their titles, whose title is one of `titles`, in their order.
pub fn titled<W: Copy>(windows: &[(W, String)], titles: &[String]) -> Vec<W> {
    windows
        .iter()
        .filter(|(_, t)| titles.contains(t))
        .map(|(w, _)| *w)
        .collect()
}

/// The product's shown outline edges, topmost first.
pub fn shown(titles: &[String]) -> Vec<HWND> {
    let named: Vec<(HWND, String)> = product::shown()
        .into_iter()
        .map(|w| (w, win::title(w)))
        .collect();
    titled(&named, titles)
}

/// True when `edges` lie right under `keyboard` in `order`, topmost first, with no other window between them.
pub fn just_under<W: Copy + PartialEq>(order: &[W], keyboard: W, edges: &[W]) -> bool {
    let Some(at) = order.iter().position(|w| *w == keyboard) else {
        return false;
    };
    let below = order[at + 1..].iter().take(edges.len());
    !edges.is_empty() && below.len() == edges.len() && below.into_iter().all(|w| edges.contains(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles() -> Vec<String> {
        vec!["Scroll outline".to_string(), "إطار التمرير".to_string()]
    }

    #[test]
    fn only_windows_titled_as_the_outline_count_in_their_order() {
        let windows = [
            (1, "KeyXtend keyboard".to_string()),
            (2, "إطار التمرير".to_string()),
            (3, "Scroll outline".to_string()),
            (4, "Scroll outline (old)".to_string()),
        ];
        assert_eq!(titled(&windows, &titles()), [2, 3]);
        assert!(titled(&windows, &[]).is_empty());
    }

    #[test]
    fn the_edges_lie_right_under_the_keyboard_with_nothing_between() {
        let (kb, edges) = (5, [7, 6, 8, 9]);
        assert!(just_under(&[1, 5, 6, 7, 8, 9, 2], kb, &edges));
        assert!(
            !just_under(&[1, 5, 6, 7, 3, 8, 9], kb, &edges),
            "another window between"
        );
        assert!(
            !just_under(&[6, 7, 8, 9, 5], kb, &edges),
            "over the keyboard"
        );
        assert!(!just_under(&[5, 6, 7], kb, &edges), "one is missing");
        assert!(!just_under(&[1, 2], kb, &edges), "no keyboard");
        assert!(!just_under(&[5, 6], kb, &[]), "no edge");
    }
}
