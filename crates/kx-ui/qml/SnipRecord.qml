// The record bar above the part (design/snip-parts.css .recbar), or at the top of the screen when no room is above: a
// pill with the pulsing dot and the time, the Mic and Sound switches, Stop and Cancel, where a press never aims the
// keyboard's letters at Snip. Its shadow, window and tip share one title, so one shield keeps all out.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: glow

    // Snip's windows, and the clock in ms.
    required property var snips
    property var now: () => Date.now()

    readonly property var sh: lk.shape.snip
    readonly property var n: snips.names
    readonly property var v: snips.view
    readonly property var rec: v && v.stage === n.stages.record ? v.record : null
    // The keys and time as the view gives them, kept while the bar hides, when that time came on the clock, whether the
    // run recorded its first frame, before which the clock holds at nought, and whether the next view starts afresh.
    property var keys: []
    property real elapsed: 0
    property real heard: 0
    property bool live: false
    property bool fresh: true
    // The bar's window, for the tests; it shows while recording.
    readonly property alias bar: win
    readonly property bool shows: !!rec && snips.up(n.stages.record, n.capture.record)

    title: res.tr("window.snip-record")
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput | (snips.low ? 0 : Qt.WindowStaysOnTopHint)
    lk: snips.lk
    res: snips.res
    box: Qt.rect(win.x, win.y, win.width, win.height)
    radius: win.height / 2
    follow: pill
    shadowed: win
    visible: shows

    // A new recording, its first frame, or a new time from the module sets the clock; a view for another reason keeps
    // it counting.
    onRecChanged: {
        if (!rec) {
            fresh = true;
            return;
        }
        keys = rec.keys;
        if (fresh || rec.elapsed_ms !== elapsed || rec.live !== live) {
            elapsed = rec.elapsed_ms;
            heard = now();
            live = rec.live;
            fresh = false;
        }
    }

    // Where it goes in Qt units: centred above the part, or at the top of its screen, on that screen.
    function place(w, h) {
        if (!rec)
            return Qt.point(0, 0);
        const [part, area] = [snips.box(rec.part), snips.box(rec.screen)];
        const y = rec.spot === n.spots.above ? part.y - sh.gap_px - h : area.y + sh.bar_top_px;
        const x = part.x + (part.width - w) / 2;
        return Qt.point(Math.max(area.x, Math.min(x, area.x + area.width - w)), y);
    }
    // Key `cap`'s icon, as Snip's windows choose it from the record bar's.
    function iconOf(cap) {
        return snips.iconOf(cap, lk.icons.snip.rec);
    }

    Window {
        id: win

        readonly property point at: glow.place(width, height)

        // Set just after its shadow's flags, so it stays above its shadow (Glow.owns).
        flags: glow.owns
        title: glow.title
        color: "transparent"
        x: Math.round(at.x)
        y: Math.round(at.y)
        // Padding 0 4 0 12: the end's, then the start's.
        width: Math.ceil(row.width + glow.sh.rec_pad_px[0] + glow.sh.rec_pad_px[1])
        height: Math.ceil(glow.sh.rec_px)

        // Made and kept out of every capture before it ever shows.
        Component.onCompleted: glow.snips.snip.shield(glow.n.capture.record, title)
        onVisibleChanged: if (visible && glow.lk.motion.moves.snip_pop.ms > 0) pop.restart()

        Rectangle {
            id: pill
            width: win.width
            height: win.height
            radius: height / 2
            color: glow.lk.palette.pop_bg
            border.width: glow.lk.contrast ? glow.lk.shape.line.px : 0
            border.color: glow.lk.palette.pop_line
            Accessible.role: Accessible.ToolBar
            Accessible.name: win.title

            Row {
                id: row
                x: glow.sh.rec_pad_px[1]
                anchors.verticalCenter: parent.verticalCenter
                spacing: glow.sh.rec_gap_px
                layoutDirection: Qt.LeftToRight
                VoiceDot {
                    id: dot
                    anchors.verticalCenter: parent.verticalCenter
                    lk: glow.lk
                    s: 1
                    px: glow.sh.rec_dot_px
                    move: glow.lk.motion.moves.snip_dot
                }
                Item {
                    width: glow.sh.rec_time_gap_px - glow.sh.rec_gap_px
                    height: 1
                }
                // It counts only while recording, so no tick runs while the bar hides, and holds at nought until the
                // first frame.
                VoiceTimer {
                    id: time
                    visible: !!glow.rec
                    runs: glow.live
                    anchors.verticalCenter: parent.verticalCenter
                    lk: glow.lk
                    res: glow.res
                    s: 1
                    format: glow.res.tr("snip.time")
                    from: glow.elapsed
                    at: glow.heard
                    now: glow.now
                    font.pixelSize: glow.sh.rec_time_px
                    font.weight: glow.sh.rec_time_weight
                    color: glow.lk.palette.pop_ink
                    Accessible.ignored: false
                    Accessible.role: Accessible.StaticText
                    Accessible.name: glow.res.tr("snip.live") + glow.res.tr("snip.join") + text
                }
                Repeater {
                    model: glow.keys
                    delegate: Row {
                        id: slot
                        required property var modelData
                        required property int index
                        readonly property bool flips: glow.n.switches.indexOf(modelData.key) >= 0
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: glow.sh.rec_gap_px
                        layoutDirection: Qt.LeftToRight
                        // A line before the switches and before Stop (.recbar .sep).
                        Rectangle {
                            visible: slot.index === 0 || slot.modelData.key === glow.n.stop
                            anchors.verticalCenter: parent.verticalCenter
                            width: glow.lk.shape.line.px
                            height: glow.sh.rec_px - 2 * glow.sh.rec_sep_px
                            color: glow.lk.palette.pop_line
                        }
                        SnipPill {
                            lk: glow.lk
                            res: glow.res
                            key: slot.modelData.key
                            word: glow.res.tr("snip.rec-" + slot.modelData.key)
                            icon: glow.iconOf(slot.modelData)
                            worded: slot.modelData.key === glow.n.stop
                            toggle: slot.flips
                            on: slot.flips && slot.modelData.on
                            why: slot.modelData.dim ? glow.res.tr("snip.dim-" + slot.modelData.dim) : ""
                            stop: slot.modelData.key === glow.n.stop
                            tip: (text, item) => tips.hint(text, item)
                            onHit: glow.snips.act(glow.n.acts.rec, slot.modelData.key)
                        }
                    }
                }
            }
        }
        PopIn {
            id: pop
            target: pill
            move: glow.lk.motion.moves.snip_pop
            amount: glow.lk.motion.amount
        }
        // Its own tip, which shares its title, so it is kept out with it.
        Tip {
            id: tips
            transientParent: win
            title: glow.title
            lk: glow.lk
            res: glow.res
            ms: glow.snips.tipMs
            lift: () => {}
        }
    }
}
