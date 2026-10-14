//! Tests of the `XAudio2` side: error kinds, the voices' format, and the factory call itself, which plays nothing.

use super::*;
use windows::Win32::Foundation::E_FAIL;

#[test]
fn a_lost_or_changed_device_is_a_loss_and_any_other_error_a_failure() {
    for code in LOST {
        assert_eq!(fault(&code.into()), Fault::Lost(code.0));
    }
    assert_eq!(fault(&E_FAIL.into()), Fault::Failed(E_FAIL.0));
}

#[test]
fn the_voices_take_mono_pcm16_at_the_clip_rate_and_every_rate_the_port_names() {
    let f = pcm(48_000);
    let (tag, channels, rate) = (f.wFormatTag, f.nChannels, f.nSamplesPerSec);
    let (per_second, block, bits) = (f.nAvgBytesPerSec, f.nBlockAlign, f.wBitsPerSample);
    assert_eq!((tag, channels, rate), (1, 1, 48_000));
    assert_eq!((per_second, block, bits), (96_000, 2, 16));
    let (low, high) = (Clip::RATES.start(), Clip::RATES.end());
    assert!(WinOs::RATES.contains(low) && WinOs::RATES.contains(high));
}

#[test]
fn the_factory_loads_from_system32_and_makes_an_engine_that_drops_cleanly() {
    assert!(factory().is_ok(), "XAudio2 2.9 and its factory are found");
    // No mastering voice is made, so this needs no audio device and plays nothing.
    let os = WinOs::new();
    let engine = os.engine();
    assert!(
        engine.is_ok(),
        "the call's signature holds: {:?}",
        engine.as_ref().err()
    );
    // The engine goes before COM, as in the rig.
    drop(engine);
    drop(os);
}
