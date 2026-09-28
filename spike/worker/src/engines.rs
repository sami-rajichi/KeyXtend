//! The two engines and which one answers: cloud only when it was allowed and has a key.

use serde_json::Value;
use spike_core::voicecfg::Engine;

use crate::cloud::Cloud;
use crate::http::{self, HTTP_OK, Reply, Request};
use crate::local::Local;
use crate::multipart::{self, Part};

/// Form field names and values shared by both engines (OpenAI-style transcription API).
pub const FILE_FIELD: &str = "file";
/// File name sent with each clip.
pub const CLIP_NAME: &str = "clip.wav";
/// Content type of each clip.
pub const WAV_TYPE: &str = "audio/wav";
/// Language field.
pub const LANG_FIELD: &str = "language";
/// Model field (cloud only).
pub const MODEL_FIELD: &str = "model";
/// Sampling temperature field and its value: 0 is the steadiest.
pub const TEMP_FIELD: (&str, &str) = ("temperature", "0");
/// Answer format field and its value.
pub const FORMAT_FIELD: (&str, &str) = ("response_format", "json");
/// Language value that lets the engine detect it.
pub const AUTO: &str = "auto";
/// Answer keys: the transcript, an error, and an error object's message.
const TEXT_KEY: &str = "text";
/// See `TEXT_KEY`.
const ERROR_KEY: &str = "error";
/// See `TEXT_KEY`.
const MESSAGE_KEY: &str = "message";
/// Longest error note kept from an answer, in characters.
const ERROR_CHARS: usize = 160;

/// Both engines, or why each is not available.
pub struct Engines {
    /// The local server.
    pub local: Result<Local, String>,
    /// The cloud engine.
    pub cloud: Result<Cloud, String>,
}

/// The engine that will answer.
pub enum Pick<'a> {
    /// The local server.
    Local(&'a Local),
    /// The cloud engine.
    Cloud(&'a Cloud),
}

impl Engines {
    /// Engine `e`, or the reason it is not available.
    pub fn pick(&self, e: Engine) -> Result<Pick<'_>, String> {
        match e {
            Engine::Local => self.local.as_ref().map(Pick::Local).map_err(Clone::clone),
            Engine::Cloud => self.cloud.as_ref().map(Pick::Cloud).map_err(Clone::clone),
        }
    }
}

impl Pick<'_> {
    /// The words in `wav`, spoken in `lang`.
    pub fn text(&self, wav: &[u8], lang: &str) -> Result<String, String> {
        match self {
            Pick::Local(l) => l.text(wav, lang),
            Pick::Cloud(c) => c.text(wav, lang),
        }
    }
}

/// Where a clip goes: an engine's address, path, wait and optional API key.
pub struct Target<'a> {
    /// Host name or address.
    pub host: &'a str,
    /// Port.
    pub port: u16,
    /// HTTPS when true.
    pub secure: bool,
    /// Request path.
    pub path: &'a str,
    /// Longest wait, in ms.
    pub timeout_ms: u64,
    /// API key sent as a bearer token.
    pub key: Option<&'a str>,
}

/// Sends the clip to `t` with `extra` fields; the transcript, or why there is none.
pub fn ask(
    t: &Target,
    wav: &[u8],
    lang: Option<&str>,
    extra: &[(&str, &str)],
) -> Result<String, String> {
    let b = multipart::boundary();
    let body = form(&b, wav, lang, extra);
    let kind = multipart::content_type(&b);
    let reply = http::post(&Request {
        host: t.host,
        port: t.port,
        secure: t.secure,
        path: t.path,
        content_type: Some(kind.as_str()),
        bearer: t.key,
        body: &body,
        timeout_ms: t.timeout_ms,
    })?;
    answer(reply)
}

/// The form both engines take: the clip, the fields in `extra`, and the language when given.
pub fn form(boundary: &str, wav: &[u8], lang: Option<&str>, extra: &[(&str, &str)]) -> Vec<u8> {
    let mut parts = vec![Part::file(FILE_FIELD, CLIP_NAME, WAV_TYPE, wav)];
    let fields = [TEMP_FIELD, FORMAT_FIELD];
    parts.extend(fields.iter().chain(extra).map(|(k, v)| Part::text(k, v)));
    if let Some(l) = lang {
        parts.push(Part::text(LANG_FIELD, l));
    }
    multipart::body(boundary, &parts)
}

