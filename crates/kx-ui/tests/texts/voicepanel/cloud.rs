//! The Qt Quick tests' cloud panels: the Cloud page on, then with an Off row's sealed key ticked, and off, a card
//! asking and an old one with its warning, the Models page's Cloud side, and Compare with cloud models, ticked or not.

use super::{Shown, paged, shown};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    CardPart, CloudBand, CloudButton, CloudCard, CloudRow, CloudSheet, CloudWhy, CompareCloud,
    CompareMic, CompareRun, CompareSheet, Date, DeviceLabel, DeviceName, Fact, FactArg, FactValue,
    Hundredths, ModelRow, ModelSide, ModelState, ModelsSheet, RunStep, ServiceMark, ServiceName,
    ServiceRow, Unavailable, UsePrice, VoiceSheet,
};

/// The training warning's fact id, which the sample card and the warned rows carry.
pub(super) const TRAINS: &str = "trains";

/// A fact `id` with `args`.
fn fact(id: &str, args: Vec<FactArg>) -> Fact {
    Fact {
        id: id.to_owned(),
        args,
        warn: false,
    }
}

/// A fact's argument `name` of `value`.
fn arg(name: &str, value: FactValue) -> FactArg {
    FactArg {
        name: name.to_owned(),
        value,
    }
}

/// `h` hundredths of a dollar, as `name`.
fn usd(name: &str, h: u32) -> FactArg {
    let money = FactValue::Money {
        amount: Hundredths(h),
        currency: "USD".to_owned(),
    };
    arg(name, money)
}

/// A service by its own name.
pub(super) fn named(name: &str) -> ServiceName {
    ServiceName::Named(name.to_owned())
}

/// Groq's free allowance.
fn daily() -> Fact {
    let n = |name, v| arg(name, FactValue::Count(v));
    fact("daily", vec![n("hours", 8), n("per_min", 20)])
}

/// A service's row on the Cloud page, `warn`ing that it trains.
fn service(name: ServiceName, mark: ServiceMark, warn: bool, tick: bool) -> ServiceRow {
    let custom = name == ServiceName::Custom;
    ServiceRow {
        name,
        mark,
        untested: mark == ServiceMark::Ready,
        free: (!custom).then(daily),
        warn: warn.then(|| TRAINS.to_owned()),
        tick,
    }
}

/// The Cloud page on, with the ticked row's footer: Groq, ready with its key, or `ElevenLabs` when `sealed`, Off
/// with its key kept; either offers Remove key.
fn cloud(sealed: bool) -> VoiceSheet {
    let rows = vec![
        service(named("Groq"), ServiceMark::Ready, false, !sealed),
        service(named("DeepInfra"), ServiceMark::NeedsKey, false, false),
        service(named("OpenRouter"), ServiceMark::Off, true, false),
        service(ServiceName::Custom, ServiceMark::NeedsAddress, false, false),
        service(named("ElevenLabs"), ServiceMark::Off, true, sealed),
    ];
    let acts = if sealed {
        vec![CloudButton::Facts, CloudButton::Remove, CloudButton::On]
    } else {
        let on = [CloudButton::Lines, CloudButton::Remove, CloudButton::Off];
        [CloudButton::Facts].into_iter().chain(on).collect()
    };
    VoiceSheet::Cloud(CloudSheet {
        on: true,
        rows,
        acts,
    })
}

/// The Cloud page with cloud services off, which lists no service (edge 21).
fn cloud_off() -> VoiceSheet {
    VoiceSheet::Cloud(CloudSheet {
        on: false,
        rows: Vec::new(),
        acts: Vec::new(),
    })
}

/// Groq's card, read on `read`, asking or switched on, `old` with its warning band.
pub(super) fn card(old: bool) -> VoiceSheet {
    let day = |d| Date::new(2026, 11, d).expect("a day");
    let switch = arg(
        "switch",
        FactValue::Text("Settings → Data Controls".to_owned()),
    );
    let days = arg("days", FactValue::Count(30));
    let price = |low, high| fact("free-or", vec![usd("low", low), usd("high", high)]);
    let mut trains = fact(if old { "unknown" } else { "never" }, Vec::new());
    trains.warn = old;
    let warn = old.then(|| fact(TRAINS, vec![switch.clone()]));
    let secs = arg("seconds", FactValue::Count(10));
    VoiceSheet::Card(Box::new(CloudCard {
        service: named("Groq"),
        place: fact("us", Vec::new()),
        trains,
        keeps: fact("logs", vec![days, switch]),
        free: daily(),
        light: UsePrice {
            hours: 15,
            price: price(60, 167),
        },
        heavy: UsePrice {
            hours: 60,
            price: price(240, 666),
        },
        bills: Some(fact("least", vec![secs])),
        rule: None,
        read: day(if old { 2 } else { 17 }),
        old,
        site: Some("groq.com".to_owned()),
        warn,
        untested: !old,
        on: old.then(|| day(10)),
        part: CardPart::Voice,
    }))
}

