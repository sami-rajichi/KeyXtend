//! `OpenRouter`'s keep-nothing switch as its ZDR endpoint list, read 2026-11-22, gives it: the switch sends
//! `provider.zdr`, and only a model with no ZDR endpoint is refused while it is on.

use super::*;

/// The shipped row.
fn router() -> &'static Service {
    shipped().get("openrouter").unwrap()
}

/// Its models that no ZDR endpoint serves, so keep-nothing refuses them; Voxtral's Mistral endpoint is one (list read
/// 2026-11-22).
const NO_ZDR: [&str; 1] = ["openai/gpt-transcribe"];

#[test]
fn openrouters_keep_nothing_switch_sends_its_zero_data_retention_field() {
    let keep = router().keep.as_ref();
    let field = keep.map(|f| (f.name.as_str(), f.value.as_str()));
    assert_eq!(field, Some(("provider", r#"{"zdr":true}"#)));
}

#[test]
fn keep_nothing_refuses_gpt_transcribe_alone_as_no_zdr_endpoint_serves_it() {
    let kept = router().models.iter().filter(|m| m.keeps);
    let ids: Vec<&str> = kept.map(|m| m.id.as_str()).collect();
    assert_eq!(ids, NO_ZDR, "Voxtral is served by Mistral's ZDR endpoint");
}
