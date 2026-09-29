//! Port services through the kernel: the injector and the layouts reach a module only with `InjectInput`.

#![allow(
    clippy::unwrap_used,
    reason = "these fixture helpers run only in tests, where a failed setup should fail the test"
)]

use crate::rig::{Rig, failed};
use kx_kernel::Policy;
use kx_module_api::Capability::InjectInput;
use kx_module_api::ModuleState::{Active, Failed};
use kx_module_api::ports::{
    InputInjectorKey, LangLabel, LayoutId, LayoutTable, Layouts, LayoutsKey, Scan, SwitchError,
};
use kx_module_api::{
    Capability, Manifest, Module, ModuleCx, ModuleError, ModuleId, ServiceError, ServiceId,
    ServiceKey,
};
use kx_platform_fake::{FakeInjector, Stroke};
use std::sync::{Arc, Mutex};

const LAYOUTS: ModuleId = ModuleId::new("layouts");
const TYPIST: ModuleId = ModuleId::new("typist");
const INJECT: &[Capability] = &[InjectInput];
const NEEDS: &[ServiceId] = &[InputInjectorKey::ID, LayoutsKey::ID];
const KEY: Scan = Scan::new(0x1E);
const SHOWN: LayoutId = LayoutId::new(0x0409_0409);
const TAP_FAILED: &str = "tap failed";

/// A layouts service with one table.
struct OneLayout(Arc<LayoutTable>);

impl Layouts for OneLayout {
    fn current(&self) -> Arc<LayoutTable> {
        Arc::clone(&self.0)
    }

    fn list(&self) -> Vec<LangLabel> {
        vec![self.0.label.clone()]
    }

    fn refresh(&self) -> bool {
        false
    }

    fn step(&self, _by: i32) -> Result<(), SwitchError> {
        Ok(())
    }

    fn pick(&self, _id: LayoutId) -> Result<(), SwitchError> {
        Ok(())
    }
}

/// What the typist saw at start: each request's refusal, and the layout it read.
#[derive(Default)]
struct Seen {
    refused: Vec<Option<ServiceError>>,
    layout: Option<LayoutId>,
}

/// A module that provides the layouts, or one that taps a key through both port services.
enum Port {
    Provider,
    Typist(Arc<Mutex<Seen>>),
}

/// A port module under `manifest`.
struct PortModule(&'static Manifest, Port);

impl Module for PortModule {
    fn manifest(&self) -> &'static Manifest {
        self.0
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        match &self.1 {
            Port::Provider => {
                let label = LangLabel {
                    id: SHOWN,
                    ..LangLabel::default()
                };
                let table = Arc::new(LayoutTable {
                    label,
                    ..LayoutTable::default()
                });
                Ok(cx.provide::<LayoutsKey>(Arc::new(OneLayout(table)))?)
            }
            Port::Typist(seen) => type_one_key(cx, seen),
        }
    }
}

/// Gets both services, notes what it saw, then taps `KEY`.
fn type_one_key(cx: &ModuleCx<'_>, seen: &Mutex<Seen>) -> Result<(), ModuleError> {
    let injector = cx.service::<InputInjectorKey>();
    let layouts = cx.service::<LayoutsKey>();
    let mut seen = seen.lock().unwrap();
    seen.refused = vec![
        injector.as_ref().err().copied(),
        layouts.as_ref().err().copied(),
    ];
    seen.layout = layouts.as_ref().ok().map(|l| l.current().label.id);
    drop(seen);
    injector?
        .tap(KEY)
        .map_err(|_| ModuleError::Start(TAP_FAILED))?;
    layouts.map(drop).map_err(ModuleError::from)
}

/// A manifest that lives for the whole test.
fn manifest(
    id: ModuleId,
    requires: &'static [ServiceId],
    provides: &'static [ServiceId],
) -> &'static Manifest {
    Box::leak(Box::new(Manifest {
        id,
        version: env!("CARGO_PKG_VERSION"),
        requires,
        provides,
        capabilities: INJECT,
        settings: None,
    }))
}

/// A rig whose platform injects, with the layouts provider and the typist added.
fn rig(policy: Policy) -> (Rig, Arc<FakeInjector>, Arc<Mutex<Seen>>) {
    let mut rig = Rig::with(policy);
    let injector = Arc::new(FakeInjector::new());
    rig.kernel
        .provide_platform::<InputInjectorKey>(injector.clone())
        .unwrap();
    let provider = PortModule(manifest(LAYOUTS, &[], &[LayoutsKey::ID]), Port::Provider);
    let seen = Arc::new(Mutex::new(Seen::default()));
    let typist = PortModule(
        manifest(TYPIST, NEEDS, &[]),
        Port::Typist(Arc::clone(&seen)),
    );
    rig.kernel.add(Box::new(provider)).unwrap();
    rig.kernel.add(Box::new(typist)).unwrap();
    (rig, injector, seen)
}

#[test]
fn with_inject_input_a_module_gets_the_injector_and_the_layouts() {
    let policy = Policy::new().allow(LAYOUTS, INJECT).allow(TYPIST, INJECT);
    let (mut rig, injector, seen) = rig(policy);
    assert!(rig.kernel.boot().unwrap().is_empty());
    assert_eq!([LAYOUTS, TYPIST].map(|m| rig.state(m)), [Some(Active); 2]);
    let seen = seen.lock().unwrap();
    assert_eq!(
        (seen.refused.as_slice(), seen.layout),
        (&[None, None][..], Some(SHOWN))
    );
    assert_eq!(injector.strokes(), [Stroke::Tap(KEY)]);
}

#[test]
fn without_inject_input_both_are_refused_and_the_module_fails() {
    let policy = Policy::new().allow(LAYOUTS, INJECT);
    let (mut rig, injector, seen) = rig(policy);
    assert_eq!(rig.kernel.boot().unwrap(), [failed(TYPIST)]);
    assert_eq!(
        [LAYOUTS, TYPIST].map(|m| rig.state(m)),
        [Some(Active), Some(Failed)]
    );
    let refused = |key| Some(ServiceError::NotGranted(key, InjectInput));
    let want = [refused(InputInjectorKey::ID), refused(LayoutsKey::ID)];
    assert_eq!(seen.lock().unwrap().refused, want);
    assert!(injector.strokes().is_empty());
}
