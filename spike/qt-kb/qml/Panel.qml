// The Arabic test panel for gate G7 (mock-up .pop): a right-to-left stand-in for Settings, the one window that takes focus.
// A header with a close button, a paged list and a text field; its shadow is a click-through window just below it.
pragma ComponentBehavior: Bound
import QtQuick
import KeyXtend.Spike

Item {
    id: root

    // The look and fonts, the keyboard size, and Qt units for a physical point (the keyboard's `logical`).
    required property var lk
    required property var res
    required property real s
    required property var logical

    // A note for the status line.
    signal note(string text)
    // The pointer came onto a button (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property var fx: JSON.parse(pd.fixedJson())
    readonly property var sh: lk.shape.panel
    readonly property var pal: lk.palette
    readonly property string fam: fx.rtl ? res.fam.arabic : res.fam.latin
    // The page shown, from 0.
    property int at: 0
    readonly property var pg: JSON.parse(pd.pageJson(at))

    PanelData { id: pd }

    // Opens the panel by the keyboard, whose plate is `plate` ({w, h} in Qt units), or closes it when open.
    function toggle(plate) {
        if (win.visible) {
            close();
            return;
        }
        // A hidden window is not laid out, so a new size or theme would leave its height stale.
        col.forceLayout();
        const p = JSON.parse(pd.spot(plate.w, win.width, win.height));
        if (p.note) {
            note(p.note);
            return;
        }
        const l = logical(p.x, p.y);
        win.x = l.x;
        win.y = l.y;
        at = 0;
        list.picked = -1;
        field.clear();
        pd.opened();
        // The shadow owns the panel, so it is shown first.
        glow.visible = true;
        win.visible = true;
        win.requestActivate();
        field.forceActiveFocus();
    }

    // Gives focus back to the app that had it while the panel is still up, then hides the panel and its shadow.
    function close() {
        tip("", null);
        pd.closed();
        win.visible = false;
        glow.visible = false;
    }

    Glow {
        id: glow
        // Not owned by the keyboard: Settings is a window of its own.
        transientParent: null
        lk: root.lk
        box: Qt.rect(win.x, win.y, win.width, win.height)
        radius: win.radius
        follow: popped

        Window {
            id: win
            readonly property real radius: root.lk.look.window_radius_px * root.sh.radius_share

            flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
            title: root.fx.title
            color: "transparent"
            visible: false
            width: root.sh.width_px * root.s
            height: col.height

            // It pops in each time it shows (mock-up .pop).
            onVisibleChanged: if (visible) pop.restart()

            PopIn {
                id: pop
                target: popped
                move: root.lk.motion.moves.panel
                amount: root.lk.motion.amount
            }
            Item {
                id: popped
                width: win.width
                height: win.height
                LayoutMirroring.enabled: root.fx.rtl
                LayoutMirroring.childrenInherit: true

                Rectangle {
                    anchors.fill: parent
                    radius: win.radius
                    color: root.pal.pop_bg
                    // High contrast has no shadow, so the panel gets an edge instead.
                    border.width: root.lk.contrast ? root.lk.shape.line.px : 0
                    border.color: root.pal.pop_line
                }
                Column {
                    id: col
                    width: parent.width

                    Item {
                        id: head
                        readonly property var pad: root.sh.head_pad_px
                        width: col.width
                        height: (pad[0] + root.sh.button_px[0] + pad[2]) * root.s

                        Row {
                            anchors.left: parent.left
                            anchors.leftMargin: head.pad[3] * root.s
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: root.sh.head_gap_px * root.s

                            Icon {
                                anchors.verticalCenter: parent.verticalCenter
                                icon: root.fx.icon
                                base: root.res.icons
                                ext: root.res.ext
                                px: root.sh.button_px[2] * root.s
                                tint: root.pal.pop_ink
                            }
                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: root.fx.title
                                textFormat: Text.PlainText
                                font.family: root.fam
                                font.pixelSize: root.sh.title_px * root.s
                                font.weight: root.sh.title_weight
                                color: root.pal.pop_ink
                                Accessible.role: Accessible.Heading
                                Accessible.name: text
                            }
                        }
                        PopButton {
                            anchors.right: parent.right
                            anchors.rightMargin: head.pad[1] * root.s
                            anchors.verticalCenter: parent.verticalCenter
                            width: root.sh.button_px[0] * root.s
                            height: width
                            radius: root.sh.button_px[1] * root.s
                            iconPx: root.sh.button_px[2] * root.s
                            lk: root.lk
                            res: root.res
                            button: root.fx.close
                            onClicked: root.close()
                            onTip: (name, at) => root.tip(name, at)
                        }
                    }
                    Rectangle {
                        width: col.width
                        height: root.lk.shape.line.px
                        color: root.pal.pop_line
                    }
                    PanelList {
                        id: list
                        width: col.width
                        height: implicitHeight
                        lk: root.lk
                        res: root.res
                        fam: root.fam
                        s: root.s
                        fx: root.fx
                        pg: root.pg
                        onTip: (name, at) => root.tip(name, at)
                        // A page button at the end disables itself, so the field takes the focus back.
                        onTurn: n => {
                            root.at = n;
                            field.forceActiveFocus();
                        }
                    }
                    Rectangle {
                        width: col.width
                        height: root.lk.shape.line.px
                        color: root.pal.pop_line
                    }
                    Item {
                        readonly property var pad: root.sh.foot_pad_px
                        width: col.width
                        height: (2 * pad[0] + root.sh.field_px[0]) * root.s

                        Rectangle {
                            anchors.fill: parent
                            anchors.topMargin: parent.pad[0] * root.s
                            anchors.bottomMargin: anchors.topMargin
                            anchors.leftMargin: parent.pad[1] * root.s
                            anchors.rightMargin: anchors.leftMargin
                            radius: root.sh.field_px[1] * root.s
                            // High contrast paints pop_hover as Highlight, so the field rests on the panel colour there.
                            color: root.lk.contrast ? root.pal.pop_bg : root.pal.pop_hover
                            border.width: field.activeFocus ? root.lk.shape.press.ring_px : root.lk.shape.line.px
                            border.color: field.activeFocus ? root.pal.ring : root.pal.pop_line

                            TextInput {
                                id: field
                                anchors.fill: parent
                                anchors.leftMargin: root.sh.field_px[2] * root.s
                                anchors.rightMargin: anchors.leftMargin
                                // Set, not left to the input language, so mirroring puts an empty field's caret on the right.
                                horizontalAlignment: TextInput.AlignLeft
                                verticalAlignment: TextInput.AlignVCenter
                                font.family: root.fam
                                font.pixelSize: root.sh.field_px[3] * root.s
                                color: root.pal.pop_ink
                                selectionColor: root.pal.pop_sel
                                selectedTextColor: root.lk.sel_ink
                                clip: true
                                Accessible.name: root.fx.field
                                Keys.onEscapePressed: root.close()

                                Text {
                                    anchors.fill: parent
                                    visible: field.text.length === 0 && field.preeditText.length === 0
                                    text: root.fx.hint
                                    textFormat: Text.PlainText
                                    font: field.font
                                    color: root.pal.pop_muted
                                    horizontalAlignment: Text.AlignLeft
                                    verticalAlignment: Text.AlignVCenter
                                    Accessible.ignored: true
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
