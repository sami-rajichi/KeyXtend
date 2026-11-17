//! The default microphone through `cpal` (WASAPI), moved from the spike: its default format, each callback's samples
//! as floats in a wiped buffer, and its errors as the job's fixed faults, never the library's words.

use super::{Format, Opened};
use crate::voice::Feed;
use crate::voice::wipe::Buf;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, ErrorKind, FromSample, SampleFormat, SizedSample, Stream, StreamConfig};
use kx_ipc::VoiceFault;

/// Opens the default microphone in its default format; its chunks go to `feed` until the stream drops.
pub fn open(feed: Feed) -> Result<Opened, VoiceFault> {
    let dev = cpal::default_host().default_input_device();
    let dev = dev.ok_or(VoiceFault::NoMic)?;
    let supported = dev.default_input_config().map_err(|e| refused(e.kind()))?;
    let cfg = supported.config();
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(&dev, cfg, feed),
        SampleFormat::I16 => build::<i16>(&dev, cfg, feed),
        SampleFormat::I32 => build::<i32>(&dev, cfg, feed),
        SampleFormat::U16 => build::<u16>(&dev, cfg, feed),
        _ => Err(VoiceFault::Internal),
    }?;
    stream.play().map_err(|e| refused(e.kind()))?;
    let format = Format {
        rate: cfg.sample_rate,
        channels: cfg.channels,
    };
    Ok(Opened {
        format,
        live: Box::new(stream),
    })
}

/// An input stream that hands each callback's samples to `feed` as floats, and a lost stream's fault once per error;
/// a chunk with no room for its copy is `Memory`.
fn build<T>(dev: &Device, cfg: StreamConfig, feed: Feed) -> Result<Stream, VoiceFault>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let lost = feed.clone();
    let data = move |data: &[T], _: &cpal::InputCallbackInfo| {
        let chunk = Buf::collect(data.iter().map(|s| s.to_sample::<f32>()));
        match chunk {
            Ok(chunk) => feed.sound(chunk),
            Err(fault) => feed.lost(fault),
        }
    };
    let error = move |e: cpal::Error| {
        if let Some(fault) = stopped(e.kind()) {
            lost.lost(fault);
        }
    };
    dev.build_input_stream(cfg, data, error, None)
        .map_err(|e| refused(e.kind()))
}

/// Why the microphone could not open.
fn refused(kind: ErrorKind) -> VoiceFault {
    match kind {
        ErrorKind::DeviceNotAvailable | ErrorKind::HostUnavailable => VoiceFault::NoMic,
        ErrorKind::DeviceBusy => VoiceFault::MicBusy,
        ErrorKind::PermissionDenied => VoiceFault::MicRefused,
        _ => VoiceFault::Internal,
    }
}

/// Why an open microphone stopped, or `None` for a glitch or a reroute it goes on after.
fn stopped(kind: ErrorKind) -> Option<VoiceFault> {
    match kind {
        ErrorKind::Xrun | ErrorKind::DeviceChanged | ErrorKind::RealtimeDenied => None,
        ErrorKind::DeviceBusy => Some(VoiceFault::MicBusy),
        ErrorKind::PermissionDenied => Some(VoiceFault::MicRefused),
        _ => Some(VoiceFault::Unplugged),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_open_that_fails_names_the_microphones_fault() {
        assert_eq!(refused(ErrorKind::DeviceNotAvailable), VoiceFault::NoMic);
        assert_eq!(refused(ErrorKind::HostUnavailable), VoiceFault::NoMic);
        assert_eq!(refused(ErrorKind::DeviceBusy), VoiceFault::MicBusy);
        assert_eq!(refused(ErrorKind::PermissionDenied), VoiceFault::MicRefused);
        assert_eq!(refused(ErrorKind::UnsupportedConfig), VoiceFault::Internal);
    }

    #[test]
    fn a_glitch_goes_on_but_a_lost_device_stops_the_run() {
        for kind in [
            ErrorKind::Xrun,
            ErrorKind::DeviceChanged,
            ErrorKind::RealtimeDenied,
        ] {
            assert_eq!(stopped(kind), None, "{kind:?}");
        }
        assert_eq!(
            stopped(ErrorKind::DeviceNotAvailable),
            Some(VoiceFault::Unplugged)
        );
        assert_eq!(
            stopped(ErrorKind::StreamInvalidated),
            Some(VoiceFault::Unplugged)
        );
        assert_eq!(
            stopped(ErrorKind::PermissionDenied),
            Some(VoiceFault::MicRefused)
        );
        assert_eq!(stopped(ErrorKind::DeviceBusy), Some(VoiceFault::MicBusy));
    }
}
