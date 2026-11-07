//! Properties of the fake box's bytes: any known parts, at least one and each once, seal and open whole, and no bytes
//! after a box's head open or panic.

use super::*;
use kx_module_api::ports::BOX_PARTS;
use proptest::prelude::*;

proptest! {
    #[test]
    fn any_known_parts_seal_and_open_whole(
        names in prop::sample::subsequence(BOX_PARTS.to_vec(), 1..=BOX_PARTS.len()),
        blobs in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..64), BOX_PARTS.len()),
    ) {
        let (_, mut run) = session();
        let padlock = run.padlock(&password(0), WORK).unwrap();
        let parts: Vec<_> = names.iter().zip(&blobs).map(|(name, bytes)| part(name, bytes)).collect();
        let sealed = run.seal(&padlock, &parts).unwrap();
        let opened = run.open(sealed.expose(), &password(0)).map(|o| o.parts);
        prop_assert_eq!(opened, Ok(parts));
    }

    #[test]
    fn bytes_after_a_box_head_never_open_and_never_panic(
        tail in prop::collection::vec(any::<u8>(), 0..256),
        headed in any::<bool>(),
    ) {
        let (_, mut run) = session();
        let head = if headed { [&BOX_MARK[..], &[BOX_VERSION], format::TAG].concat() } else { Vec::new() };
        prop_assert!(run.open(&[head, tail].concat(), &password(0)).is_err());
    }
}
