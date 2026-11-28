// The question Close asks when the snip was neither copied nor saved (design "Close", snip-editor.js ask()): its own
// window centred over the editor, or over the part while the strip shows, with Save, Don't save and Back. Save says
// why it waits; in a picture's editor it flattens the picture first, as the editor's Save does.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: glow

    // Snip's windows.
    required property var snips

    readonly property var sh: lk.shape.snip
    readonly property var n: snips.names
    readonly property var v: snips.view
    // The question as the view gives it, kept while it hides so nothing reads one that is gone.
    property var ask: null
    readonly property var editor: snips.editor
    readonly property bool editing: !!v && v.stage === n.stages.edit
    readonly property bool shows: !!v && !!v.ask && (editing ? editor.editor.visible : snips.up(n.stages.strip, n.capture.strip))
    // The box it sits over and the box it stays in, in Qt units.
    readonly property rect over: editing ? Qt.rect(editor.editor.x, editor.editor.y, editor.editor.width, editor.editor.height) : v && v.strip ? snips.box(v.strip.part) : Qt.rect(0, 0, 0, 0)
    readonly property rect area: !editing && v && v.strip ? snips.box(v.strip.screen) : over
    // The question's window, for the tests.
    readonly property alias question: win

    lk: snips.lk
    res: snips.res
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput | (snips.low ? 0 : Qt.WindowStaysOnTopHint)
    box: Qt.rect(win.x, win.y, win.width, win.height)
    radius: lk.look.window_radius_px
    follow: card
    shadowed: win
    visible: shows

    onVChanged: if (v && v.ask) ask = v.ask

    // Answer `id`: Save in a picture's editor flattens it first; every other answer goes by its key.
    function answer(id) {
        if (id === n.answer.save && editing && editor.pic)
            editor.keep(n.keep.save);
        else
            snips.act(n.acts.ask, id);
    }
    // Where a span `size` long starts, centred on the one at `from`, `len` long, and kept inside `start` and `room`.
    function centred(from, len, start, room, size) {
        return Math.max(start, Math.min(from + (len - size) / 2, start + room - size));
    }

    Window {
        id: win

        // Set just after its shadow's flags, so it stays above its shadow (Glow.owns).
        flags: glow.owns
        title: glow.res.tr("window.snip-ask")
        color: "transparent"
        // Made once, its shadow first, before it ever shows, so it starts over its shadow and owned by it.
        Component.onCompleted: glow.snips.snip.make(title)
        x: Math.round(glow.centred(glow.over.x, glow.over.width, glow.area.x, glow.area.width, width))
        y: Math.round(glow.centred(glow.over.y, glow.over.height, glow.area.y, glow.area.height, height))
        width: Math.ceil(glow.sh.ask_px[0])
        height: Math.ceil(body.implicitHeight + 2 * glow.sh.ask_px[1])

        onVisibleChanged: if (visible && glow.lk.motion.moves.snip_pop.ms > 0) pop.restart()

        Rectangle {
            id: card
            objectName: "card"
            width: win.width
            height: win.height
            radius: glow.lk.look.window_radius_px
            color: glow.lk.palette.pop_bg
            border.width: glow.lk.shape.line.px
            border.color: glow.lk.palette.pop_line
            Accessible.role: Accessible.AlertMessage
            Accessible.name: said.text

            Column {
                id: body
                x: glow.sh.ask_px[1]
                y: glow.sh.ask_px[1]
                width: parent.width - 2 * glow.sh.ask_px[1]
                spacing: glow.sh.ask_px[1]
                Row {
                    width: parent.width
                    spacing: glow.sh.icon_gap_px
                    layoutDirection: Qt.LeftToRight
                    Icon {
                        icon: glow.lk.icons.snip.ask.mark
                        base: glow.res.icons
                        ext: glow.res.ext
                        px: glow.sh.icon_px
                        tint: glow.lk.palette.pop_ink
                    }
                    Label {
                        id: said
                        width: parent.width - glow.sh.icon_px - parent.spacing
                        text: glow.res.tr("snip.ask")
                        wrapMode: Text.Wrap
                        font.family: glow.res.fam.ui
                        font.pixelSize: glow.lk.shape.clip.button_px[3]
                        font.weight: glow.lk.shape.clip.button_weight
                        color: glow.lk.palette.pop_ink
                    }
                }
                Row {
                    spacing: glow.lk.shape.clip.button_px[5]
                    layoutDirection: Qt.LeftToRight
                    Repeater {
                        model: glow.n.ids.answers
                        delegate: SnipButton {
                            required property string modelData
                            readonly property bool saves: modelData === glow.n.answer.save
                            lk: glow.lk
                            res: glow.res
                            name: glow.res.tr("snip.ask-" + modelData)
                            icon: glow.lk.icons.snip.ask[modelData]
                            worded: true
                            tone: saves ? "enter" : "ghost"
                            why: saves && glow.ask && glow.ask.save_dim ? glow.res.tr("snip.dim-" + glow.ask.save_dim) : ""
                            uid: "snip.ask." + modelData
                            tip: (text, item) => tips.hint(text, item)
                            onClicked: glow.answer(modelData)
                        }
                    }
                }
            }
            // A press on the question aims the keyboard's letters at Snip; the press still reaches the key.
            PressWatch {
                onTouched: glow.snips.aim()
            }
        }
        PopIn {
            id: pop
            target: card
            move: glow.lk.motion.moves.snip_pop
            amount: glow.lk.motion.amount
        }
        Tip {
            id: tips
            transientParent: win
            lk: glow.lk
            res: glow.res
            ms: glow.snips.tipMs
            lift: () => {}
        }
    }
}
