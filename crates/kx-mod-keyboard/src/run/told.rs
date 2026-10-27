//! What typing tells word prediction (ADR-0033): each piece that reached the app waits here, in order, under the
//! face's lock, until the face takes and tells it.

use super::Typing;
use crate::plan::{Route, Wait};
use crate::spec::Tell;
use kx_module_api::ports::Piece;

impl Typing {
    /// Takes the pieces that reached the app since the last take, oldest first.
    pub fn take_pieces(&mut self) -> Vec<Piece> {
        std::mem::take(&mut self.pieces)
    }

    /// Notes `piece` as reached the app, such as a language switch's break.
    pub(crate) fn tell(&mut self, piece: Piece) {
        self.pieces.push(piece);
    }
}

/// What a send of `route` with `wait` tells, for a key that tells `tell` when it goes out alone: text sent as text,
/// the key's own piece, or a break for a chord, a key the app composes with a waiting dead key, or one only the app's
/// layout reads.
pub(super) fn piece(route: &Route, wait: Wait, tell: Option<&Tell>) -> Piece {
    match (route, tell) {
        (Route::Text(text, _), _) => Piece::Text(text.clone()),
        (Route::Tap(_), Some(tell)) if wait != Wait::Ask => tell.piece(),
        _ => Piece::Break,
    }
}
