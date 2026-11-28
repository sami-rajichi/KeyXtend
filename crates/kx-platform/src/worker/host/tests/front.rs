//! A host of dialogs lets its worker take the foreground before each ask; a plain host never does.

use super::*;

#[test]
fn a_host_of_dialogs_grants_its_worker_the_foreground_before_each_ask_and_others_never() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut plain = host(&script, &clock);
    assert_eq!(ping(&mut plain), Ok(1));
    assert!(
        !script.calls().contains(&Call::Front),
        "{:?}",
        script.calls()
    );
    drop(plain);
    let (script, clock) = (Script::default(), Hands::default());
    let mut dialogs = host(&script, &clock).fronted();
    assert_eq!(ping(&mut dialogs), Ok(1));
    assert_eq!(ping(&mut dialogs), Ok(2));
    let calls = [
        Call::Front,
        asked(1),
        reply(),
        peek(),
        Call::Front,
        asked(2),
        reply(),
    ];
    assert_eq!(script.calls(), calls);
}
