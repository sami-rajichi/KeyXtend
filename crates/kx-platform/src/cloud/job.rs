//! One run or key test in a speech-net job of its own: the stream, the shared state and the timer, opened together; a
//! run's sound handed over in parts, each piece's buffer wiped as it goes. Dropping it cancels the job's run and says
//! goodbye to the job, and never waits on the worker.

use super::board::Board;
use super::live::{Back, Live, Tell};
use super::{frame, tick};
use crate::worker::{Launch, NetFrames, Spawner, Split, Stream, Times, Wait};
use kx_ipc::NetAsk;
use kx_module_api::ports::{CloudFault, CloudRun};
use kx_module_api::{Clock, SecretBytes};
use std::sync::Arc;

/// A run's job's stream thread.
const THREAD: &str = "kx-net";

/// A run or key test and its job.
pub(super) struct NetJob<S: Spawner<NetFrames, Worker: Split<NetFrames>>> {
    stream: Arc<Stream<S, NetFrames>>,
    live: Arc<Live>,
}

impl<S: Spawner<NetFrames, Worker: Split<NetFrames>>> NetJob<S> {
    /// A job as `launch` and `times` say, whose notes `board` judges and `tell` hears; nothing starts before `begin`.
    ///
    /// # Errors
    /// `Gone` when its timer cannot start.
    pub(super) fn open(
        spawner: Arc<S>,
        clock: Arc<dyn Clock>,
        (launch, times): (Launch, Times),
        board: Board,
        tell: Tell,
    ) -> Result<Self, CloudFault> {
        let live = Arc::new(Live::new(board, tell, Arc::clone(&clock)));
        let hear = {
            let live = Arc::clone(&live);
            move |told| live.hear(told)
        };
        let stream = Stream::open(THREAD, spawner, Arc::clone(&clock), times, launch, hear);
        let stream = Arc::new(stream);
        let weak = Arc::downgrade(&live);
        let fire = move || weak.upgrade().is_some_and(Live::fire);
        let wake = tick::start(clock, fire).ok_or(CloudFault::Gone)?;
        let back: Arc<dyn Back> = Arc::clone(&stream) as Arc<dyn Back>;
        live.bind(Arc::downgrade(&back), wake);
        Ok(Self { stream, live })
    }

    /// The shared state, for the host's own checks.
    pub(super) const fn live(&self) -> &Arc<Live> {
        &self.live
    }

    /// Sends the ask `make` builds from the job's fresh id, which the run notes first.
    ///
    /// # Errors
    /// `Gone` when the job cannot start or take it.
    pub(super) fn begin(
        &self,
        make: impl FnOnce(u32) -> NetAsk<'static> + Send + 'static,
    ) -> Result<u32, CloudFault> {
        let live = Arc::clone(&self.live);
        let make = move |id| {
            live.name(id);
            make(id)
        };
        self.stream
            .begin(make, Wait::Off)
            .map_err(|_| CloudFault::Gone)
    }

    /// Hands run `id`'s `pieces` over in parts, stopping once the run ended, then starts the first piece's time; a
    /// piece's buffer is wiped once its parts are sent.
    pub(super) fn hand_over(&self, id: u32, pieces: Vec<SecretBytes>) {
        for part in frame::parts(id, pieces) {
            if self.live.ended() || self.stream.ask(part, Wait::Off).is_err() {
                break;
            }
        }
        self.live.start();
    }
}

impl<S: Spawner<NetFrames, Worker: Split<NetFrames>>> CloudRun for NetJob<S> {}

impl<S: Spawner<NetFrames, Worker: Split<NetFrames>>> Drop for NetJob<S> {
    /// Ends the run and cancels the job's run; the stream then says goodbye to the job on its own thread.
    fn drop(&mut self) {
        self.live.close();
    }
}
