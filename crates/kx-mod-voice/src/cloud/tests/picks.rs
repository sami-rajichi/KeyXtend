//! A cloud pick's text: its service and model parted at the first mark, a local file's name kept local, and its row
//! and listed model found once for every reader, in a run's own table too.

use super::*;

#[test]
fn a_pick_names_its_service_and_model_with_the_mark() {
    assert_eq!(
        pick_text("groq", turbo()),
        format!("groq{PICK_MARK}{}", turbo())
    );
    let routed = pick_text("openrouter", router_model(false));
    assert_eq!(
        cloud_pick(&routed),
        Some(("openrouter", router_model(false))),
        "its listed model"
    );
    assert_eq!(cloud_pick("custom:llama:7b"), Some(("custom", "llama:7b")));
    // A local file whose name holds the mark, as Linux and macOS allow, stays local (P15).
    for local in [
        "canary-v2-q8",
        "my-model.gguf",
        "",
        "ollama:llama.gguf",
        ":x",
    ] {
        assert_eq!(cloud_pick(local), None, "{local}");
    }
    let test = with_rows(ROUTER);
    assert_eq!(test.pick("router-test:a:b"), Some(("router-test", "a:b")));
    assert_eq!(test.pick("groq-ish:a"), None, "only a row id parts a pick");
}

#[test]
fn a_picks_row_and_listed_model_are_found_once_for_every_reader() {
    let text = pick_text("groq", turbo());
    let (groq, model, listed) = pick_row(&text).unwrap();
    let listed = listed.map(|m| m.id.as_str());
    assert_eq!(
        (groq.id.as_str(), model, listed),
        ("groq", turbo(), Some(turbo()))
    );
    let (custom, model, listed) = pick_row("custom:llama:7b").unwrap();
    assert_eq!(
        (custom.id.as_str(), model, listed),
        ("custom", "llama:7b", None)
    );
    assert!(pick_row("canary-v2-q8").is_none(), "a local pick");
    let router = pick_text("router-test", "openai/whisper-large-v3-turbo");
    assert!(pick_row(&router).is_none(), "a row the shipped table lacks");
    let held = with_rows(ROUTER);
    let (row, model, listed) = held.pick_row(&router).unwrap();
    let listed = listed.map(|m| m.name.as_str());
    assert_eq!(
        (row.id.as_str(), model, listed),
        (
            "router-test",
            "openai/whisper-large-v3-turbo",
            Some("Whisper Large v3 Turbo")
        )
    );
}
