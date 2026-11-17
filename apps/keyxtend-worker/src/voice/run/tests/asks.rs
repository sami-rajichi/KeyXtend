//! The library's side: a broken or panicking library, asks answered by notes, and a library without its card part
//! (edge case 28), which offers the processor alone.

use super::*;

#[test]
fn without_its_processor_part_the_engine_answers_runner() {
    let (mut engine, lib, tell, rx) = rig(0);
    lib.script().broken = true;
    let mut pen = engine
        .load(1, &model(Target::Cpu, "en", 0), tell.clone())
        .unwrap();
    pen.end(0);
    assert_eq!(next(&rx), fault(1, VoiceFault::Runner));
    engine.devices(2, tell).unwrap();
    assert_eq!(next(&rx), fault(2, VoiceFault::Runner));
    assert_eq!(lib.script().starts, 1, "the parts load once");
}

#[test]
fn a_panicking_library_tells_an_internal_fault_and_the_engine_goes_on() {
    let (mut engine, lib, tell, rx) = rig(0);
    lib.script().panic = true;
    let _pen = engine
        .load(1, &model(Target::Cpu, "en", 0), tell.clone())
        .unwrap();
    assert_eq!(
        run_notes(&rx, 1),
        [loading(1), fault(1, VoiceFault::Internal)]
    );
    lib.script().panic = false;
    engine.devices(2, tell).unwrap();
    assert!(matches!(next(&rx), VoiceNote::Devices { id: 2, .. }));
}

#[test]
fn devices_facts_and_checks_are_answered_by_notes() {
    let (mut engine, lib, tell, rx) = rig(4000);
    lib.script()
        .found
        .insert(0, device(DeviceKind::BuiltIn, "Vulkan1", 1000));
    engine.devices(1, tell.clone()).unwrap();
    let VoiceNote::Devices { list, .. } = next(&rx) else {
        panic!("no devices");
    };
    let kinds: Vec<DeviceKind> = list.iter().map(|d| d.kind).collect();
    assert_eq!(
        kinds,
        [DeviceKind::Cpu, DeviceKind::BuiltIn, DeviceKind::Card]
    );
    engine.facts(2, &file(), tell.clone()).unwrap();
    assert!(matches!(next(&rx), VoiceNote::Facts { id: 2, .. }));
    let clip = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/voice/check-fr.wav");
    let clip = clip.to_string_lossy();
    engine
        .check(3, &model(card(1000), "fr-FR", 0), &clip, tell)
        .unwrap();
    let notes: Vec<VoiceNote> = (0..4).map(|_| next(&rx)).collect();
    assert_eq!(notes[0], loading(3));
    assert!(matches!(notes[3], VoiceNote::Checked { id: 3, .. }));
}

#[test]
fn a_library_without_its_card_part_offers_the_processor_alone_and_still_serves() {
    let (mut engine, lib, tell, rx) = rig(4000);
    lib.script().found.truncate(1);
    engine.devices(1, tell.clone()).unwrap();
    let VoiceNote::Devices { list, .. } = next(&rx) else {
        panic!("no devices");
    };
    let kinds: Vec<DeviceKind> = list.iter().map(|d| d.kind).collect();
    assert_eq!(kinds, [DeviceKind::Cpu]);
    let notes = write(
        &mut engine,
        &tell,
        &rx,
        2,
        &model(card(1000), "en", 0),
        &[10],
    );
    assert_eq!(
        notes[..3],
        [
            fault(2, VoiceFault::NoCard),
            loading(2),
            piece(2, 0, "10 en")
        ]
    );
    assert_eq!(lib.script().loads, [(Place::Cpu, 8)]);
}
