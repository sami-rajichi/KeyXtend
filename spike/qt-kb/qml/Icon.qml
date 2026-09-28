// One Lucide icon, tinted; readers skip it, since the key or button around it carries the name.
import QtQuick
import QtQuick.Controls.impl

IconImage {
    // Icon name, the icons folder URL and the file extension.
    required property string icon
    required property string base
    required property string ext
    // Size in Qt units, and colour.
    required property real px
    required property color tint

    width: px
    height: px
    sourceSize: Qt.size(px, px)
    source: icon.length > 0 ? base + "/" + icon + "." + ext : ""
    color: tint
    Accessible.ignored: true
}
