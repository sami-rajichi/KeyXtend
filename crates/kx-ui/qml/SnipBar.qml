// The capture bar, design B "Keycaps" (design/snip-parts.css .cbar.cb-keys): the keyboard's keycaps on its plate at the
// top centre of the pointer's screen, Region, Window and Full screen, then the two video ways with their red light, then
// Snip settings and Close as icons alone. Its shadow, window and tip share one title, so one shield keeps all out.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: glow

    // Snip's windows.
    required property var snips

    readonly property var sh: lk.shape.snip
    readonly property var n: snips.names
    readonly property var v: snips.view
    readonly property var pick: v && v.stage === n.stages.pick ? v.pick : null
    // The keys in the spec's order as the view gives them, kept while the bar hides so none reads a key that is gone.
    property var keys: []
    // The bar's window, for the tests; it shows while picking, once the frozen screens are up.
    readonly property alias bar: win
    readonly property bool shows: !!pick && snips.up(n.stages.pick, n.capture.bar)
    // It drew and its pop ended.
    readonly property bool ready: drawn && !pop.running

    title: res.tr("window.snip-bar")
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput | (snips.low ? 0 : Qt.WindowStaysOnTopHint)
    lk: snips.lk
    res: snips.res
    box: Qt.rect(win.x, win.y, win.width, win.height)
    radius: lk.look.window_radius_px
    follow: plate
    shadowed: win
    visible: shows

    onPickChanged: if (pick) keys = pick.keys

    SnipOpen {
        snips: glow.snips
        shadow: glow
        ready: glow.ready
        stage: glow.n.stages.pick
    }

    Window {
        id: win

        readonly property rect screenBox: glow.pick ? glow.snips.box(glow.pick.screen) : Qt.rect(0, 0, 0, 0)

        // Set just after its shadow's flags, so it stays above its shadow (Glow.owns).
        flags: glow.owns
        title: glow.title
        color: "transparent"
        x: Math.round(screenBox.x + (screenBox.width - width) / 2)
        y: Math.round(screenBox.y + glow.sh.bar_top_px)
        width: Math.ceil(row.width + 2 * glow.sh.plate_pad_px)
        height: Math.ceil(glow.sh.key_px + 2 * glow.sh.plate_pad_px)

        // Made and kept out of every capture before it ever shows.
        Component.onCompleted: glow.snips.snip.shield(glow.n.capture.bar, title)
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
                Repeater {
                    model: glow.n.bar
                    delegate: Row {
                        id: part
                        required property var modelData
                        required property int index
                        layoutDirection: Qt.LeftToRight
                        // The gap before each group but the first (.cb-keys .sep).
                        Item {
                            width: part.index > 0 ? glow.sh.sep_px : 0
                            height: 1
                        }
                        Repeater {
                            model: part.modelData.keys
                            delegate: SnipKey {
                                required property string modelData
                                readonly property var cap: glow.keys.find(k => k.key === modelData) || null
                                visible: !!cap
                                lk: glow.lk
                                res: glow.res
                                group: glow.n.acts.bar
                                key: modelData
                                word: glow.res.tr("snip.way-" + modelData)
                                icon: glow.lk.icons.snip.bar[modelData] || ""
                                worded: part.modelData.worded
                                pick: part.modelData.picks
                                on: !!cap && cap.on
                                why: cap && cap.dim ? glow.res.tr("snip.dim-" + cap.dim) : ""
                                lit: part.modelData.lit
                                tip: (text, item) => tips.hint(text, item)
                                onHit: glow.snips.act(glow.n.acts.bar, modelData)
                            }
                        }
                    }
                }
            }
            // A press on the bar aims the keyboard's letters at Snip; the press still reaches the key.
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
