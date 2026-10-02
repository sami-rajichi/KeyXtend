// One suggestion chip (mock-up .chip): an optional icon and a word, Arabic words in the Arabic font.
import QtQuick

Item {
    id: c

    // The chip's text and icon, the look, fonts and icons, size, and the corner radius the top bar uses.
    required property var chip
    required property var lk
    required property var res
    required property real s
    required property real radius

    signal clicked

    readonly property var sh: lk.shape.chip
    readonly property bool ar: !!chip.ar

    width: row.width + 2 * sh.pad_px * s
    height: sh.height_px * s
    anchors.verticalCenter: parent ? parent.verticalCenter : undefined
    Accessible.role: Accessible.Button
    Accessible.name: chip.text
    Accessible.onPressAction: c.clicked()

    Rectangle {
        anchors.fill: parent
        radius: c.radius
        color: c.lk.palette.chip_bg
    }
    Hover {
        lk: c.lk
        radius: c.radius
        visible: area.containsMouse
    }
    Row {
        id: row
        anchors.centerIn: parent
        spacing: c.sh.gap_px * c.s
        Icon {
            visible: !!c.chip.icon
            anchors.verticalCenter: parent.verticalCenter
            icon: c.chip.icon || ""
            base: c.res.icons
            ext: c.res.ext
            px: c.sh.icon_px * c.s
            // legend_2 missed AA on Dolch light's chip (3.83:1), so it takes the plate's text colour; a chip is a button, so
            // high contrast draws it in ButtonText, as its word. kx-theme checks this pair.
            tint: c.lk.contrast ? c.lk.palette.legend : c.lk.palette.legend_2_plate
        }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            text: c.chip.text
            font.family: c.ar ? c.res.fam.arabic : c.res.fam.latin
            font.pixelSize: c.sh.font_px[c.ar ? 1 : 0] * c.s
            font.weight: c.sh.weight
            color: c.lk.palette.legend
        }
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: c.clicked()
    }
}
