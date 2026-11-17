//! The folder reveal's grant (ADR-0046): only the voice module may hold `OpenFolder` or ask for the reveal, which
//! reaches only that gate's holders.

use super::{asking, holders, reached};
use crate::rig::Rig;
use kx_module_api::ports::{FolderReveal, FolderRevealKey};
use kx_module_api::{Capability, ModuleId, ServiceKey};
use kx_platform_fake::FakeFolderReveal;
use std::collections::BTreeSet;
use std::sync::Arc;

/// The one module the gate is for.
const VOICE: ModuleId = ModuleId::new("voice");

#[test]
fn only_the_voice_module_may_hold_open_folder_or_ask_for_the_reveal() {
    let voice_only = |ids: BTreeSet<ModuleId>| ids.iter().all(|id| *id == VOICE);
    assert!(voice_only(holders(Capability::OpenFolder)), "ADR-0046");
    assert!(voice_only(asking(&[FolderRevealKey::ID])));
}

#[test]
fn the_folder_reveal_reaches_only_the_open_folder_holders() {
    let reveal: Arc<dyn FolderReveal> = Arc::new(FakeFolderReveal::new());
    let got = reached::<FolderRevealKey>(&Rig::new(), &reveal);
    assert_eq!(got, holders(Capability::OpenFolder));
}
