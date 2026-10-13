// One icon (Lucide or our own drawing), which the icon provider draws at its size with its ink in the tint, so it
// needs no effect; the tint's alpha is its opacity. Readers skip it, as its control has the name.
import QtQuick

Image {
    id: glyph

    // Icon name, the URL that serves the icons (`res.icons`) and their file type.
    required property string icon
    required property string base
    required property string ext
    // Size in Qt units, and colour.
    required property real px
    required property color tint
    // How much of the tint's opacity the icon shows, such as a chevron drawn lighter.
    property real share: 1

    width: px
    height: px
    sourceSize: Qt.size(px, px)
    opacity: tint.a * share
    // The provider names a tint by its six hex digits, as `#rrggbb` without the hash.
    source: icon.length > 0 ? base + "/" + Qt.rgba(tint.r, tint.g, tint.b, 1).toString().slice(1) + "/" + icon + "." + ext : ""
    // A colour change asks for a new image each frame; uncached, each one goes when the next comes.
    cache: false
    Accessible.ignored: true
}
