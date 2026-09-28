// A tooltip with a button's name after a short hover; it never takes focus and lets clicks through.
import QtQuick

Window {
    id: tip

    // The look, fonts, the hover time, and the keyboard's screen in Qt units.
    required property var lk
    required property var res
    required property int ms
    required property rect area

    // The button under the pointer, and its name.
    property Item at: null
    property string name: ""

    readonly property var sh: lk.shape.tip

    flags: Qt.ToolTip | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    color: "transparent"
    visible: false
    width: label.implicitWidth + 2 * sh.pad_px[1]
    height: label.implicitHeight + 2 * sh.pad_px[0]

    // Shows `text` over `item` after the hover time; an empty text hides it, unless it comes late from another button.
    function hint(text, item) {
        if (text.length === 0 && item && item !== at)
            return;
        wait.stop();
        visible = false;
        name = text;
        at = text.length > 0 ? item : null;
        if (at)
            wait.start();
    }

    // Centres it above the button, or below when the screen's top is too close, and keeps it on screen.
    function place() {
        const top = at.mapToGlobal(at.width / 2, 0);
        const bottom = at.mapToGlobal(at.width / 2, at.height);
        const above = top.y - sh.gap_px - height;
        x = Math.max(area.x, Math.min(top.x - width / 2, area.x + area.width - width));
        y = above >= area.y ? above : bottom.y + sh.gap_px;
    }

    Timer {
        id: wait
        interval: tip.ms
        onTriggered: {
            if (!tip.at)
                return;
            tip.place();
            tip.visible = true;
        }
    }
    Rectangle {
        anchors.fill: parent
        radius: tip.sh.radius_px
        color: tip.lk.palette.pop_bg
        border.width: tip.lk.shape.line.px
        border.color: tip.lk.palette.pop_line
    }
    Text {
        id: label
        anchors.centerIn: parent
        text: tip.name
        textFormat: Text.PlainText
        font.family: tip.res.fam.latin
        font.pixelSize: tip.sh.font_px
        font.weight: tip.sh.weight
        color: tip.lk.palette.pop_ink
    }
}
