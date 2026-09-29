//! Boot and start: failures stay contained, the plan fails what it must, grants gate services.

use crate::record::{Did, Mode};
use crate::rig::{Rig, failed};
use crate::samples::{
    Alpha, BROKEN_SPEC, Beta, Gamma, LEVEL_SPEC, Mic, PLATFORM, Ping, sample, service,
};
use kx_kernel::grants::Policy;
use kx_kernel::{KERNEL, KernelError};
use kx_module_api::Capability::Microphone;
use kx_module_api::ModuleState::{Active, Failed, Starting};
use kx_module_api::{ModuleId, ServiceError, ServiceKey};

const A: ModuleId = ModuleId::new("a");
const B: ModuleId = ModuleId::new("b");
const C: ModuleId = ModuleId::new("c");

#[test]
fn a_failed_start_is_contained_and_rolled_back() {
    let mut rig = Rig::new();
    rig.add(sample(A).provides(&[Alpha::ID]).mode(Mode::Fail));
    rig.add(sample(B));
    let notices = rig.kernel.boot().unwrap();
    assert_eq!([A, B].map(|m| rig.state(m)), [Some(Failed), Some(Active)]);
    let moves = [(A, Starting), (A, Failed), (B, Starting), (B, Active)];
    assert_eq!(rig.moves(), moves, "every state change is on the bus");
    assert_eq!(notices, [failed(A)]);
    assert!(
        rig.log.all().contains(&Did::Dropped(Alpha::ID)),
        "its service left the registry"
    );
    rig.kernel.bus().publish(&Ping);
    assert_eq!(rig.log.pings(), [B], "its subscription is gone");
}

#[test]
fn a_cycle_fails_only_its_members() {
    let mut rig = Rig::new();
    rig.add(sample(A).requires(&[Beta::ID]).provides(&[Alpha::ID]));
    rig.add(sample(B).requires(&[Alpha::ID]).provides(&[Beta::ID]));
    rig.add(sample(C));
    let notices = rig.kernel.boot().unwrap();
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(states, [Some(Failed), Some(Failed), Some(Active)]);
    assert_eq!(notices, [failed(A), failed(B)]);
    assert_eq!(rig.log.starts(), [C], "cycle members never run");
    assert!(matches!(
        rig.kernel.retry(A),
        Err(KernelError::NotStartable(A))
    ));
}

#[test]
fn a_missing_service_fails_the_module_and_its_dependents() {
    let mut rig = Rig::new();
    rig.add(sample(A).requires(&[Gamma::ID]).provides(&[Alpha::ID]));
    rig.add(sample(B).requires(&[Alpha::ID]));
    rig.add(sample(C));
    let notices = rig.kernel.boot().unwrap();
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(states, [Some(Failed), Some(Failed), Some(Active)]);
    assert_eq!(notices, [failed(A), failed(B)]);
    assert_eq!(rig.log.starts(), [C]);
}

#[test]
fn an_ungranted_capability_is_refused_and_fails_a_module_that_needs_it() {
    let mut rig = Rig::with(Policy::new().allow(C, &[Microphone]));
    let mic = service(Mic::ID, PLATFORM, &rig.log);
    rig.kernel.provide_platform::<Mic>(mic).unwrap();
    let asks = |id| sample(id).requires(&[Mic::ID]).caps(&[Microphone]);
    rig.add(asks(A));
    rig.add(asks(B).tolerant());
    rig.add(asks(C));
    rig.kernel.boot().unwrap();
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(states, [Some(Failed), Some(Active), Some(Active)]);
    let log = rig.log.all();
    let refused = ServiceError::NotGranted(Mic::ID, Microphone);
    assert!(log.contains(&Did::Refused(B, refused)), "{log:?}");
    assert!(log.contains(&Did::Got(C, PLATFORM)), "{log:?}");
}

#[test]
fn a_second_provider_fails_and_the_first_serves() {
    let mut rig = Rig::new();
    rig.add(sample(A).provides(&[Alpha::ID]));
    rig.add(sample(B).provides(&[Alpha::ID]));
    rig.add(sample(C).requires(&[Alpha::ID]));
    let notices = rig.kernel.boot().unwrap();
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(states, [Some(Active), Some(Failed), Some(Active)]);
    assert_eq!(notices, [failed(B)]);
    assert!(rig.log.all().contains(&Did::Got(C, A.as_str())));
}

#[test]
fn a_module_providing_a_platform_service_fails_and_consumers_get_the_platform_one() {
    let mut rig = Rig::new();
    let alpha = service(Alpha::ID, PLATFORM, &rig.log);
    rig.kernel.provide_platform::<Alpha>(alpha).unwrap();
    let again = rig
        .kernel
        .provide_platform::<Alpha>(service(Alpha::ID, PLATFORM, &rig.log));
    let taken = ServiceError::AlreadyProvided(Alpha::ID);
    assert!(matches!(again, Err(KernelError::Service(e)) if e == taken));
    rig.add(sample(A).provides(&[Alpha::ID]));
    rig.add(sample(B).requires(&[Alpha::ID]));
    let notices = rig.kernel.boot().unwrap();
    assert_eq!([A, B].map(|m| rig.state(m)), [Some(Failed), Some(Active)]);
    assert_eq!(notices, [failed(A)]);
    assert!(rig.log.all().contains(&Did::Got(B, PLATFORM)));
}

#[test]
fn add_refuses_a_repeated_id_and_the_kernel_id() {
    let mut rig = Rig::new();
    rig.add(sample(A));
    let twice = rig.kernel.add(sample(A).mode(Mode::Fail).build(&rig.log));
    assert!(matches!(twice, Err(KernelError::Duplicate(A))));
    let kernel = rig.kernel.add(sample(KERNEL).build(&rig.log));
    assert!(matches!(kernel, Err(KernelError::Reserved(KERNEL))));
    rig.kernel.boot().unwrap();
    assert_eq!(rig.log.starts(), [A], "only the first one was added");
    assert_eq!(rig.state(A), Some(Active));
    assert_eq!(rig.state(KERNEL), None);
}

#[test]
fn a_broken_settings_spec_fails_that_module_alone() {
    let mut rig = Rig::new();
    rig.add(sample(A).settings(&BROKEN_SPEC));
    rig.add(sample(B).settings(&LEVEL_SPEC));
    rig.add(sample(C));
    let notices = rig.kernel.boot().unwrap();
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(states, [Some(Failed), Some(Active), Some(Active)]);
    assert_eq!(notices, [failed(A)]);
    assert_eq!(rig.log.starts(), [B, C]);
    assert!(matches!(
        rig.kernel.retry(A),
        Err(KernelError::NotStartable(A))
    ));
}
