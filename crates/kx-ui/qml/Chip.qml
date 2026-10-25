// One suggestion chip (mock-up .chip): an optional icon and a word, Arabic words in the Arabic font. In forget mode it
// wears the forget mark in the danger keys' colours; bare, it shows its icon alone, and its name is its tip.
import QtQuick

Item {
    id: c

    // The chip's text and icon (`{text, icon, ar}`, with `name` and `uid` when readers and gates know it otherwise),
    // the look, fonts and icons, size, and the corner radius the top bar uses.
    required property var chip
    required property var lk
    required property var res
    required property real s
    required property real radius
    // Forget mode's look, the icon alone, and whether its name shows as a tip, as a bare chip's does.
    property bool forget: false
    property bool bare: false
    property bool tipped: bare

    signal clicked
    // The pointer came onto a tipped chip (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property var sh: lk.shape.chip
    readonly property bool ar: !!chip.ar
    readonly property string name: chip.name || chip.text
    readonly property string icon: forget ? lk.icons.words.forget : chip.icon || ""
    // Its colour, its text's and its icon's: legend_2 missed AA on Dolch light's chip (3.83:1), so the icon takes the
    // plate's text colour; a chip is a button, so high contrast draws it in ButtonText, as its word. kx-theme checks
    // each pair, the danger ones too.
    readonly property color fill: forget ? lk.palette.danger : lk.palette.chip_bg
    readonly property color ink: forget ? lk.palette.danger_ink : lk.palette.legend
    readonly property color mark: forget ? ink : lk.contrast ? lk.palette.legend : lk.palette.legend_2_plate
    // Its width with its text, which the lane fits before it hides any text, and with its icon alone.
    readonly property real full: word.implicitWidth + (icon ? glyph.width + row.spacing : 0) + 2 * sh.pad_px * s
    readonly property real slim: (icon ? glyph.width : 0) + 2 * sh.pad_px * s

    width: row.width + 2 * sh.pad_px * s
    height: sh.height_px * s
    anchors.verticalCenter: parent ? parent.verticalCenter : undefined
    Accessible.role: Accessible.Button
    Accessible.name: c.name
    Accessible.id: chip.uid || ""
    Accessible.onPressAction: c.clicked()

    Rectangle {
        anchors.fill: parent
        radius: c.radius
        color: c.fill
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
            id: glyph
            visible: !!c.icon
            anchors.verticalCenter: parent.verticalCenter
            icon: c.icon
            base: c.res.icons
            ext: c.res.ext
            px: c.sh.icon_px * c.s
            tint: c.mark
        }
        Label {
            id: word
            visible: !c.bare
            anchors.verticalCenter: parent.verticalCenter
            text: c.chip.text
            font.family: c.ar ? c.res.fam.arabic : c.res.fam.latin
            font.pixelSize: c.sh.font_px[c.ar ? 1 : 0] * c.s
            font.weight: c.sh.weight
            color: c.ink
        }
    }
    // It acts at let-go, wherever the pointer went (ADR-0024); a tipped chip names itself while the pointer is on it.
    Act {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onContainsMouseChanged: if (c.tipped) c.tip(containsMouse ? c.name : "", c)
        onActed: {
            if (c.tipped)
                c.tip("", c);
            c.clicked();
        }
    }
}
