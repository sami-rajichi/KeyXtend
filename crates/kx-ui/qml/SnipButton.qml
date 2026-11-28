// A button of Snip's editor, question and panel (mock-up .eb, .eb.wide): the panels' button, with ClipButton's sizes
// and tones, its icon with or without its word, named for readers and in its tip. A dimmed one shows faded, says why
// in its tip and description and never acts; it acts at let-go (ADR-0024).
import QtQuick

Item {
    id: b

    // Its name, icon (empty for its word alone), tone (`flat`, `ghost`, `shown` or `enter`, as ClipButton's), whether its
    // word shows, why it waits (empty while it acts), whether it is the one picked of its row (null for a plain button),
    // the look, fonts, icons and size, its gate id, and what shows a tip over it.
    required property string name
    required property string icon
    property string tone: "flat"
    property bool worded: false
    property string why: ""
    property var checked: null
    required property var lk
    required property var res
    property real s: 1
    property string uid: ""
    property var tip: (text, item) => {}

    // A click while it acts.
    signal clicked

    readonly property var sh: lk.shape.clip.button_px
    readonly property var p: lk.palette
    readonly property bool dim: why.length > 0
    readonly property bool hot: !dim && area.containsMouse
    // What its tip says: the reason it waits, or the name of one that shows its icon alone.
    readonly property string says: dim ? res.noteTip(name, why) : worded ? "" : name
    // Each tone's fill and ink; kx-theme checks each pair, under the pointer too (drawn/clip.rs).
    readonly property var looks: ({
            enter: [p.enter, p.enter_ink],
            ghost: [p.pop_hover, lk.sel_ink],
            shown: [p.pop_sel, lk.sel_ink],
            flat: ["transparent", hot ? lk.sel_ink : p.pop_ink]
        })
    readonly property color fill: looks[tone][0]
    readonly property color ink: looks[tone][1]

    width: worded ? row.width + 2 * sh[2] * s : height
    height: sh[0] * s
    opacity: dim ? lk.shape.panel.off_share : 1
    Accessible.role: checked === null ? Accessible.Button : Accessible.RadioButton
    Accessible.name: name
    Accessible.description: why
    Accessible.id: uid
    Accessible.checkable: checked !== null
    Accessible.checked: checked === true
    Accessible.onPressAction: b.act()
    Accessible.onToggleAction: b.act()

    function act() {
        if (!dim)
            clicked();
    }

    Rectangle {
        anchors.fill: parent
        radius: b.sh[1] * b.s
        color: b.fill
        border.width: b.tone === "flat" ? 0 : b.lk.shape.line.px
        border.color: b.p.pop_line
    }
    // A flat button takes pop_hover under the pointer; the others take the theme's hover tint over their fill.
    Hover {
        lk: b.lk
        radius: b.sh[1] * b.s
        visible: b.hot
        tint: b.tone === "flat" ? b.p.pop_hover : b.p.hover
    }
    Row {
        id: row
        anchors.centerIn: parent
        spacing: b.sh[5] * b.s
        // Left to right in every language: Arabic flips the text only (ADR-0020).
        layoutDirection: Qt.LeftToRight
        Icon {
            visible: !!b.icon
            anchors.verticalCenter: parent.verticalCenter
            icon: b.icon
            base: b.res.icons
            ext: b.res.ext
            px: b.sh[4] * b.s
            tint: b.ink
        }
        Label {
            visible: b.worded
            anchors.verticalCenter: parent.verticalCenter
            text: b.name
            font.family: b.res.fam.ui
            font.pixelSize: b.sh[3] * b.s
            font.weight: b.lk.shape.clip.button_weight
            color: b.ink
        }
    }
    Act {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onActed: b.act()
        onContainsMouseChanged: b.tip(containsMouse ? b.says : "", b)
    }
}
