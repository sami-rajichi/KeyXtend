//! Asks and calls handed to the stream without waiting: a `tell` posts on its own stream and never hangs, and a later
//! call runs on the stream's thread after the frames it already holds.

use super::*;

#[test]
fn a_tell_that_posts_on_its_own_stream_sends_the_ask_and_never_hangs() {
    let script = Script::default();
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    let shared = Shared::default();
    let own = Arc::clone(&shared);
    let tell = move |told: Told<VoiceNote>| {
        let held = own.lock().unwrap_or_else(PoisonError::into_inner);
        if let (Some(stream), Told::Note(note)) = (held.as_ref(), told) {
            stream.post(stop(note.id()));
        }
    };
    let (spawner, clock) = (Arc::new(script.clone()), Arc::new(Hands::default()));
    let stream = Stream::open(THREAD, spawner, clock, TIMES, launch(), tell);
    script.act(VoiceAct::Say(vec![open]));
    let mut slot = shared.lock().unwrap_or_else(PoisonError::into_inner);
    assert_eq!(slot.insert(stream).begin(start, Wait::Off), Ok(FIRST));
    drop(slot);
    let posted = Call::Voice(stop(FIRST));
    while called.recv_timeout(LONG).expect("the post is sent") != posted {}
    let stream = shared.lock().unwrap_or_else(PoisonError::into_inner).take();
    drop(stream);
}

#[test]
fn a_later_call_runs_on_the_stream_thread_and_a_post_needs_a_live_run() {
    let script = Script::default();
    let (stream, heard) = streamed(script.clone(), &Hands::default());
    let (ran, on) = mpsc::channel();
    stream.post(stop(FIRST));
    stream.later(move || drop(ran.send(thread::current().name().map(str::to_owned))));
    assert_eq!(on.recv_timeout(LONG), Ok(Some(THREAD.to_owned())));
    assert!(script.calls().is_empty(), "no worker took the run");
    assert!(heard.try_recv().is_err(), "nothing is told");
}
