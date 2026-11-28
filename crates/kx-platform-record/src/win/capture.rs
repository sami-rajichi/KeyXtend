//! Windows Graphics Capture of the part's monitor: it draws the pointer, drops its yellow border and leaves out shielded
//! windows. Each new frame's part is copied on the graphics card into a texture of its own, which the encoder reads
//! there, so no frame passes through the processor.

use windows::Graphics::Capture::{
    Direct3D11CaptureFrame, Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Win32::Foundation::{HMODULE, POINT, RECT, S_OK};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_BOX,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_SDK_VERSION,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11CreateDevice, ID3D11Device,
    ID3D11DeviceContext, ID3D11Multithread, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, HMONITOR, MONITOR_DEFAULTTONULL, MONITORINFO, MonitorFromPoint,
};
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::core::Interface;

use super::{refused, size32};
use crate::record::{Area, RecordFault, Screen, Shape};

/// A running capture of one part.
pub(super) struct Capture {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    pool: Direct3D11CaptureFramePool,
    session: GraphicsCaptureSession,
    area: D3D11_BOX,
}

impl Capture {
    /// Starts capturing the monitor that holds `area`, `shape`'s size of it, with the pointer and without the border as
    /// the session reads them back, or refuses; the pool holds `buffers` frames.
    pub(super) fn open(area: &Area, shape: &Shape, buffers: u8) -> Result<Self, RecordFault> {
        let (monitor, area) = place(area, shape)?;
        let (device, context) = device()?;
        let item = item(monitor)?;
        let pool = pool(&device, &item, buffers)?;
        let start = || -> windows::core::Result<GraphicsCaptureSession> {
            let session = pool.CreateCaptureSession(&item)?;
            session.SetIsCursorCaptureEnabled(true)?;
            session.SetIsBorderRequired(false)?;
            if !session.IsCursorCaptureEnabled()? || session.IsBorderRequired()? {
                return Err(windows::core::Error::empty());
            }
            session.StartCapture()?;
            Ok(session)
        };
        let session = start().map_err(refused)?;
        Ok(Self {
            device,
            context,
            pool,
            session,
            area,
        })
    }

    /// The graphics device the frames live on, which the encoder shares.
    pub(super) fn device(&self) -> &ID3D11Device {
        &self.device
    }

    /// The newest frame in the pool, the older ones closed; `None` when none came.
    fn newest(&self) -> Result<Option<Direct3D11CaptureFrame>, RecordFault> {
        let mut newest: Option<Direct3D11CaptureFrame> = None;
        loop {
            match self.pool.TryGetNextFrame() {
                Ok(frame) => {
                    if let Some(old) = newest.replace(frame) {
                        let _ = old.Close();
                    }
                }
                // An empty answer is no frame, not a failure.
                Err(e) if e.code() == S_OK => return Ok(newest),
                Err(e) => return Err(refused(e)),
            }
        }
    }

    /// Copies `frame`'s part into a new texture on the card, then gives the frame back to the pool.
    #[allow(unsafe_code, reason = "Copies a captured texture on our device.")]
    fn keep(&self, frame: &Direct3D11CaptureFrame) -> Result<ID3D11Texture2D, RecordFault> {
        let surface = frame.Surface().map_err(refused)?;
        let access: IDirect3DDxgiInterfaceAccess = surface.cast().map_err(refused)?;
        // SAFETY: the frame's live surface, read as the texture it wraps.
        let texture: ID3D11Texture2D = unsafe { access.GetInterface() }.map_err(refused)?;
        let copy = texture_of(&self.device, &self.area)?;
        let area = Some(&raw const self.area);
        // SAFETY: both textures live on our device, and the box lies inside the monitor, as `place` checked.
        unsafe {
            self.context
                .CopySubresourceRegion(&copy, 0, 0, 0, 0, &texture, 0, area);
        }
        let _ = frame.Close();
        Ok(copy)
    }
}

impl Screen<ID3D11Texture2D> for Capture {
    fn grab(&mut self) -> Result<Option<ID3D11Texture2D>, RecordFault> {
        match self.newest()? {
            Some(frame) => self.keep(&frame).map(Some),
            None => Ok(None),
        }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        let _ = self.session.Close();
        let _ = self.pool.Close();
    }
}

