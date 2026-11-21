//! The module's start over the shared fake host: it reads `[voice]`, drops a bad pick or service entry alone with the
//! settings notice, fails on a bad section or a missing service, starts with a fixed line when cloud speech is missing,
//! and stops.

use std::sync::{Arc, Mutex};

use kx_mod_voice::{ID, VoiceModule, VoiceSettings};
use kx_module_api::ports::{CloudSpeechKey, SpeechKey};
use kx_module_api::{
    Host, Module, ModuleCx, Notice, NoticeArg, ServiceKey, Subscription, args, keys,
};
use kx_test_support::host::FakeHost;
use tracing::Level;

use crate::logs::lines_of;
use crate::rig::{Fakes, start, stop};

/// The notices published on `host`'s bus while the guard lives.
fn notices(host: &FakeHost) -> (Arc<Mutex<Vec<Notice>>>, Subscription) {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&heard);
    let guard = host
        .bus()
        .subscribe(move |n: &Notice| sink.lock().unwrap().push(n.clone()));
    (heard, guard.unwrap())
}

/// A `picks` table of `pairs`.
fn picks(pairs: &[(&str, &str)]) -> toml::Value {
    let table = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), toml::Value::from(*v)));
    toml::Value::Table(table.collect())
}

#[test]
fn it_starts_on_the_defaults_and_stops() {
    let mut host = Fakes::new().host();
    let (heard, _guard) = notices(&host);
    let mut module = VoiceModule::new();
    assert_eq!(module.manifest().id.as_str(), "voice");
    assert!(start(&mut module, &mut host).is_ok());
    stop(&mut module, &mut host);
    assert!(heard.lock().unwrap().is_empty() && host.sink.writes().is_empty());
}

#[test]
fn a_bad_pick_falls_back_alone_with_the_notice_and_is_written_back() {
    let mut host = Fakes::new().host();
    host.put(
        "picks",
        picks(&[("fr", "whisper-medium-q5"), ("ar", "cohere-q8")]),
    );
    let (heard, _guard) = notices(&host);
    let mut module = VoiceModule::new();
    assert!(start(&mut module, &mut host).is_ok());
    stop(&mut module, &mut host);
    let mut want = Notice::about(keys::SETTINGS_RESET, ID);
    want.args
        .push((args::KEYS, NoticeArg::List(vec!["picks.fr".into()])));
    assert_eq!(*heard.lock().unwrap(), [want]);
    let kept = picks(&[("ar", "cohere-q8")]);
    let written = toml::Table::from_iter([("picks".to_owned(), kept)]);
    assert_eq!(host.sink.writes(), [(ID, written)]);
    let saved = host.sink.read_as::<VoiceSettings>().unwrap();
    assert_eq!(saved.picks.get("ar").map(String::as_str), Some("cohere-q8"));
    assert_eq!(saved.picks.len(), 1);
}

#[test]
fn a_service_entry_no_row_takes_is_dropped_alone_and_written_back() {
    let mut host = Fakes::new().host();
    let entry = |terms: i64| {
        let on = "2026-11-17".parse::<toml::value::Datetime>().unwrap();
        toml::Value::Table(toml::Table::from_iter([
            ("on".to_owned(), toml::Value::Datetime(on)),
            ("terms".to_owned(), terms.into()),
        ]))
    };
    let services = [("groq", entry(1)), ("no-such-row", entry(1))];
    let services = toml::Table::from_iter(services.map(|(k, v)| (k.to_owned(), v)));
    host.put("services", toml::Value::Table(services));
    let (heard, _guard) = notices(&host);
    let mut module = VoiceModule::new();
    assert!(start(&mut module, &mut host).is_ok());
    stop(&mut module, &mut host);
    let mut want = Notice::about(keys::SETTINGS_RESET, ID);
    let named = vec!["services.no-such-row".into()];
    want.args.push((args::KEYS, NoticeArg::List(named)));
    assert_eq!(*heard.lock().unwrap(), [want]);
    let mut groq = entry(1);
    groq.as_table_mut()
        .unwrap()
        .insert("keep".to_owned(), true.into());
    let kept = toml::Table::from_iter([("groq".to_owned(), groq)]);
    let written = toml::Table::from_iter([("services".to_owned(), toml::Value::Table(kept))]);
    assert_eq!(host.sink.writes(), [(ID, written)]);
}

#[test]
fn a_bad_section_fails_the_start() {
    let mut host = Fakes::new().host();
    host.put("job_mb", 1.into());
    let mut module = VoiceModule::new();
    assert!(module.start(&mut ModuleCx::new(&mut host)).is_err());
}

#[test]
fn a_missing_service_fails_the_start() {
    let mut host = Fakes::new().host();
    host.services.remove(&SpeechKey::ID);
    let mut module = VoiceModule::new();
    assert!(start(&mut module, &mut host).is_err());
}

#[test]
fn a_hidden_run_starts_on_the_same_services() {
    let mut host = Fakes::new().host();
    let mut module = VoiceModule::hidden();
    assert!(start(&mut module, &mut host).is_ok());
    stop(&mut module, &mut host);
}

#[test]
fn a_host_without_cloud_speech_still_starts_and_says_so_in_one_fixed_line() {
    let mut host = Fakes::new().host();
    host.services.remove(&CloudSpeechKey::ID);
    let mut module = VoiceModule::new();
    let lines = lines_of(|| {
        assert!(
            start(&mut module, &mut host).is_ok(),
            "local models dictate"
        )
    });
    stop(&mut module, &mut host);
    let warned: Vec<&str> = lines
        .iter()
        .filter(|(level, _)| *level == Level::WARN)
        .map(|(_, message)| message.as_str())
        .collect();
    assert_eq!(warned, ["cloud speech is not available"]);
}
