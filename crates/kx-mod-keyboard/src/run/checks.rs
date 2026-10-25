//! Checks around a send: whether any of it reached the app, whether the app's own layout decides it, and whether it
//! typed anything in the layout shown.

use super::Io;
use crate::plan::{Plan, Route};
use crate::spec::Role;
use kx_module_api::ports::InjectError;

/// True when any of a send reached the app: all of it, or the part the system took before refusing the rest.
pub(super) fn reached(went: Result<(), InjectError>) -> bool {
    match went {
        Ok(()) => true,
        Err(InjectError::Refused { sent, .. }) => sent > 0,
        Err(InjectError::Unmappable) => false,
    }
}

/// True when `plan` sends a character key as a scan code, so the app's own layout decides what it types.
pub(super) fn keyed(role: Role, plan: &Plan) -> bool {
    let scan = |p: &Plan| {
        matches!(
            p,
            Plan::Send {
                route: Route::Tap(_) | Route::Chord(..),
                ..
            }
        )
    };
    role == Role::Char && scan(plan)
}

/// True when the app in front has another installed layout than the table shown, as `Follow` would show
/// after a refresh; with none in front, or one that is not installed, the table shown stands.
pub(super) fn stale(io: &Io<'_>) -> bool {
    let shown = io.table.label.id;
    let other = io.source.foreground().filter(|&front| front != shown);
    other.is_some_and(|front| io.source.installed().contains(&front))
}

/// True when `route` typed anything in the layout shown, as the layout source reads it.
pub(super) fn types(io: &Io<'_>, route: &Route) -> bool {
    let layout = io.table.label.id;
    match route {
        Route::Tap(key) => io.source.types_any(layout, *key, &[]),
        Route::Chord(held, key) => io.source.types_any(layout, *key, held),
        Route::Text(..) => true,
    }
}
