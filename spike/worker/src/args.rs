//! The worker's switches: weak mode, cloud allowed, and a folder to keep clips in.

use std::path::PathBuf;

use spike_core::voicecfg::VoiceConfig;

/// What the worker was started with.
#[derive(Debug, Default)]
pub struct Args {
    /// Run the local server capped (weak mode).
    pub weak: bool,
    /// The cloud engine may be used.
    pub cloud: bool,
    /// Folder to keep each recorded clip in, for the bench.
    pub keep: Option<PathBuf>,
}

impl Args {
    /// Reads the switches named in `v`; anything else is an error.
    pub fn parse(v: &VoiceConfig, mut it: impl Iterator<Item = String>) -> Result<Args, String> {
        let mut a = Args::default();
        while let Some(s) = it.next() {
            if s == v.weak_arg {
                a.weak = true;
            } else if s == v.cloud_arg {
                a.cloud = true;
            } else if s == v.keep_arg {
                a.keep = Some(it.next().ok_or(format!("{s} needs a folder"))?.into());
            } else {
                return Err(format!("unknown switch {s}"));
            }
        }
        Ok(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voice() -> spike_core::voicecfg::VoiceConfig {
        spike_core::config::load().expect("spike.toml loads").voice
    }

    fn of(a: &[&str]) -> Result<Args, String> {
        Args::parse(&voice(), a.iter().map(|s| s.to_string()))
    }

    #[test]
    fn no_switch_means_normal_local_and_nothing_kept() {
        let a = of(&[]).expect("valid");
        assert!(!a.weak && !a.cloud && a.keep.is_none());
    }

    #[test]
    fn each_switch_is_read() {
        let v = voice();
        let a = of(&[&v.weak_arg, &v.cloud_arg, &v.keep_arg, "D:/clips"]).expect("valid");
        assert!(a.weak && a.cloud);
        assert_eq!(a.keep.as_deref(), Some(std::path::Path::new("D:/clips")));
    }

    #[test]
    fn unknown_or_incomplete_switches_are_errors() {
        assert!(of(&["--fly"]).is_err());
        assert!(of(&[&voice().keep_arg]).is_err(), "keep needs a folder");
    }
}
