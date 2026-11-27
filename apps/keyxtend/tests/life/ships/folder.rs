//! The folder reveal's grant: only voice typing (ADR-0046) and Snip, for Open folder and Play (ADR-0056), may hold
//! `OpenFolder` or ask for the reveal, which reaches only that gate's holders.

use super::{asking, holders, reached};
use crate::rig::Rig;
use kx_module_api::ports::{FolderReveal, FolderRevealKey};
use kx_module_api::{Capability, ModuleId, ServiceKey};
use kx_platform_fake::FakeFolderReveal;
use std::collections::BTreeSet;
use std::sync::Arc;

/// The two modules the gate is for.
const OPENERS: [ModuleId; 2] = [kx_mod_voice::ID, kx_mod_snip::ID];

#[test]
fn only_voice_typing_and_snip_hold_open_folder_and_ask_for_the_reveal() {
    let openers = BTreeSet::from(OPENERS);
    assert_eq!(
        holders(Capability::OpenFolder),
        openers,
        "ADR-0046, ADR-0056"
    );
    assert_eq!(asking(&[FolderRevealKey::ID]), openers);
}

#[test]
fn the_folder_reveal_reaches_only_the_open_folder_holders() {
    let reveal: Arc<dyn FolderReveal> = Arc::new(FakeFolderReveal::new());
    let got = reached::<FolderRevealKey>(&Rig::new(), &reveal);
    assert_eq!(got, holders(Capability::OpenFolder));
}
