//! Microphone samples to what the engines take: mono, 16 kHz, 16-bit.

/// Largest 16-bit sample value, as a float scale.
const PCM_SCALE: f32 = i16::MAX as f32;

/// Interleaved frames of `channels` averaged to one channel.
pub fn mono(frames: &[f32], channels: u16) -> Vec<f32> {
    if channels == 0 {
        return Vec::new();
    }
    frames
        .chunks_exact(usize::from(channels))
        .map(|f| f.iter().sum::<f32>() / f.len() as f32)
        .collect()
}

/// `x` from `from` Hz to `to` Hz: averaged windows going down, straight lines between samples going up.
pub fn to_rate(x: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == 0 || to == 0 || x.is_empty() {
        return Vec::new();
    }
    let n = (x.len() as u64 * u64::from(to) / u64::from(from)) as usize;
    let step = f64::from(from) / f64::from(to);
    let last = x.len() - 1;
    (0..n)
        .map(|i| {
            let start = i as f64 * step;
            let a = (start as usize).min(last);
            if step > 1.0 {
                let b = (((i + 1) as f64 * step) as usize).clamp(a + 1, x.len());
                x[a..b].iter().sum::<f32>() / (b - a) as f32
            } else {
                let t = (start - a as f64) as f32;
                x[a] + (x[(a + 1).min(last)] - x[a]) * t
            }
        })
        .collect()
}

/// Floats in -1..1 as 16-bit samples, clamped.
pub fn pcm16(x: &[f32]) -> Vec<i16> {
    x.iter()
        .map(|v| (v.clamp(-1.0, 1.0) * PCM_SCALE).round() as i16)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_is_averaged_to_mono() {
        assert_eq!(mono(&[1.0, 3.0, 2.0, 4.0], 2), vec![2.0, 3.0]);
        assert_eq!(mono(&[0.5, 0.25], 1), vec![0.5, 0.25]);
        assert!(mono(&[1.0], 0).is_empty());
    }

    #[test]
    fn common_mic_rates_become_16k() {
        let flat = |n| vec![0.5f32; n];
        for (from, n) in [(48000, 4800), (44100, 4410), (16000, 1600), (8000, 800)] {
            let out = to_rate(&flat(n), from, 16000);
            assert_eq!(out.len(), 1600, "from {from} Hz");
            assert!(
                out.iter().all(|&x| (x - 0.5).abs() < 1e-6),
                "from {from} Hz"
            );
        }
        assert!(to_rate(&flat(10), 0, 16000).is_empty());
    }

    #[test]
    fn a_ramp_keeps_its_shape() {
        let ramp: Vec<f32> = (0..12).map(|i| i as f32).collect();
        assert_eq!(
            to_rate(&ramp, 48000, 16000),
            vec![1.0, 4.0, 7.0, 10.0],
            "means of each 3"
        );
        assert_eq!(
            to_rate(&[0.0, 2.0, 4.0], 8000, 16000),
            vec![0.0, 1.0, 2.0, 3.0, 4.0, 4.0]
        );
    }

    #[test]
    fn pcm16_rounds_and_clamps() {
        assert_eq!(pcm16(&[0.0, 0.5, 2.0, -2.0]), vec![0, 16384, 32767, -32767]);
    }
}
