// One icon (Lucide or our own drawing), tinted through Qt Quick Effects' public colourisation: full brightness turns
// its strokes white, then full colourisation paints them in the tint. Readers skip it, as its control has the name.
import QtQuick
import QtQuick.Effects

Item {
    id: glyph

    // Icon name, the icons folder URL and the file extension.
    required property string icon
    required property string base
    required property string ext
    // Size in Qt units, and colour.
    required property real px
    required property color tint

    width: px
    height: px
    Accessible.ignored: true

    Image {
        id: art
        anchors.fill: parent
        visible: false
        sourceSize: Qt.size(glyph.px, glyph.px)
        source: glyph.icon.length > 0 ? glyph.base + "/" + glyph.icon + "." + glyph.ext : ""
    }
    MultiEffect {
        anchors.fill: art
        source: art
        opacity: glyph.tint.a
        brightness: 1
        colorization: 1
        colorizationColor: glyph.tint
    }
}