/// The language the cloud engine gets: none for `auto`, which it detects by itself.
pub fn cloud_lang(lang: &str) -> Option<&str> {
    (lang != AUTO).then_some(lang)
}

/// The transcript in an engine's answer, or its error.
pub fn answer(r: Reply) -> Result<String, String> {
    if r.status == HTTP_OK {
        return text_of(&r.body);
    }
    let why = text_of(&r.body).err().unwrap_or_default();
    Err(format!("the engine answered {}: {why}", r.status))
}

/// The `text` of a JSON answer; an `error` string or object becomes the error.
pub fn text_of(body: &[u8]) -> Result<String, String> {
    let v: Value =
        serde_json::from_slice(body).map_err(|_| "the answer is not JSON".to_string())?;
    if let Some(t) = v.get(TEXT_KEY).and_then(Value::as_str) {
        return Ok(t.trim().to_string());
    }
    let e = v.get(ERROR_KEY);
    let why = e
        .and_then(Value::as_str)
        .or_else(|| e.and_then(|e| e.get(MESSAGE_KEY)).and_then(Value::as_str))
        .unwrap_or("the answer has no text");
    Err(why.chars().take(ERROR_CHARS).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spike_core::voicecfg::CloudConfig;

    fn cloud_cfg() -> CloudConfig {
        spike_core::config::load()
            .expect("spike.toml loads")
            .voice
            .cloud
    }

    #[test]
    fn cloud_is_refused_before_any_network_call() {
        let off = Cloud::new(&cloud_cfg(), false, Some("k".into()));
        assert!(off.is_err_and(|e| e.contains("off")));
        let no_key = Cloud::new(&cloud_cfg(), true, None);
        assert!(no_key.is_err_and(|e| e.contains(&cloud_cfg().key_env)));
        assert!(Cloud::new(&cloud_cfg(), true, Some(String::new())).is_err());
        assert!(Cloud::new(&cloud_cfg(), true, Some("k".into())).is_ok());
    }

    #[test]
    fn the_engine_asked_for_is_the_one_used_or_its_reason_is_given() {
        let e = Engines {
            local: Err("server did not start".into()),
            cloud: Cloud::new(&cloud_cfg(), true, Some("k".into())),
        };
        assert!(matches!(e.pick(Engine::Cloud), Ok(Pick::Cloud(_))));
        assert_eq!(
            e.pick(Engine::Local).err().as_deref(),
            Some("server did not start")
        );
    }

    #[test]
    fn answers_are_read_from_json() {
        assert_eq!(text_of(br#"{"text":" hello "}"#), Ok("hello".to_string()));
        assert!(text_of(br#"{"error":"bad"}"#).is_err_and(|e| e.contains("bad")));
        assert!(text_of(br#"{"error":{"message":"no key"}}"#).is_err_and(|e| e.contains("no key")));
        assert!(text_of(b"<html>").is_err());
    }

    #[test]
    fn a_refusal_names_the_status_and_the_server_message() {
        fn refused(body: &[u8]) -> Result<String, String> {
            answer(Reply {
                status: 401,
                body: body.to_vec(),
            })
        }
        assert_eq!(
            refused(br#"{"error":{"message":"bad key"}}"#),
            Err("the engine answered 401: bad key".to_string())
        );
        let page = refused(b"<html>login page</html>").expect_err("refused");
        assert!(page.contains("401") && page.contains("not JSON"), "{page}");
        assert!(!page.contains("login"), "the raw body is never echoed");
    }

    #[test]
    fn the_local_server_always_gets_the_language_but_the_cloud_never_gets_auto() {
        let has_lang =
            |lang| String::from_utf8_lossy(&form("B", b"", lang, &[])).contains(LANG_FIELD);
        assert!(
            has_lang(Some(AUTO)),
            "the local server detects the language only when told auto"
        );
        assert!(!has_lang(None));
        assert_eq!(cloud_lang("ar"), Some("ar"));
        assert_eq!(cloud_lang(AUTO), None);
    }
}
