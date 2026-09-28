//! The opt-in cloud engine: refused unless the worker was started with the cloud switch and a key is set.

use spike_core::voicecfg::CloudConfig;

use crate::engines::{self, MODEL_FIELD, Target};

/// HTTPS port.
const HTTPS_PORT: u16 = 443;
/// Why the cloud engine is refused when it was not turned on.
const OFF: &str = "cloud is off: the worker was not started with the cloud switch";

/// The cloud engine with its key. No `Debug`, so the key cannot reach a log.
pub struct Cloud {
    cfg: CloudConfig,
    key: String,
}

impl Cloud {
    /// The engine, refused when not `allowed` or when `key` is missing, empty or holds control characters.
    pub fn new(cfg: &CloudConfig, allowed: bool, key: Option<String>) -> Result<Cloud, String> {
        if !allowed {
            return Err(OFF.to_string());
        }
        let key = key
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .ok_or_else(|| format!("no API key in {}", cfg.key_env))?;
        if key.chars().any(char::is_control) {
            return Err(format!("the key in {} is not valid", cfg.key_env));
        }
        Ok(Cloud {
            cfg: cfg.clone(),
            key,
        })
    }

    /// The engine with the key from the environment; the key is not even read when cloud is off.
    pub fn from_env(cfg: &CloudConfig, allowed: bool) -> Result<Cloud, String> {
        let key = allowed.then(|| std::env::var(&cfg.key_env).ok()).flatten();
        Cloud::new(cfg, allowed, key)
    }

    /// The words in `wav`, spoken in `lang`; `auto` is left for the engine to detect.
    pub fn text(&self, wav: &[u8], lang: &str) -> Result<String, String> {
        let t = Target {
            host: &self.cfg.host,
            port: HTTPS_PORT,
            secure: true,
            path: &self.cfg.path,
            timeout_ms: self.cfg.timeout_ms,
            key: Some(&self.key),
        };
        let model = [(MODEL_FIELD, self.cfg.model.as_str())];
        engines::ask(&t, wav, engines::cloud_lang(lang), &model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> CloudConfig {
        spike_core::config::load()
            .expect("spike.toml loads")
            .voice
            .cloud
    }

    #[test]
    fn the_key_is_trimmed_and_control_characters_are_refused() {
        let c = Cloud::new(&cfg(), true, Some(" k1 \r\n".into())).expect("valid");
        assert_eq!(c.key, "k1");
        assert!(Cloud::new(&cfg(), true, Some("k1\r\nX-Evil: 1".into())).is_err());
    }
}
