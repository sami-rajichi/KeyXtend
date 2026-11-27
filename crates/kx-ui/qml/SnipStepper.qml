// A − and + pair around a value (snip-editor.js stepper()): each key named for what it does, and dimmed with why at an
// end. The value shows as its number, or as a dot that wide for a line's thickness.
import QtQuick

Row {
    id: st

    // The look, texts and size; which stepper it is (`snip.less-<what>`), the value, the dot's width (0 for none), why
    // each end waits (empty while it acts), and what shows a tip.
    required property var lk
    required property var res
    property real s: 1
    required property string what
    required property string value
    property real dot: 0
    property string least: ""
    property string most: ""
    property var tip: (text, item) => {}

    // A click on − (-1) or + (1).
    signal stepped(int by)

    readonly property var sh: lk.shape.clip.button_px

    spacing: sh[5] * s
    layoutDirection: Qt.LeftToRight

    SnipButton {
        lk: st.lk
        res: st.res
        s: st.s
        name: st.res.tr("snip.less-" + st.what)
        icon: st.lk.icons.snip.edit.less
        why: st.least
        uid: "snip.less." + st.what
        tip: st.tip
        onClicked: st.stepped(-1)
    }
    Item {
        width: Math.max(st.sh[0] * st.s, said.width)
        height: st.sh[0] * st.s
        Accessible.role: Accessible.StaticText
        Accessible.name: st.value
        Label {
            id: said
            anchors.centerIn: parent
            visible: st.dot <= 0
            text: st.value
            font.family: st.res.fam.latin
            font.pixelSize: st.sh[3] * st.s
            font.weight: st.lk.shape.clip.button_weight
            color: st.lk.palette.pop_ink
        }
        Rectangle {
            anchors.centerIn: parent
            visible: st.dot > 0
            width: Math.min(st.dot, parent.height)
            height: width
            radius: width / 2
            color: st.lk.palette.pop_ink
        }
    }
    SnipButton {
        lk: st.lk
        res: st.res
        s: st.s
        name: st.res.tr("snip.more-" + st.what)
        icon: st.lk.icons.snip.edit.more
        why: st.most
        uid: "snip.more." + st.what
        tip: st.tip
        onClicked: st.stepped(1)
    }
}
