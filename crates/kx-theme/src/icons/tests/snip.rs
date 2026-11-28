//! Snip's icons (design/snip-scene.js and snip-editor.js, but Restore's is the owner's pick and Speed's a stand-in):
//! each key by the hub's or the view's own id, the tools, Crop's ✓ and ✗, and the tray row.

use super::*;

/// Every name as `group.id=name`, without the `snip.` prefix.
fn names(i: &Icons) -> Vec<String> {
    let named = i.snip.named().into_iter();
    named
        .map(|(at, n)| format!("{}={n}", at.trim_start_matches("snip.")))
        .collect()
}

#[test]
fn each_snip_key_and_tool_shows_its_chosen_icon() {
    let i = parse(&text()).expect("icons load");
    let want = "tick=check length=film mic_off=mic-off sound_off=volume-x \
                bar.close=x bar.full=monitor bar.region=square-dashed bar.settings=settings-2 \
                bar.video-full=monitor-play bar.video-region=video bar.window=app-window \
                strip.close=x strip.copy=copy strip.edit=square-pen strip.mic=mic strip.play=play \
                strip.record=circle-dot strip.save=save strip.sound=volume-2 strip.speed=chevron-right \
                rec.cancel=x rec.mic=mic rec.sound=volume-2 rec.stop=square \
                tools.arrow=move-up-right tools.box=square tools.circle=circle tools.cover=eye-off \
                tools.crop=crop tools.eraser=eraser tools.highlighter=highlighter tools.line=slash \
                tools.pen=pencil tools.steps=list-ordered tools.text=type edit.back=minimize-2 edit.close=x \
                edit.full=maximize-2 edit.less=minus edit.more=plus edit.move=grip-horizontal \
                edit.redo=redo-2 edit.size=move-diagonal-2 edit.undo=undo-2 crop.cancel=x crop.ok=check \
                ask.back=undo-2 ask.drop=trash-2 ask.mark=save ask.save=save \
                panel.change=folder panel.open=folder-open";
    let want: Vec<&str> = want.split_whitespace().collect();
    assert_eq!(
        names(&i),
        want,
        "the mock-up's, but Restore's is the owner's pick and Speed's a stand-in"
    );
    assert_eq!(
        i.tray.snip, "scissors-line-dashed",
        "the Snip panel's title icon"
    );
    assert!(
        i.named()
            .contains(&("snip.bar.region".into(), "square-dashed"))
    );
}

#[test]
fn a_bad_snip_icon_is_refused_with_where_it_is_used() {
    let t = text().replacen(
        "video-full = \"monitor-play\"",
        "video-full = \"Monitor\"",
        1,
    );
    let e = parse(&t).expect_err("a capital");
    assert!(e.contains("snip.bar.video-full"), "{e}");
}
