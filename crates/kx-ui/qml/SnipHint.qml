// The hint under the capture bar (design/snip-parts.css .hint), in the keyboard's language: how to pick with the way
// picked, or a note for a moment, such as why a video stopped; at the top centre of its screen when no bar shows. It lets
// every click through and never takes focus.
import QtQuick

Window {
    id: hint

    // Snip's windows.
    required property var snips

    readonly property var lk: snips.lk
    readonly property var res: snips.res
    readonly property var sh: lk.shape.snip
    readonly property var v: snips.view
    readonly property var n: snips.names
    readonly property bool picking: !!v && v.stage === n.stages.pick
    // What it says: the view's note, else while picking the hint for the picked way, or for the second corner.
    readonly property string said: v && v.note ? res.tr("snip.note-" + v.note) : picking ? wayHint() : ""
    // The screen whose top centre holds it: the live stage's, the one it last sat on, the frozen one, or the first one.
    readonly property rect screenBox: snips.box(screenOf())
    // The live stage's screen as last seen, so a note right after a snip ends, as Bare's after a countdown, stays there.
    property var seen: null

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput | (snips.low ? 0 : Qt.WindowStaysOnTopHint)
    title: res.tr("window.snip-hint")
    color: "transparent"
    visible: said.length > 0 && snips.up(snips.stage, n.capture.hint)
    x: Math.round(screenBox.x + (screenBox.width - width) / 2)
    y: Math.round(screenBox.y + sh.bar_top_px + (picking ? sh.key_px + 2 * sh.plate_pad_px + sh.hint_gap_px : 0))
    width: Math.ceil(text.implicitWidth + 2 * sh.hint_pad_px[1])
    height: Math.ceil(text.implicitHeight + 2 * sh.hint_pad_px[0])

    // Made and kept out of every capture before it ever shows.
    Component.onCompleted: snips.snip.shield(n.capture.hint, title)
    onVisibleChanged: if (visible && lk.motion.moves.snip_pop.ms > 0) pop.restart()
    onVChanged: seen = liveScreen() || seen

    // The hint for the picked way, or for the opposite corner once the first is clicked; none while a new view's stage
    // is still being read, as from picking straight to the strip.
    function wayHint() {
        if (!v.pick)
            return "";
        if (v.pick.corner)
            return res.tr("snip.hint-corner");
        const lit = v.pick.keys.find(k => k.on);
        return lit ? res.tr("snip.hint-" + lit.key) : "";
    }
    // The screen it sits on, in physical pixels.
    function screenOf() {
        const none = { x: 0, y: 0, w: 0, h: 0 };
        if (!v)
            return none;
        const frozen = v.frozen.length > 0 ? v.frozen[0].rect : null;
        return liveScreen() || seen || frozen || boxOf(snips.screens[0]) || none;
    }
    // The live stage's screen in physical pixels: its own, or the one holding the counting part's centre; null idle.
    function liveScreen() {
        const stage = v ? v[v.stage] : null;
        if (!stage)
            return null;
        if (stage.screen)
            return stage.screen;
        const r = stage.part;
        return r ? boxOf(snips.spots.screenOf(r.x + r.w / 2, r.y + r.h / 2, true)) : null;
    }
    // Screen `sc`'s box in physical pixels, null for none; Qt keeps each screen's corner unscaled, its size in Qt units.
    function boxOf(sc) {
        const d = sc ? sc.devicePixelRatio : 1;
        return sc ? { x: sc.virtualX, y: sc.virtualY, w: sc.width * d, h: sc.height * d } : null;
    }

    Rectangle {
        id: pill
        width: hint.width
        height: hint.height
        radius: height / 2
        color: hint.snips.tag
        // A note is read out as it shows, as Notice.qml's; a hint is read when asked.
        Accessible.role: hint.v && hint.v.note ? Accessible.AlertMessage : Accessible.StaticText
        Accessible.name: hint.said
        Label {
            id: text
            anchors.centerIn: parent
            text: hint.said
            font.family: hint.res.fam.ui
            font.pixelSize: hint.sh.hint_px
            color: hint.lk.common.snip_tag_ink
        }
    }
    PopIn {
        id: pop
        target: pill
        move: hint.lk.motion.moves.snip_pop
        amount: hint.lk.motion.amount
    }
}
