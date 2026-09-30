// Draws the tray icon's picture in the theme (a badge in the badge colours with the keyboard glyph) and hands it to the
// Ui object. The window is see-through and click-through and never takes focus; it shows only while it is grabbed.
import QtQuick

Window {
    id: pic

    // The app's Ui object, the look, and the icons folder and file type.
    required property var ui
    required property var lk
    required property var res

    readonly property var sh: lk.shape.tray
    // The picture's side in whole Qt units, so the grab has at least the tray's pixels; the Ui object scales it exactly.
    readonly property real side: Math.ceil(ui.trayPx() / Screen.devicePixelRatio)

    // A grab is running, and another was asked for meanwhile.
    property bool busy: false
    property bool again: false

    // Draws the picture again and hands it over, as when the look changes; a draw asked during a grab follows it.
    function draw() {
        if (busy) {
            again = true;
            return;
        }
        busy = true;
        visible = true;
        Qt.callLater(grab);
    }
    function grab() {
        const started = badge.grabToImage(result => {
            ui.trayShow(result.image);
            done();
        });
        if (!started)
            done();
    }
    function done() {
        busy = false;
        if (again) {
            again = false;
            draw();
        } else {
            visible = false;
        }
    }

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    color: "transparent"
    opacity: 0
    visible: false
    width: side
    height: side

    Rectangle {
        id: badge
        anchors.fill: parent
        radius: pic.side * pic.sh.radius_share
        color: pic.lk.palette.badge_bg
        Icon {
            anchors.centerIn: parent
            icon: pic.lk.icons.tray.glyph
            base: pic.res.icons
            ext: pic.res.ext
            px: pic.side * pic.sh.glyph_share
            tint: pic.lk.palette.badge_ink
        }
    }
}
