//! The device or service that wrote each language's text, read back from the copy's log by id, never any text.

use kx_ipc::gate::{DEVICE, FAILED_LINE, FAULT, SERVICE, WROTE_LINE};
use kx_mod_voice::keys::KEY_UNREAD;
use kx_module_api::ports::{CardId, CloudFault, DevicePick};
use serde_json::{Value, json};

use super::super::wrote::{self, Writer};
use crate::budget::stamp_us;

/// Two made-up service ids, as a row's id is written.
const ONE: &str = "alpha";
const OTHER: &str = "beta";

fn card(id: &str) -> Writer {
    Writer::Device(DevicePick::Card(CardId::new(id).expect("an id")))
}

fn cpu() -> Writer {
    Writer::Device(DevicePick::Cpu)
}

fn service(id: &str) -> Writer {
    Writer::Service(id.to_owned())
}

/// A line of the copy's log stamped at second `s` past 10:00 that holds `rest`.
fn logged(s: u8, rest: &str) -> String {
    format!("2026-11-16T10:00:{s:02}.000000Z  INFO keyxtend::played::told: {rest}")
}

/// Whether `read` fits `want`.
fn fits(read: &[&Writer], want: &Writer) -> bool {
    let read = read.iter().map(|&w| w.clone()).collect();
    wrote::judge(&Ok(read), want).1
}

#[test]
fn the_copys_written_lines_since_its_start_give_each_device_or_service_by_id() {
    let m = WROTE_LINE;
    let log = [
        logged(0, &format!("{m} device=Vulkan9")),
        logged(2, "another line device=Vulkan8"),
        logged(3, &format!("{m} device=cpu")),
        logged(4, &format!("{m} device=Vulkan0")),
        logged(5, &format!("{m} service={ONE}")),
    ]
    .join("\n");
    let since = stamp_us("2026-11-16T10:00:01.000000Z").expect("a stamp");
    let got = wrote::read(&log, since);
    assert_eq!(got, Ok(vec![cpu(), card("Vulkan0"), service(ONE)]));
    assert_eq!(wrote::read("", since), Ok(Vec::new()), "none written");
}

#[test]
fn a_written_line_without_one_id_is_refused_and_its_value_never_printed() {
    let m = WROTE_LINE;
    for bad in [
        "device=auto",
        "device=a/b",
        "device=\"Vulkan0\"",
        "devices=cpu",
        "",
        "service=",
        "service=a/b",
        "device=cpu service=alpha",
    ] {
        let log = logged(3, &format!("{m} {bad}"));
        let e = wrote::read(&log, 0).expect_err(bad);
        assert!(e.contains(DEVICE), "{e}");
        assert!(bad.is_empty() || !e.contains(bad), "{e}");
    }
}

#[test]
fn with_a_card_picked_only_that_card_may_write_and_some_device_must() {
    let (vk, auto) = (card("Vulkan0"), Writer::Device(DevicePick::Auto));
    assert!(fits(&[&vk], &vk));
    assert!(!fits(&[&cpu()], &vk), "the processor wrote it");
    assert!(!fits(&[&card("Vulkan1")], &vk), "another card");
    assert!(!fits(&[&vk, &cpu()], &vk), "one on the processor");
    assert!(!fits(&[], &vk), "no reading");
    assert!(
        fits(&[&cpu()], &cpu()) && !fits(&[&vk], &cpu()),
        "the processor"
    );
    assert!(
        fits(&[&cpu()], &auto) && fits(&[&vk], &auto),
        "Automatic takes either"
    );
    assert!(!fits(&[], &auto), "but needs one");
    assert!(
        !fits(&[&service(ONE)], &auto),
        "never a silent move to a service"
    );
    let (v, ok) = wrote::judge(&Err("no log".into()), &vk);
    assert!(!ok && v["device_error"] == "no log", "{v}");
}

#[test]
fn on_a_cloud_line_only_its_service_may_write_and_it_must() {
    let one = service(ONE);
    assert!(fits(&[&one], &one));
    assert!(fits(&[&one, &one], &one), "each dictation");
    assert!(!fits(&[&service(OTHER)], &one), "another service");
    assert!(!fits(&[&one, &cpu()], &one), "a local model wrote one");
    assert!(!fits(&[], &one), "nothing written");
    let (v, ok) = wrote::judge(&Err("no log".into()), &one);
    assert!(!ok && v["service_error"] == "no log", "{v}");
}

#[test]
fn a_languages_fields_are_the_writers_id_and_numbers_never_text() {
    let vk = card("Vulkan0");
    let auto = Writer::Device(DevicePick::Auto);
    let (v, ok) = wrote::judge(&Ok(vec![cpu(), vk.clone()]), &auto);
    assert!(ok);
    assert_eq!(v["device"], "Vulkan0", "the last one");
    assert_eq!(v["written"], 2);
    assert_eq!(v["device_match"], true);
    let strings: Vec<&str> = v
        .as_object()
        .unwrap()
        .values()
        .filter_map(|x| x.as_str())
        .collect();
    assert_eq!(strings, ["Vulkan0"], "only the device's id is text");
    let (none, _) = wrote::judge(&Ok(Vec::new()), &vk);
    let fields = [&none["device"], &none["written"], &none["device_match"]];
    assert_eq!(fields, [&Value::Null, &json!(0), &json!(false)]);
    let (cloud, _) = wrote::judge(&Ok(vec![service(ONE)]), &service(ONE));
    let fields = [
        &cloud["service"],
        &cloud["written"],
        &cloud["service_match"],
    ];
    assert_eq!(fields, [&json!(ONE), &json!(1), &json!(true)]);
}

#[test]
fn a_failed_cloud_dictation_gives_its_faults_kind_and_never_another_value() {
    let failed = FAILED_LINE;
    let log = [
        logged(0, &format!("{failed} {FAULT}=Rate {SERVICE}={ONE}")),
        logged(2, &format!("{failed} {FAULT}=Key {SERVICE}={ONE}")),
        logged(3, &format!("{failed} {FAULT}=a/b {SERVICE}={ONE}")),
        logged(4, &format!("another line {FAULT}=Credit")),
        logged(5, &format!("{failed} {SERVICE}={ONE}")),
    ]
    .join("\n");
    let since = stamp_us("2026-11-16T10:00:01.000000Z").expect("a stamp");
    assert_eq!(wrote::faults(&log, since), ["Key"]);
}

#[test]
fn a_key_that_could_not_be_read_before_the_send_gives_the_key_kind_and_nothing_of_its_line() {
    let failed = FAILED_LINE;
    let path = r"C:\daily\voice\keys\alpha.sealed";
    let log = [
        logged(1, &format!("{KEY_UNREAD} error={path}")),
        logged(2, &format!("{failed} {FAULT}=Rate {SERVICE}={ONE}")),
        logged(3, "a cloud key went error=Key"),
    ]
    .join("\n");
    let since = stamp_us("2026-11-16T10:00:00.000000Z").expect("a stamp");
    let got = wrote::faults(&log, since);
    assert_eq!(got, [CloudFault::Key.kind(), CloudFault::Rate.kind()]);
    assert!(!got.concat().contains("daily"), "never its value: {got:?}");
}
