//! The system's studio: a video job's thread is made per-monitor DPI aware, so it reads the part in physical pixels
//! as the app gives it, then starts COM and Media Foundation; the other jobs never call it and keep the worker's
//! default awareness.

use std::any::Any;
use std::path::Path;

use windows::Win32::Graphics::Direct3D11::ID3D11Texture2D;
use windows::Win32::System::Com::COINIT_MULTITHREADED;

use super::capture::Capture;
use super::com::Com;
use super::cut::Cut;
use super::encode::Encoder;
use super::media::Mf;
use super::pace::Pace;
use super::reel::{Decoded, WinReel};
use super::{ears, physical, room};
use crate::record::{Ear, Film, Plan, RecordFault, Reel, Shape, Sink, Studio, Switch};
use crate::table::{Table, table};

/// The system's capture, sound devices and encoders.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinStudio;

/// The shipped table, or `Failed` when it does not parse.
fn shipped() -> Result<&'static Table, RecordFault> {
    table().ok_or(RecordFault::Failed)
}

impl Studio for WinStudio {
    type Frame = ID3D11Texture2D;
    type Still = Decoded;

    fn ready(&self) -> Result<Box<dyn Any>, RecordFault> {
        let table = shipped()?;
        let com = Com::start(COINIT_MULTITHREADED).map_err(|_| RecordFault::Failed)?;
        if !physical() {
            return Err(RecordFault::Failed);
        }
        let mf = Mf::start(&table.media_dlls)?;
        // The tuple drops in order: Media Foundation ends before COM.
        Ok(Box::new((mf, com)))
    }

    fn room_mb(&self, folder: &Path) -> Option<u64> {
        room::room_mb(folder)
    }

    fn ear(&self, which: Switch) -> Result<Box<dyn Ear>, RecordFault> {
        ears::open(which, shipped()?)
    }

    fn film(&self, plan: &Plan) -> Result<Film<ID3D11Texture2D>, RecordFault> {
        let (table, shape) = (shipped()?, plan.shape());
        let capture = Capture::open(&plan.area, &shape, table.video.buffers)?;
        let sink = Encoder::open(&plan.file, &shape, capture.device(), table)?;
        Ok(Film {
            screen: Box::new(capture),
            sink: Box::new(sink),
            clock: Box::new(Pace::new(shape.fps)?),
        })
    }

    fn reel(&self, from: &Path) -> Result<Box<dyn Reel<Decoded>>, RecordFault> {
        Ok(Box::new(WinReel::open(from)?))
    }

    fn cut(&self, to: &Path, shape: Shape) -> Result<Box<dyn Sink<Decoded>>, RecordFault> {
        Ok(Box::new(Cut::open(to, shape, shipped()?)?))
    }
}
