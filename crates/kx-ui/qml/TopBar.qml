// The top bar: move grip, suggestion chips, light or dark, the theme, the size buttons, minimise and close (mock-up
// topBar plus the owner's sizes and theme button). It never mirrors: in Arabic only its texts change (ADR-0020).
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: bar

    // Its box, the look, fonts, icons and texts, size, the preset shown (null between presets), how many presets there are,
    // the size buttons' texts in the keyboard's language, and the mode.
    required property var box
    required property var lk
    required property var res
    required property real s
    required property var preset
    required property int presets
    required property var sizes
    required property bool dark
    // Suggestion chips, `{text, icon, ar}`; none until word suggestions come.
    property var chips: []

    // One signal per button; chips and sizes carry their index.
    signal grip
    signal chip(int i)
    signal flip
    signal theme
    signal smaller
    signal bigger
    signal pickSize(int i)
    signal minimise
    signal close
    // The pointer came onto a button (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property var sh: lk.shape.bar
    readonly property var p: lk.palette
    readonly property real radius: lk.look.key_radius_px * sh.radius_share
    // The light or dark switch shows what a click gives.
    readonly property string mode: dark ? "mode_on" : "mode"

    x: box.x
    y: box.y
    width: box.w
    height: box.h

    // One top-bar button: `part` names its icon (`icons.toml [bar]`) and its name (`bar.*`); a size shows a letter.
    component Button: Item {
        id: b
        property string part: ""
        property string text: ""
        property string name: bar.res.bar(part)
        // The gates find each button by this id, whatever the language.
        property string uid: part ? bar.res.uid("bar", part) : ""
        property bool lit: false
        readonly property bool hot: area.containsMouse
        // Hover text sits on pop_hover, which is Highlight in high contrast, so it takes sel_ink there; the lit size
        // letter is text in pop_ink, and its mark carries the ring colour.
        readonly property color ink: hot ? bar.lk.sel_ink : lit ? bar.p.pop_ink : bar.p.legend_2_plate
        signal clicked
        width: bar.sh.button_px[0] * bar.s
        height: bar.sh.button_px[1] * bar.s
        Accessible.role: Accessible.Button
        Accessible.name: b.name
        Accessible.id: b.uid
        Accessible.onPressAction: b.clicked()
        Hover {
            lk: bar.lk
            radius: bar.radius
            visible: b.hot
            tint: bar.p.pop_hover
        }
        Icon {
            visible: !b.text
            anchors.centerIn: parent
            icon: b.text ? "" : bar.lk.icons.bar[b.part]
            base: bar.res.icons
            ext: bar.res.ext
            px: bar.sh.icon_px * bar.s
            tint: b.ink
        }
        Label {
            visible: !!b.text
            anchors.centerIn: parent
            text: b.text
            font.family: bar.res.fam.ui
            font.pixelSize: bar.sh.label_px * bar.s
            font.weight: b.lit ? bar.sh.lit_weight : bar.sh.weight
            color: b.ink
        }
        // The lit size's mark: a pill under its letter, in sel_ink on a hovered button in high contrast.
        Rectangle {
            readonly property bool mark: true
            visible: b.lit
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom
            anchors.bottomMargin: bar.sh.mark_px[2] * bar.s
            width: bar.sh.mark_px[0] * bar.s
            height: bar.sh.mark_px[1] * bar.s
            radius: height / 2
            color: b.hot && bar.lk.contrast ? bar.lk.sel_ink : bar.p.ring
        }
        // It acts at let-go, wherever the pointer went (ADR-0024).
        Act {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onContainsMouseChanged: bar.tip(containsMouse ? b.name : "", b)
            onActed: {
                bar.tip("", b);
                b.clicked();
            }
        }
    }

    Row {
        id: leftRow
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        spacing: bar.sh.gap_px * bar.s
        Button {
            part: "grip"
            onClicked: bar.grip()
        }
    }

    Row {
        anchors.left: leftRow.right
        anchors.right: rightRow.left
        anchors.margins: bar.sh.gap_px * bar.s
        anchors.verticalCenter: parent.verticalCenter
        clip: true
        spacing: bar.lk.shape.chip.gap_px * bar.s
        Repeater {
            model: bar.chips
            delegate: Chip {
                required property var modelData
                required property int index
                chip: modelData
                radius: bar.radius
                lk: bar.lk
                res: bar.res
                s: bar.s
                onClicked: bar.chip(index)
            }
        }
    }

    Row {
        id: rightRow
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        spacing: bar.sh.gap_px * bar.s
        Button {
            part: bar.mode
            onClicked: bar.flip()
        }
        Button {
            part: "theme"
            onClicked: bar.theme()
        }
        Button {
            part: "smaller"
            onClicked: bar.smaller()
        }
        // One button per preset; the keyboard's settings allow no more presets than the sizes named here.
        Repeater {
            model: bar.presets
            delegate: Button {
                required property int index
                readonly property var says: bar.sizes[index]
                text: says.label
                name: says.name
                uid: bar.res.uid("size", says.id)
                lit: bar.preset === index
                onClicked: bar.pickSize(index)
            }
        }
        Button {
            part: "bigger"
            onClicked: bar.bigger()
        }
        Button {
            part: "minimise"
            onClicked: bar.minimise()
        }
        Button {
            part: "close"
            onClicked: bar.close()
        }
    }
}
