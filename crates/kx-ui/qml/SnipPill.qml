// One button of the record bar (design/snip-parts.css .rb): flat on the pill, red while its sound records, red with its
// word for Stop. A dimmed one never acts and says why in its tip; it acts at the press, as keys do.
import QtQuick

Item {
    id: pb

    // The look and texts; the key's id, word and icon; whether its word shows, whether it is a switch and on, why it is
    // dimmed (empty while it acts), and whether it is Stop.
    required property var lk
    required property var res
    required property string key
    required property string word
    required property string icon
    property bool worded: false
    property bool toggle: false
    property bool on: false
    property string why: ""
    property bool stop: false
    // Shows `text` as a tip over `item`, or hides it with an empty text; tests leave it quiet.
    property var tip: (text, item) => {}

    // A press on it while it acts.
    signal hit

    readonly property var sh: lk.shape.snip
    readonly property var p: lk.palette
    readonly property bool dim: why.length > 0
    readonly property bool lit: toggle && on && !dim
    readonly property bool hot: area.containsMouse && !dim
    // Stop's white on red, else under the pointer sel_ink on pop_hover, as a panel's flat button, then a recording
    // sound's red (Highlight in high contrast) or the muted ink; drawn/snip.rs checks each.
    readonly property color ink: stop ? p.rec_ink : hot ? lk.sel_ink : lit ? (lk.contrast ? p.ring : lk.common.rec_dot) : p.pop_muted
    readonly property string says: dim ? res.noteTip(word, why) : worded ? "" : word

    width: Math.ceil(Math.max(sh.rec_key_px, row.width + 2 * sh.rec_key_pad_px))
    height: sh.rec_key_px

    Accessible.role: Accessible.Button
    Accessible.name: pb.word
    Accessible.description: pb.why
    Accessible.id: res.uid("snip.rec", pb.key)
    Accessible.checkable: pb.toggle
    Accessible.checked: pb.on
    Accessible.onPressAction: pb.act()
    Accessible.onToggleAction: pb.act()

    function act() {
        if (!dim)
            hit();
    }

    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: pb.stop ? pb.p.rec : pb.hot ? pb.p.pop_hover : "transparent"
        Row {
            id: row
            objectName: "row"
            anchors.centerIn: parent
            // A dimmed one fades as the panels' dimmed buttons do, so it never looks like a switch that is merely off.
            opacity: pb.dim ? pb.lk.shape.panel.off_share : 1
            spacing: pb.sh.icon_gap_px
            layoutDirection: Qt.LeftToRight
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                icon: pb.icon
                base: pb.res.icons
                ext: pb.res.ext
                px: pb.sh.icon_px
                tint: pb.ink
            }
            Label {
                visible: pb.worded
                anchors.verticalCenter: parent.verticalCenter
                text: pb.word
                font.family: pb.res.fam.ui
                font.pixelSize: pb.sh.word_px
                font.weight: pb.lk.look.legend_weight
                color: pb.ink
            }
        }
    }
    Act {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        atPress: true
        onActed: pb.act()
        onContainsMouseChanged: pb.tip(containsMouse ? pb.says : "", pb)
    }
}