/// The monitor that holds `area`'s corner, and `shape`'s box on it there; refused when it leaves that monitor.
#[allow(unsafe_code, reason = "Reads a monitor's place.")]
fn place(area: &Area, shape: &Shape) -> Result<(HMONITOR, D3D11_BOX), RecordFault> {
    let corner = POINT {
        x: area.x,
        y: area.y,
    };
    // SAFETY: a plain lookup by a point.
    let monitor = unsafe { MonitorFromPoint(corner, MONITOR_DEFAULTTONULL) };
    let mut info = MONITORINFO {
        cbSize: size32::<MONITORINFO>(),
        ..Default::default()
    };
    // SAFETY: a monitor handle from the lookup, and a local whose size it states.
    if monitor.is_invalid() || !unsafe { GetMonitorInfoW(monitor, &raw mut info) }.as_bool() {
        return Err(RecordFault::Failed);
    }
    box_on(area, shape, info.rcMonitor).map(|area| (monitor, area))
}

/// `shape`'s box from `area`'s corner on a monitor at `rc`, in the monitor's own pixels; refused when it leaves it.
pub(super) fn box_on(area: &Area, shape: &Shape, rc: RECT) -> Result<D3D11_BOX, RecordFault> {
    let span = |from: i32, to: i32| u32::try_from(i64::from(to) - i64::from(from)).ok();
    let (left, top) = (span(rc.left, area.x), span(rc.top, area.y));
    let (wide, tall) = (span(rc.left, rc.right), span(rc.top, rc.bottom));
    let (Some(left), Some(top), Some(wide), Some(tall)) = (left, top, wide, tall) else {
        return Err(RecordFault::Failed);
    };
    let (right, bottom) = (
        left.saturating_add(shape.width),
        top.saturating_add(shape.height),
    );
    if right > wide || bottom > tall {
        return Err(RecordFault::Failed);
    }
    Ok(D3D11_BOX {
        left,
        top,
        front: 0,
        right,
        bottom,
        back: 1,
    })
}

/// A Direct3D 11 device on the default graphics card, with BGRA and video support as the capture and the encoder
/// need, guarded for the encoder's threads.
#[allow(unsafe_code, reason = "Makes a Direct3D device of ours.")]
fn device() -> Result<(ID3D11Device, ID3D11DeviceContext), RecordFault> {
    let (mut device, mut context) = (None, None);
    let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT;
    // SAFETY: two locals for Windows to fill; no adapter and no software module, so the default hardware one.
    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            flags,
            None,
            D3D11_SDK_VERSION,
            Some(&raw mut device),
            None,
            Some(&raw mut context),
        )
    }
    .map_err(refused)?;
    let (device, context) = device.zip(context).ok_or(RecordFault::Failed)?;
    let guard: ID3D11Multithread = device.cast().map_err(refused)?;
    // SAFETY: a live device of ours; the old setting it hands back is not needed.
    let _ = unsafe { guard.SetMultithreadProtected(true) };
    Ok((device, context))
}

/// The capture item of `monitor`.
#[allow(unsafe_code, reason = "Asks Windows for a monitor's capture item.")]
fn item(monitor: HMONITOR) -> Result<GraphicsCaptureItem, RecordFault> {
    let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
        .map_err(refused)?;
    // SAFETY: a live monitor handle from `place`.
    unsafe { interop.CreateForMonitor(monitor) }.map_err(refused)
}

/// A free-threaded pool of `buffers` frames of `item`'s size on `device`.
#[allow(unsafe_code, reason = "Shares our Direct3D device with WinRT.")]
fn pool(
    device: &ID3D11Device,
    item: &GraphicsCaptureItem,
    buffers: u8,
) -> Result<Direct3D11CaptureFramePool, RecordFault> {
    let dxgi: IDXGIDevice = device.cast().map_err(refused)?;
    // SAFETY: a live DXGI device of ours.
    let shared = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi) };
    let winrt: IDirect3DDevice = shared.and_then(|d| d.cast()).map_err(refused)?;
    let size = item.Size().map_err(refused)?;
    let format = DirectXPixelFormat::B8G8R8A8UIntNormalized;
    Direct3D11CaptureFramePool::CreateFreeThreaded(&winrt, format, i32::from(buffers), size)
        .map_err(refused)
}

/// A BGRA texture on the card of `area`'s size, which the encoder's colour converter reads.
#[allow(unsafe_code, reason = "Makes a texture of ours.")]
fn texture_of(device: &ID3D11Device, area: &D3D11_BOX) -> Result<ID3D11Texture2D, RecordFault> {
    let bind = D3D11_BIND_SHADER_RESOURCE.0 | D3D11_BIND_RENDER_TARGET.0;
    let desc = D3D11_TEXTURE2D_DESC {
        Width: area.right - area.left,
        Height: area.bottom - area.top,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: bind.cast_unsigned(),
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };
    let mut texture = None;
    // SAFETY: a described texture with no first data, into a local.
    unsafe { device.CreateTexture2D(&raw const desc, None, Some(&raw mut texture)) }
        .map_err(refused)?;
    texture.ok_or(RecordFault::Failed)
}
