//! The start tells the shown layout once, each change tells the language in front, and an ask tells it again, so any
//! module start order works.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::common::{ids, service, source};
use kx_mod_layouts::{LayoutsModule, MANIFEST};
use kx_module_api::ports::{LayoutAsk, LayoutChanged, LayoutSource, LayoutSourceKey, Layouts};
use kx_module_api::{Bus, Module, ModuleCx, ServiceKey};
use kx_test_support::host::{FakeHost, Stored};
use std::sync::{Arc, Mutex};

/// The change the service tells when the golden Arabic layout shows.
fn arabic() -> LayoutChanged {
    LayoutChanged {
        id: ids()[2],
        iso: "ar".into(),
        rtl: true,
    }
}

#[test]
fn a_change_tells_the_language_and_its_direction() {
    let fake = source();
    let (layouts, heard) = service(&fake);
    assert_eq!(layouts.pick(ids()[2]), Ok(()));
    assert!(layouts.refresh());
    assert_eq!(heard.changes(), [arabic()]);
}

#[test]
fn tell_says_the_shown_layout_again_and_nothing_with_none_installed() {
    let fake = source();
    let (layouts, heard) = service(&fake);
    layouts.tell();
    let english = LayoutChanged::from(&layouts.current().label);
    assert_eq!((english.iso.as_str(), english.rtl), ("en", false));
    assert_eq!(heard.changes(), [english]);
    for id in ids() {
        fake.remove(id);
    }
    assert!(layouts.refresh());
    layouts.tell();
    assert_eq!(heard.changes().len(), 1, "no layout, nothing told");
}

#[test]
fn the_start_tells_the_shown_layout_once_and_an_ask_tells_it_again() {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = MANIFEST.capabilities.to_vec();
    let source: Arc<dyn LayoutSource> = source();
    let stored: Stored = (LayoutSourceKey::CAPABILITY, Box::new(source));
    host.services.insert(LayoutSourceKey::ID, stored);
    let bus = Bus::new(Arc::clone(&host.core));
    let heard = Arc::new(Mutex::new(Vec::new()));
    let keep = Arc::clone(&heard);
    let _ear = bus
        .subscribe(move |e: &LayoutChanged| keep.lock().unwrap().push(e.clone()))
        .unwrap();
    LayoutsModule.start(&mut ModuleCx::new(&mut host)).unwrap();
    let english = LayoutChanged {
        id: ids()[0],
        iso: "en".into(),
        rtl: false,
    };
    let told = || heard.lock().unwrap().clone();
    assert_eq!(
        told(),
        std::slice::from_ref(&english),
        "a start tells it once"
    );
    bus.publish(&LayoutAsk).unwrap();
    assert_eq!(told(), [english.clone(), english], "an ask tells it again");
    host.held.clear();
    bus.publish(&LayoutAsk).unwrap();
    assert_eq!(told().len(), 2, "a stopped module hears none");
}
