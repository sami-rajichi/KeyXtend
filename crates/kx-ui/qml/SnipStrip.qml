// The strip under the part taken (design/snip-parts.css .strip): keycaps on the keyboard's plate, Copy, Edit, Save and
// Close for a picture, a recorded video's length then Play, Speed with its speed, Copy, Save and Close (ADR-0060), or
// Record, Mic, Sound and Close before a video, a key that acted with its tick and done word, and one that waits for the
// video's copy at its speed. Its shadow, window and tip share one title, so one shield keeps all out.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: glow

    // Snip's windows.
    required property var snips

    readonly property var sh: lk.shape.snip
    readonly property var n: snips.names
    readonly property var v: snips.view
    readonly property var st: v && v.stage === n.stages.strip ? v.strip : null
    // The keys, length and speed as the view gives them, kept while the strip hides so none reads a key that is gone.
    property var keys: []
    property var length: null
    property var speed: null
    // The strip's window, for the tests; it shows once the part is taken, and hides while it has no keys, as while a
    // frame's handle is held, so the setting click reaches the frozen screen.
    readonly property alias strip: win
    readonly property bool shows: !!st && st.keys.length > 0 && snips.up(n.stages.strip, n.capture.strip)

    title: res.tr("window.snip-strip")
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput | (snips.low ? 0 : Qt.WindowStaysOnTopHint)
    lk: snips.lk
    res: snips.res
    box: Qt.rect(win.x, win.y, win.width, win.height)
    radius: lk.look.window_radius_px
    follow: plate
    shadowed: win
    visible: shows

    onStChanged: if (st) {
        keys = st.keys;
        length = st.length_ms;
        speed = st.speed;
    }

    // Where it goes in Qt units: centred under the part, above it, or at the bottom centre of its screen, on that screen.
    function place(w, h) {
        if (!st)
            return Qt.point(0, 0);
        const [part, area, gap] = [snips.box(st.part), snips.box(st.screen), sh.gap_px];
        const x = st.spot === n.spots.bottom ? area.x + (area.width - w) / 2 : part.x + (part.width - w) / 2;
        const y = st.spot === n.spots.under ? part.y + part.height + gap : st.spot === n.spots.above ? part.y - gap - h : area.y + area.height - gap - h;
        return Qt.point(Math.max(area.x, Math.min(x, area.x + area.width - w)), y);
    }
    // Key `cap`'s icon: the tick once done, else as Snip's windows choose it from the strip's.
    function iconOf(cap) {
        return st && st.done === cap.key ? lk.icons.snip.tick : snips.iconOf(cap, lk.icons.snip.strip);
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
        width: Math.ceil(row.width + 2 * glow.sh.plate_pad_px)
        height: Math.ceil(glow.sh.key_px + 2 * glow.sh.plate_pad_px)

        // Made and kept out of every capture before it ever shows.
        Component.onCompleted: glow.snips.snip.shield(glow.n.capture.strip, title)
        onVisibleChanged: if (visible && glow.lk.motion.moves.snip_pop.ms > 0) pop.restart()

        Rectangle {
            id: plate
            width: win.width
            height: win.height
            radius: glow.lk.look.window_radius_px
            color: glow.lk.palette.plate
            border.width: glow.lk.contrast ? glow.lk.shape.line.px : 0
            border.color: glow.lk.palette.pop_line
            Accessible.role: Accessible.ToolBar
            Accessible.name: win.title

            Row {
                id: row
                x: glow.sh.plate_pad_px
                y: glow.sh.plate_pad_px
                layoutDirection: Qt.LeftToRight
                // A video's length before its keys (.strip .len), on the plate as no key.
                Item {
                    visible: glow.length !== null
                    width: visible ? len.width + 2 * glow.sh.key_pad_px[1] : 0
                    height: glow.sh.key_px
                    Accessible.role: Accessible.StaticText
                    Accessible.name: glow.res.tr("snip.length") + glow.res.tr("snip.join") + time.text
                    Row {
                        id: len
                        anchors.centerIn: parent
                        spacing: glow.sh.icon_gap_px
                        layoutDirection: Qt.LeftToRight
                        Icon {
                            anchors.verticalCenter: parent.verticalCenter
                            icon: glow.lk.icons.snip.length
                            base: glow.res.icons
                            ext: glow.res.ext
                            px: glow.sh.icon_px
                            tint: glow.lk.palette.legend_2_plate
                        }
                        VoiceTimer {
                            id: time
                            anchors.verticalCenter: parent.verticalCenter
                            lk: glow.lk
                            res: glow.res
                            s: 1
                            format: glow.res.tr("snip.time")
                            from: glow.length || 0
                            at: 0
                            runs: false
                            font.pixelSize: glow.sh.word_px
                            font.weight: glow.lk.look.legend_weight
                            color: glow.lk.palette.legend_2_plate
                        }
                    }
                }
                Repeater {
                    model: glow.keys
                    delegate: SnipKey {
                        required property var modelData
                        readonly property bool done: !!glow.st && glow.st.done === modelData.key
                        readonly property bool flips: glow.n.switches.indexOf(modelData.key) >= 0
                        lk: glow.lk
                        res: glow.res
                        group: glow.n.acts.strip
                        key: modelData.key
                        word: glow.res.tr(done ? "snip.done-" + modelData.key : "snip.strip-" + modelData.key)
                        icon: glow.iconOf(modelData)
                        on: flips && modelData.on
                        toggle: flips
                        value: modelData.key === glow.n.speed && glow.speed !== null ? glow.res.tr("snip.speed-times").arg(glow.speed) : ""
                        why: modelData.dim ? glow.res.tr("snip.dim-" + modelData.dim) : ""
                        say: modelData.key === glow.n.play ? glow.res.tr("snip.strip-play-say") : ""
                        waits: !!glow.st && glow.st.waits === modelData.key
                        // Record wears Enter's colours, on which no red light reaches 3:1, so it shows none.
                        enter: modelData.key === glow.n.record
                        tip: (text, item) => tips.hint(text, item)
                        onHit: glow.snips.act(glow.n.acts.strip, modelData.key)
                    }
                }
            }
            // A press on the strip aims the keyboard's letters at Snip; the press still reaches the key.
            PressWatch {
                onTouched: glow.snips.aim()
            }
        }
        PopIn {
            id: pop
            target: plate
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
