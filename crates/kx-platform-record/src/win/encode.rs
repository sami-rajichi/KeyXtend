//! A recording's file: BGRA textures on the graphics card, which the Sink Writer's own colour converter and the
//! hardware encoder read there through a DXGI device manager, so no frame passes through the processor.

use std::path::Path;

use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D};
use windows::Win32::Media::MediaFoundation::{
    IMF2DBuffer, IMFDXGIDeviceManager, IMFMediaBuffer, MFCreateDXGIDeviceManager,
    MFCreateDXGISurfaceBuffer, MFVideoFormat_RGB32,
};
use windows::core::Interface;

use super::refused;
use super::writer::Writer;
use crate::record::{RecordFault, Shape, Sink};
use crate::table::Table;

/// A recording being written from textures on the card.
pub(super) struct Encoder {
    writer: Writer,
    /// Kept for the writer's life, as it reaches the frames through it.
    _manager: IMFDXGIDeviceManager,
}

impl Encoder {
    /// Starts writing `path` as `shape` says, its frames read from textures on `device`.
    pub(super) fn open(
        path: &Path,
        shape: &Shape,
        device: &ID3D11Device,
        table: &Table,
    ) -> Result<Self, RecordFault> {
        let manager = manager(device).map_err(refused)?;
        let writer = Writer::open(path, shape, MFVideoFormat_RGB32, Some(&manager), table)?;
        Ok(Self {
            writer,
            _manager: manager,
        })
    }
}

impl Sink<ID3D11Texture2D> for Encoder {
    #[allow(unsafe_code, reason = "Wraps a texture of ours in a buffer.")]
    fn frame(&mut self, slot: u64, frame: &ID3D11Texture2D) -> Result<(), RecordFault> {
        let wrap = || -> windows::core::Result<IMFMediaBuffer> {
            // SAFETY: a live texture on the writer's device, wrapped whole; the buffer's length is its own.
            unsafe {
                let buffer = MFCreateDXGISurfaceBuffer(&ID3D11Texture2D::IID, frame, 0, false)?;
                buffer.SetCurrentLength(buffer.cast::<IMF2DBuffer>()?.GetContiguousLength()?)?;
                Ok(buffer)
            }
        };
        self.writer.frame(slot, &wrap().map_err(refused)?)
    }

    fn sound(&mut self, first: u64, pcm: &[i16]) -> Result<(), RecordFault> {
        self.writer.sound(first, pcm)
    }

    fn finish(self: Box<Self>) -> Result<(), RecordFault> {
        self.writer.finish()
    }
}

/// A device manager over `device`, through which the writer's converter and encoder reach the frames.
#[allow(unsafe_code, reason = "Makes a device manager of ours.")]
fn manager(device: &ID3D11Device) -> windows::core::Result<IMFDXGIDeviceManager> {
    let (mut token, mut manager) = (0, None);
    // SAFETY: two locals for Windows to fill, then a live manager and device.
    unsafe {
        MFCreateDXGIDeviceManager(&raw mut token, &raw mut manager)?;
        let manager = manager.ok_or_else(windows::core::Error::empty)?;
        manager.ResetDevice(device, token)?;
        Ok(manager)
    }
}