/// A cloud model `name` of `service` at `mark`, maybe Unavailable.
fn model(name: &str, service: &str, mark: ServiceMark, na: bool) -> ModelRow {
    ModelRow {
        name: Redacted::new(name.to_owned()),
        bytes: 0,
        default: false,
        state: ModelState::Ready,
        used: false,
        tick: false,
        fetch: None,
        cloud: Some(CloudRow {
            service: named(service),
            mark,
            unavailable: na.then_some(Unavailable::KeepNothing),
        }),
    }
}

/// The Models page's Cloud side.
fn models() -> VoiceSheet {
    let mut first = model("Whisper Large v3 Turbo", "Groq", ServiceMark::Ready, false);
    first.tick = true;
    let rows = vec![
        first,
        model("voxtral-small", "OpenRouter", ServiceMark::Ready, true),
        model(
            "Nemotron 3.5 Transcribe",
            "DeepInfra",
            ServiceMark::NeedsKey,
            false,
        ),
    ];
    VoiceSheet::Models(ModelsSheet {
        rows,
        best: None,
        untested: false,
        slow: false,
        folder: None,
        offer: None,
        side: ModelSide::Cloud,
        band: Some(CloudBand::Listed),
    })
}

/// Groq's cloud model `name` on Compare, `tick`ed or not.
fn groq_pick(name: &str, tick: bool) -> CompareCloud {
    CompareCloud {
        name: Redacted::new(name.to_owned()),
        service: named("Groq"),
        tick,
    }
}

/// Compare's set text with its `cloud` models, warning that the recording goes to `send`.
fn ready(cloud: Vec<CompareCloud>, send: Vec<ServiceName>) -> VoiceSheet {
    VoiceSheet::Compare(CompareSheet::Ready {
        models: 2,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        mic: CompareMic::Closed,
        cloud,
        send,
    })
}

/// Compare's set text with a ticked cloud model and its warning.
fn compare() -> VoiceSheet {
    ready(
        vec![groq_pick("Whisper Large v3 Turbo", true)],
        vec![named("Groq")],
    )
}

/// Compare's set text with Groq's two models unticked, so no warning shows yet.
fn compare_unticked() -> VoiceSheet {
    let picks = ["Whisper Large v3 Turbo", "Whisper Large v3"];
    ready(picks.map(|n| groq_pick(n, false)).to_vec(), Vec::new())
}

/// Compare's run with a cloud model that failed on its key.
fn comparing() -> VoiceSheet {
    let run = |name: &str, step, why| CompareRun {
        name: Redacted::new(name.to_owned()),
        step,
        why,
    };
    VoiceSheet::Compare(CompareSheet::Running {
        at: 2,
        of: 2,
        name: Redacted::new("Whisper small".to_owned()),
        rows: vec![
            run(
                "Whisper Large v3 Turbo",
                RunStep::Failed,
                Some(CloudWhy::Key),
            ),
            run("Whisper small", RunStep::Writing, None),
        ],
    })
}

/// The cloud panels, by name.
pub fn panels() -> Vec<(&'static str, Shown)> {
    use kx_module_api::ports::PanelButton::{Cancel, Use};
    vec![
        ("cloud", paged(shown(cloud(false), &[]), 0, 1, 1)),
        ("cloudSealed", paged(shown(cloud(true), &[]), 0, 1, 1)),
        ("cloudOff", shown(cloud_off(), &[])),
        ("card", shown(card(false), &[])),
        ("cardOld", shown(card(true), &[])),
        ("modelsCloud", paged(shown(models(), &[Use]), 0, 1, 1)),
        ("compareCloud", shown(compare(), &[])),
        ("compareCloudUnticked", shown(compare_unticked(), &[])),
        ("comparingCloud", shown(comparing(), &[Cancel])),
    ]
}
