// The top bar: move grip, suggestion chips, light or dark, the size buttons, minimise and close (mock-up topBar plus the owner's sizes).
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: bar

    // Its box, the look, fonts and icons, the bar settings, size, the preset shown, and whether it is dark.
    required property var box
    required property var lk
    required property var res
    required property var cfg
    required property real s
    required property var preset
    required property bool dark

    // One signal per button; chips and sizes carry their index.
    signal grip
    signal chip(int i)
    signal flip
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

    x: box.x
    y: box.y
    width: box.w
    height: box.h

    // One top-bar button: an icon or a short text; lit buttons use the ring colour.
    component Button: Item {
        id: b
        property string icon: ""
        property string text: ""
        property string name: ""
        property bool lit: false
        signal clicked
        width: bar.sh.button_px[0] * bar.s
        height: bar.sh.button_px[1] * bar.s
        Accessible.role: Accessible.Button
        Accessible.name: b.name
        Accessible.onPressAction: b.clicked()
        Rectangle {
            anchors.fill: parent
            radius: bar.radius
            color: area.containsMouse ? bar.p.pop_hover : "transparent"
        }
        Icon {
            visible: b.icon.length > 0
            anchors.centerIn: parent
            icon: b.icon
            base: bar.res.icons
            ext: bar.res.ext
            px: bar.sh.icon_px * bar.s
            tint: b.lit ? bar.p.ring : area.containsMouse ? bar.p.legend : bar.p.legend_2
        }
        Text {
            visible: b.text.length > 0
            anchors.centerIn: parent
            text: b.text
            font.family: bar.res.fam.latin
            font.pixelSize: bar.sh.label_px * bar.s
            font.weight: b.lit ? bar.sh.lit_weight : Font.Normal
            color: b.lit ? bar.p.ring : area.containsMouse ? bar.p.legend : bar.p.legend_2
            Accessible.ignored: true
        }
        MouseArea {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onContainsMouseChanged: bar.tip(containsMouse ? b.name : "", b)
            onClicked: {
                bar.tip("", b);
                b.clicked();
            }
        }
    }

    Row {
        id: leftRow
        anchors.verticalCenter: parent.verticalCenter
        spacing: bar.sh.gap_px * bar.s
        Button {
            icon: bar.cfg.grip.icon
            name: bar.cfg.grip.name
            onClicked: bar.grip()
        }
    }

    // Sample chips that type their text.
    Row {
        anchors.left: leftRow.right
        anchors.right: rightRow.left
        anchors.margins: bar.sh.gap_px * bar.s
        anchors.verticalCenter: parent.verticalCenter
        clip: true
        spacing: bar.lk.shape.chip.gap_px * bar.s
        Repeater {
            model: bar.cfg.chips
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
            icon: bar.dark ? bar.cfg.theme.icon_on : bar.cfg.theme.icon
            name: bar.cfg.theme.name
            onClicked: bar.flip()
        }
        Button {
            icon: bar.cfg.smaller.icon
            name: bar.cfg.smaller.name
            onClicked: bar.smaller()
        }
        Repeater {
            model: bar.cfg.sizes
            delegate: Button {
                required property string modelData
                required property int index
                text: modelData
                name: bar.cfg.size_names[index]
                lit: bar.preset === index
                onClicked: bar.pickSize(index)
            }
        }
        Button {
            icon: bar.cfg.bigger.icon
            name: bar.cfg.bigger.name
            onClicked: bar.bigger()
        }
        Button {
            icon: bar.cfg.minimise.icon
            name: bar.cfg.minimise.name
            onClicked: bar.minimise()
        }
        Button {
            icon: bar.cfg.close.icon
            name: bar.cfg.close.name
            onClicked: bar.close()
        }
    }
}
