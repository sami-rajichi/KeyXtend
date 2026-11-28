// The Snip panel (design "The Snip panel", snip-editor.js panel()): Snip's own settings above the keyboard at its size,
// with a Capture page and a Video page, opened from the capture bar or the tray. It never takes focus, and it hides
// while the keyboard is aside and Hidden, as every window of ours but the capture's does.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: glow

    // Snip's windows.
    required property var snips

    readonly property var win: snips.win
    readonly property var sh: lk.shape.snip
    readonly property var n: snips.names
    readonly property var v: snips.view
    // The page as the view gives it, kept while the panel hides so nothing reads one that is gone.
    property var page: null
    readonly property bool shows: !!v && !!v.panel && !!win && !snips.hush
    // The keyboard's size, which the panel takes.
    readonly property real s: win ? win.s : 1
    // The panel's window, for the tests.
    readonly property alias pane: pane

    lk: snips.lk
    res: snips.res
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput | (snips.low ? 0 : Qt.WindowStaysOnTopHint)
    box: Qt.rect(pane.x, pane.y, pane.width, pane.height)
    radius: lk.look.window_radius_px
    follow: card
    shadowed: pane
    visible: shows

    onVChanged: if (v && v.panel) page = v.panel

    // Shows a tip over `item`, or hides it for an empty `text`.
    function hint(text, item) {
        tips.hint(text, item);
    }
    // Places it above the keyboard, centred, as a pop-up goes.
    function place() {
        const at = win ? win.spots.popSpot(pane.width, pane.height, NaN) : null;
        if (!at)
            return;
        pane.x = Math.round(at.x);
        pane.y = Math.round(at.y);
    }

    // Placed again as the keyboard moves or changes size.
    Connections {
        target: glow.win
        function onAtChanged() {
            glow.place();
        }
        function onDrawnChanged() {
            glow.place();
        }
    }

    Window {
        id: pane

        // Set just after its shadow's flags, so it stays above its shadow (Glow.owns).
        flags: glow.owns
        title: glow.res.tr("window.snip-panel")
        color: "transparent"
        width: Math.ceil(glow.sh.panel_px * glow.s)
        height: Math.ceil(body.implicitHeight)

        // Made once, its shadow first, before it ever shows, so it starts over its shadow and owned by it.
        Component.onCompleted: glow.snips.snip.make(title)
        onHeightChanged: glow.place()
        onVisibleChanged: {
            if (!visible)
                return;
            glow.place();
            if (glow.lk.motion.moves.snip_pop.ms > 0)
                pop.restart();
        }

        Rectangle {
            id: card
            width: pane.width
            height: pane.height
            radius: glow.lk.look.window_radius_px
            color: glow.lk.palette.pop_bg
            border.width: glow.lk.shape.line.px
            border.color: glow.lk.palette.pop_line
            Accessible.role: Accessible.Pane
            Accessible.name: pane.title

            Column {
                id: body
                width: parent.width
                // Snip's mark and the panel's name, then Close (.ph).
                Item {
                    width: body.width
                    height: glow.sh.editor_rows_px[0] * glow.s
                    Row {
                        x: glow.sh.editor_pad_px[0] * glow.s
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: glow.sh.icon_gap_px * glow.s
                        layoutDirection: Qt.LeftToRight
                        Icon {
                            anchors.verticalCenter: parent.verticalCenter
                            icon: glow.lk.icons.keys.snip
                            base: glow.res.icons
                            ext: glow.res.ext
                            px: glow.sh.icon_px * glow.s
                            tint: glow.lk.palette.pop_ink
                        }
                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            text: glow.res.tr("snip.way-settings")
                            font.family: glow.res.fam.ui
                            font.pixelSize: glow.lk.shape.clip.button_px[3] * glow.s
                            font.weight: glow.lk.shape.clip.button_weight
                            color: glow.lk.palette.pop_ink
                        }
                    }
                    SnipButton {
                        anchors.right: parent.right
                        anchors.rightMargin: glow.sh.editor_pad_px[0] * glow.s
                        anchors.verticalCenter: parent.verticalCenter
                        lk: glow.lk
                        res: glow.res
                        s: glow.s
                        name: glow.res.tr("snip.way-close")
                        icon: glow.lk.icons.snip.edit.close
                        uid: "snip.panel.close"
                        tip: glow.hint
                        onClicked: glow.snips.act(glow.n.acts.close, "")
                    }
                }
                // The two pages (.tabs).
                Row {
                    x: glow.sh.editor_pad_px[0] * glow.s
                    spacing: glow.lk.shape.clip.button_px[5] * glow.s
                    layoutDirection: Qt.LeftToRight
                    Repeater {
                        model: glow.n.ids.tabs
                        delegate: SnipChoice {
                            required property string modelData
                            panel: glow
                            name: glow.res.tr("snip.panel-tab-" + modelData)
                            on: !!glow.page && glow.page.tab === modelData
                            uid: "snip.tab." + modelData
                            onClicked: glow.snips.act(glow.n.acts.tab, modelData)
                        }
                    }
                }
                SnipPage {
                    width: body.width
                    panel: glow
                }
                Item {
                    width: 1
                    height: glow.sh.editor_pad_px[0] * glow.s
                }
            }
            // A press on the panel aims the keyboard's letters at Snip; the press still reaches the key.
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
            transientParent: pane
            lk: glow.lk
            res: glow.res
            ms: glow.snips.tipMs
            lift: () => {}
        }
    }
}
