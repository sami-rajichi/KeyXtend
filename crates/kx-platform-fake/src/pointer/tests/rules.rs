//! The fake keeps the real host's rules: a feed settles only when the host would wake, a let-go keeps the run, and a
//! given-up engine drops while its run stays.

use super::*;

/// What the still engine does, and how often it settled.
#[derive(Clone, Debug, Default)]
struct Plan {
    /// `on` pushes one act.
    push: bool,
    /// `on` sets the deadline here.
    arm: Option<Mono>,
    /// The engine holds notes until it settles.
    notes: bool,
    due: Option<Mono>,
    settles: usize,
    /// The host dropped the engine.
    dropped: bool,
}

/// An engine that drops every event and does only what its plan says.
struct Still(Arc<Mutex<Plan>>);

impl Still {
    fn plan(&self) -> std::sync::MutexGuard<'_, Plan> {
        lock(&self.0)
    }
}

impl PointerEngine for Still {
    fn on(&mut self, _: PointerEvent, _: Mono, acts: &mut Vec<PointerAct>) -> bool {
        let mut plan = self.plan();
        if plan.push {
            acts.push(SEEN);
        }
        plan.due = plan.arm.or(plan.due);
        true
    }

    fn settle(&mut self, _: Mono, _: &mut Vec<PointerAct>) {
        let mut plan = self.plan();
        plan.settles += 1;
        plan.notes = false;
    }

    fn deadline(&self) -> Option<Mono> {
        self.plan().due
    }

    fn has_notes(&self) -> bool {
        self.plan().notes
    }

    fn release(&mut self, acts: &mut Vec<PointerAct>) {
        acts.push(LET_GO);
    }

    fn max_acts(&self) -> usize {
        [SEEN].len()
    }
}

impl Drop for Still {
    fn drop(&mut self) {
        self.plan().dropped = true;
    }
}

/// A started hook on a still engine that follows `plan`, and that plan.
fn started(plan: Plan) -> (FakePointerHook, Box<dyn HookRun>, Arc<Mutex<Plan>>) {
    let hook = FakePointerHook::new();
    let plan = Arc::new(Mutex::new(plan));
    let run = hook.start(Box::new(Still(Arc::clone(&plan))), CONFIG);
    (hook, run.unwrap(), plan)
}

/// A plan whose `on` pushes an act, sets the deadline to `arm` and holds notes as given.
fn plan(push: bool, arm: Option<Mono>, notes: bool) -> Plan {
    Plan {
        push,
        arm,
        notes,
        ..Plan::default()
    }
}

#[test]
fn a_feed_settles_only_when_the_host_would_wake() {
    let cases = [
        (plan(false, None, false), 0),
        (plan(true, None, false), 1),
        (plan(false, Some(T2), false), 1),
        (plan(false, None, true), 1),
    ];
    for (given, settles) in cases {
        let (hook, _run, kept) = started(given.clone());
        let _ = hook.feed(MOVE, T1);
        assert_eq!(lock(&kept).settles, settles, "{given:?}");
    }
}

#[test]
fn a_given_up_engine_lets_go_once_and_drops_while_its_run_stays() {
    let (hook, run, kept) = started(Plan::default());
    hook.give_up();
    assert_eq!(hook.take_sent(), [LET_GO], "its one release");
    assert!(lock(&kept).dropped, "dropped at once");
    assert!(hook.running(), "the run stays until the module drops it");
    assert_eq!(hook.start(probe().0, CONFIG).err(), Some(HookError::Busy));
    assert!(!hook.feed(MOVE, T1), "events pass untouched");
    hook.settle(T1);
    assert_eq!((hook.deadline(), lock(&kept).settles), (None, 0));
    drop(run);
    assert!(hook.sent().is_empty(), "no second release");
    assert_eq!((hook.stops(), hook.running()), (1, false));
}

#[test]
fn a_let_go_keeps_the_run_and_settles_as_the_hosts_turn_would() {
    let (hook, run, kept) = started(Plan::default());
    hook.let_go(T1);
    assert_eq!(hook.take_sent(), [LET_GO]);
    assert_eq!(lock(&kept).settles, 1, "the turn the host asks for");
    assert!(hook.running() && !lock(&kept).dropped, "the run goes on");
    assert!(hook.feed(MOVE, T2), "the engine still decides");
    drop(run);
    assert_eq!(hook.take_sent(), [LET_GO], "the stop lets go again");
}
