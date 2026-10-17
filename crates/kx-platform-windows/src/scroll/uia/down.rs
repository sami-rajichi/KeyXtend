//! The walk down a window to a point (ADR-0029): for a target our window covers, UI Automation goes down from the
//! target window's element to the deepest one holding the point, as `ElementFromPoint` would without ours over it.

use kx_module_api::ports::Point;
use windows::Win32::UI::Accessibility::{IUIAutomationElement, IUIAutomationTreeWalker};

use crate::screen::{contains, rect_of};

/// A tree the walk down reads: UI Automation's control view, or a fake in tests.
pub(super) trait Tree {
    /// One place in the tree.
    type Node;
    /// The first child of `node`; none for a leaf, or when the tree cannot say.
    fn first(&self, node: &Self::Node) -> Option<Self::Node>;
    /// The sibling after `node`; none after the last.
    fn next(&self, node: &Self::Node) -> Option<Self::Node>;
    /// True when the box of `node` holds screen point `p`.
    fn holds(&self, node: &Self::Node, p: Point) -> bool;
}

/// The deepest node down from `top` whose box holds `p`, taking the first such child at each level: `top` when no
/// child holds it, and as deep as it got once `late` says the time is up.
pub(super) fn deepest<T: Tree>(
    tree: &T,
    top: T::Node,
    p: Point,
    late: impl Fn() -> bool,
) -> T::Node {
    let mut at = top;
    let mut next = tree.first(&at);
    while let Some(node) = next {
        if late() {
            break;
        }
        next = if tree.holds(&node, p) {
            let below = tree.first(&node);
            at = node;
            below
        } else {
            tree.next(&node)
        };
    }
    at
}

/// UI Automation's control view, as the walk up reads it.
pub(super) struct View<'a>(pub &'a IUIAutomationTreeWalker);

#[allow(unsafe_code, reason = "Plain UI Automation tree reads.")]
impl Tree for View<'_> {
    type Node = IUIAutomationElement;

    fn first(&self, node: &IUIAutomationElement) -> Option<IUIAutomationElement> {
        // SAFETY: a plain call on a live element; a failure, or no child, gives none.
        unsafe { self.0.GetFirstChildElement(node) }.ok()
    }

    fn next(&self, node: &IUIAutomationElement) -> Option<IUIAutomationElement> {
        // SAFETY: as above, for the next sibling.
        unsafe { self.0.GetNextSiblingElement(node) }.ok()
    }

    fn holds(&self, node: &IUIAutomationElement, p: Point) -> bool {
        // SAFETY: a plain property read on a live element; a failed read holds nothing.
        let edges = unsafe { node.CurrentBoundingRectangle() };
        edges.is_ok_and(|r| contains(rect_of([r.left, r.top, r.right, r.bottom]), p))
    }
}

#[cfg(test)]
mod tests;
