//! The start and the stop: the watch and the picture session follow them, a missing gate or a refused watch fails
//! the start, a locked shelf fails it until a try again (edge 15), and a newer list fails it untouched.

use kx_mod_clipboard::{ClipSettings, ClipboardModule, MANIFEST};
use kx_module_api::ports::{
    BlobName, CalendarKey, ClipError, ClipboardKey, ClipboardWriterKey, PicturesKey, SealedError,
    SealedShelfKey,
};
use kx_module_api::{Module, ModuleError, ServiceError, ServiceId, ServiceKey};
use kx_platform_fake::ShelfCall;

use crate::rig::{Fakes, Heard, LIST, TODAY, start};

/// The shelf calls that change it.
const WRITES: [ShelfCall; 3] = [ShelfCall::Save, ShelfCall::Remove, ShelfCall::DropFolder];

#[test]
fn the_start_watches_and_opens_pictures_by_the_settings_and_the_stop_lets_go() {
    let (fakes, mut module) = (Fakes::new(), ClipboardModule::default());
    let mut host = fakes.host(MANIFEST.capabilities);
    let settings = ClipSettings::from_defaults().unwrap();
    start(&mut module, &mut host).unwrap();
    assert!(fakes.clip.watching());
    assert_eq!(fakes.clip.config(), Some(settings.clip_config()));
    assert_eq!(fakes.pics.config(), Some(settings.picture_config()));
    module.stop();
    assert!(!fakes.clip.watching() && !fakes.pics.running());
    start(&mut module, &mut host).unwrap();
    assert_eq!((fakes.clip.watches(), fakes.pics.opens()), (2, 2));
    assert!(fakes.clip.watching() && fakes.pics.running());
}

#[test]
fn each_missing_gate_fails_the_start_and_watches_nothing() {
    let gated: [(ServiceId, _); 5] = [
        (ClipboardKey::ID, ClipboardKey::CAPABILITY),
        (ClipboardWriterKey::ID, ClipboardWriterKey::CAPABILITY),
        (SealedShelfKey::ID, SealedShelfKey::CAPABILITY),
        (PicturesKey::ID, PicturesKey::CAPABILITY),
        (CalendarKey::ID, CalendarKey::CAPABILITY),
    ];
    for &cap in MANIFEST.capabilities {
        let fakes = Fakes::new();
        let left: Vec<_> = MANIFEST
            .capabilities
            .iter()
            .copied()
            .filter(|c| *c != cap)
            .collect();
        let mut host = fakes.host(&left);
        let (id, _) = gated
            .iter()
            .find(|(_, c)| *c == Some(cap))
            .copied()
            .unwrap();
        let refused = start(&mut ClipboardModule::default(), &mut host);
        assert_eq!(
            refused,
            Err(ModuleError::Service(ServiceError::NotGranted(id, cap)))
        );
        assert_eq!(fakes.clip.watches(), 0);
    }
}

#[test]
fn a_locked_shelf_fails_the_start_with_no_notice_of_its_own_and_a_try_again_starts() {
    let (fakes, mut module) = (Fakes::new(), ClipboardModule::default());
    fakes.saved("kept");
    fakes.shelf.fail(ShelfCall::Load, SealedError::Locked);
    let mut host = fakes.host(MANIFEST.capabilities);
    let heard = Heard::listen(&host);
    assert!(matches!(
        start(&mut module, &mut host),
        Err(ModuleError::Start(_))
    ));
    assert!(
        heard.notices().is_empty(),
        "the kernel tells a failed start, not the module"
    );
    assert_eq!(fakes.clip.watches(), 0, "nothing ran");
    start(&mut module, &mut host).unwrap();
    assert!(fakes.clip.watching());
}

#[test]
fn a_newer_versions_list_fails_the_start_and_changes_nothing() {
    let (fakes, mut module) = (Fakes::new(), ClipboardModule::default());
    fakes.saved("kept");
    let (day, list) = (BlobName::new(TODAY).unwrap(), BlobName::new(LIST).unwrap());
    let mut head = fakes.shelf.blob(&day, &list).unwrap();
    head[0] += 1;
    fakes.shelf.plant(&day, &list, head.clone());
    let before = WRITES.map(|c| fakes.shelf.calls(c));
    let mut host = fakes.host(MANIFEST.capabilities);
    assert!(matches!(
        start(&mut module, &mut host),
        Err(ModuleError::Start(_))
    ));
    assert_eq!(WRITES.map(|c| fakes.shelf.calls(c)), before);
    assert_eq!(fakes.shelf.blob(&day, &list), Some(head));
    assert_eq!(fakes.clip.watches(), 0);
}

#[test]
fn a_refused_watch_fails_the_start_before_any_thread() {
    let (fakes, mut module) = (Fakes::new(), ClipboardModule::default());
    fakes.clip.fail_watch(ClipError::Refused);
    let mut host = fakes.host(MANIFEST.capabilities);
    assert!(matches!(
        start(&mut module, &mut host),
        Err(ModuleError::Start(_))
    ));
    assert_eq!(fakes.pics.opens(), 0);
}
