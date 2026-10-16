//! Tests of the speaker thread over the fake system: its queue never blocks the caller, and its stop tears down.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use kx_module_api::{Clock, Mono};
use kx_platform::ui::Speaker;

use super::fake::{Call, Part, World};
use super::{DROP, KEY, LIMIT, clip, config};
use crate::sound::Actor;
use crate::sound::actor::QUEUE;

/// A clock that never moves, so the thread never stops its engine.
struct Still;

impl Clock for Still {
    fn now(&self) -> Mono {
        Mono::default()
    }
}

/// A speaker over `world` whose drop waits for its thread as long as a test may.
fn actor(world: &World) -> Actor {
    let world = world.clone();
    let mut speaker = Actor::spawn(move || world, Arc::new(Still));
    speaker.stop_wait(LIMIT);
    speaker
}

/// Hears each play that reaches `world`.
fn heard(world: &World) -> Receiver<()> {
    let (heard, each) = mpsc::channel();
    world.state().heard = Some(heard);
    each
}

/// Waits until `n` plays were heard.
fn hear(each: &Receiver<()>, n: usize) {
    for _ in 0..n {
        each.recv_timeout(LIMIT).expect("the play is heard");
    }
}

/// How many plays reached the fake.
fn plays(world: &World) -> usize {
    let calls = world.take().into_iter();
    calls.filter(|c| matches!(c, Call::Play(..))).count()
}

/// A speaker whose thread is blocked making the engine for a play of the key clip.
struct Blocked {
    speaker: Actor,
    world: World,
    /// Opens the gate, so the thread goes on.
    open: Sender<()>,
}

fn blocked() -> Blocked {
    let world = World::default();
    let ((entered, inside), (open, gate)) = (mpsc::channel(), mpsc::channel());
    world.state().gate = Some((entered, gate));
    let speaker = actor(&world);
    speaker.prepare(vec![clip(KEY)], config(1));
    speaker.play(KEY, 1.0);
    inside
        .recv_timeout(LIMIT)
        .expect("the thread blocks making the engine");
    Blocked {
        speaker,
        world,
        open,
    }
}

#[test]
fn play_never_waits_and_a_full_queue_drops_plays_but_keeps_a_prepare() {
    let Blocked {
        speaker,
        world,
        open,
    } = blocked();
    let each = heard(&world);
    let speaker = Arc::new(speaker);
    let (caller, (done, returned)) = (Arc::clone(&speaker), mpsc::channel());
    // The key clip alone is prepared, so the drops play only if the later prepare survives the full queue.
    let calls = std::thread::spawn(move || {
        (0..QUEUE + 3).for_each(|_| caller.play(DROP, 1.0));
        caller.prepare(vec![clip(KEY), clip(DROP)], config(1));
        let _ = done.send(());
    });
    let answer = returned.recv_timeout(LIMIT);
    let _ = open.send(());
    assert!(answer.is_ok(), "a call waited on the blocked thread");
    calls.join().expect("the caller ends");
    hear(&each, QUEUE + 1);
    drop(speaker);
    let want = QUEUE + 1;
    assert_eq!(plays(&world), want, "the plays past the queue were dropped");
}

#[test]
fn a_stuck_thread_is_left_after_the_stop_wait() {
    let Blocked {
        mut speaker,
        world,
        open,
    } = blocked();
    speaker.stop_wait(Duration::ZERO);
    let (done, returned) = mpsc::channel();
    let stopper = std::thread::spawn(move || {
        drop(speaker);
        let _ = done.send(());
    });
    let answer = returned.recv_timeout(LIMIT);
    let calls = world.take();
    let _ = open.send(());
    assert!(answer.is_ok(), "the drop waited on the stuck thread");
    assert!(calls.is_empty(), "nothing was built or torn down yet");
    stopper.join().expect("the drop ends");
}

#[test]
fn dropping_the_speaker_destroys_the_voices_then_the_master_then_the_engine() {
    let world = World::default();
    let each = heard(&world);
    let speaker = actor(&world);
    speaker.prepare(vec![clip(KEY), clip(DROP)], config(2));
    speaker.play(KEY, 1.0);
    hear(&each, 1);
    drop(speaker);
    let calls = world.take().into_iter();
    let gone: Vec<Call> = calls.filter(|c| matches!(c, Call::Gone(_))).collect();
    let parts = (1..=4).map(Part::Voice).chain([Part::Master, Part::Engine]);
    assert_eq!(gone, parts.map(Call::Gone).collect::<Vec<_>>());
}
