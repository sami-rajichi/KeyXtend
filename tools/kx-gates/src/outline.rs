//! The scroll outline's edge windows as G8 and G9 find them: the product's shown windows titled with kx-ui's outline
//! title in a shipped language, and whether they lie under the keyboard in the z-order with only ours between.
#![cfg(windows)]

use windows::Win32::Foundation::HWND;

use crate::apps::Ctx;
use crate::{ftl, product, win};

/// The keyboard's name in `stack`.
pub const KEYBOARD: &str = "keyboard";
/// An edge's name in `stack`.
pub const EDGE: &str = "edge";

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

/// True when every one of `edges` lies under `keyboard` in `order`, topmost first, with only windows that are `ours`
/// between, such as the keyboard's shadow, which Windows keeps right under the keyboard it owns.
pub fn just_under<W: Copy + PartialEq>(
    order: &[W],
    keyboard: W,
    edges: &[W],
    ours: impl Fn(W) -> bool,
) -> bool {
    let Some(at) = order.iter().position(|w| *w == keyboard) else {
        return false;
    };
    let below = &order[at + 1..];
    let Some(last) = below.iter().rposition(|w| edges.contains(w)) else {
        return false;
    };
    let span = &below[..=last];
    let all_there = edges.iter().all(|e| span.contains(e));
    all_there && span.iter().all(|&w| edges.contains(&w) || ours(w))
}

/// `order`, topmost first, from the first to the last of `keyboard` and `edges`, any other window named by `name`;
/// empty when none of them is in it.
pub fn stack<W: Copy + PartialEq>(
    order: &[W],
    keyboard: W,
    edges: &[W],
    name: impl Fn(W) -> String,
) -> Vec<String> {
    let ours = |w: &W| *w == keyboard || edges.contains(w);
    let (Some(first), Some(last)) = (order.iter().position(ours), order.iter().rposition(ours))
    else {
        return Vec::new();
    };
    let named = |&w: &W| match w {
        w if w == keyboard => KEYBOARD.to_string(),
        w if edges.contains(&w) => EDGE.to_string(),
        w => name(w),
    };
    order[first..=last].iter().map(named).collect()
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
    fn the_edges_lie_under_the_keyboard_with_only_ours_between() {
        let (kb, edges) = (5, [7, 6, 8, 9]);
        let ours = |w: i32| w == 4;
        assert!(just_under(&[1, 5, 6, 7, 8, 9, 2], kb, &edges, ours));
        assert!(
            just_under(&[1, 5, 4, 6, 7, 8, 9], kb, &edges, ours),
            "the keyboard's shadow between"
        );
        assert!(
            !just_under(&[1, 5, 6, 7, 3, 8, 9], kb, &edges, ours),
            "another app's window between"
        );
        assert!(
            !just_under(&[6, 5, 7, 8, 9], kb, &edges, ours),
            "one over the keyboard"
        );
        assert!(!just_under(&[5, 6, 7], kb, &edges, ours), "one is missing");
        assert!(!just_under(&[1, 2], kb, &edges, ours), "no keyboard");
        assert!(!just_under(&[5, 6], kb, &[], ours), "no edge");
    }

    #[test]
    fn the_stack_names_ours_and_what_lies_between_from_the_first_to_the_last() {
        let name = |w: i32| format!("w{w}");
        let (kb, edges) = (5, [6, 7]);
        assert_eq!(
            stack(&[1, 5, 3, 6, 7, 2], kb, &edges, name),
            [KEYBOARD, "w3", EDGE, EDGE]
        );
        assert_eq!(
            stack(&[6, 1, 5, 7], kb, &edges, name),
            [EDGE, "w1", KEYBOARD, EDGE],
            "an edge over the keyboard"
        );
        assert!(stack(&[1, 2], kb, &edges, name).is_empty(), "none of ours");
    }
}
