//! Systems with no recorder yet (P15): every video job answers `NoVideo`, so the app dims both video keys there.

use std::any::Any;
use std::path::Path;

use crate::record::{Ear, Film, Plan, RecordFault, Reel, Shape, Sink, Studio, Switch};

/// A studio with nothing in it.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoStudio;

impl Studio for NoStudio {
    type Frame = ();
    type Still = ();

    fn ready(&self) -> Result<Box<dyn Any>, RecordFault> {
        Err(RecordFault::NoVideo)
    }

    fn room_mb(&self, _: &Path) -> Option<u64> {
        None
    }

    fn ear(&self, _: Switch) -> Result<Box<dyn Ear>, RecordFault> {
        Err(RecordFault::NoVideo)
    }

    fn film(&self, _: &Plan) -> Result<Film<()>, RecordFault> {
        Err(RecordFault::NoVideo)
    }

    fn reel(&self, _: &Path) -> Result<Box<dyn Reel<()>>, RecordFault> {
        Err(RecordFault::NoVideo)
    }

    fn cut(&self, _: &Path, _: Shape) -> Result<Box<dyn Sink<()>>, RecordFault> {
        Err(RecordFault::NoVideo)
    }
}
