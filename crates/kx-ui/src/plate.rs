//! What the plate shows after the `Backdrop` port was tried: see-through over a material, else solid.

use kx_platform::ui::{BackdropKind, UiError};

/// What the face should paint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plate {
    /// A solid colour.
    Solid,
    /// See-through, since the system took the backdrop; G13 checks by eye that it really shows.
    SeeThrough,
}

/// The plate after trying backdrop `kind`; any refusal leaves it solid.
pub fn plate_after(kind: BackdropKind, tried: Result<(), UiError>) -> Plate {
    match (kind, tried) {
        (BackdropKind::None, _) | (_, Err(_)) => Plate::Solid,
        _ => Plate::SeeThrough,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refused_backdrop_leaves_a_solid_plate() {
        let see = plate_after(BackdropKind::Acrylic, Ok(()));
        assert_eq!(see, Plate::SeeThrough);
        let refused = plate_after(BackdropKind::Mica, Err(UiError::Refused(1)));
        assert_eq!(refused, Plate::Solid);
        let none = plate_after(BackdropKind::None, Ok(()));
        assert_eq!(none, Plate::Solid, "no material, no see-through");
    }
}
