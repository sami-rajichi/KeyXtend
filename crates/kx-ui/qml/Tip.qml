// A tooltip with a button's name, and for a dimmed one when it comes, after a short hover; it never takes focus and
// lets clicks through.
import QtQuick

Window {
    id: tip

    // The window whose buttons it names owns it: Windows keeps an owned window above its owner, also after the guard
    // lifts the owner, whereas a hidden window shows again where it was (ADR-0013).
    required transientParent
    // The look, fonts and the hover time.
    required property var lk
    required property var res
    required property int ms
    // Lifts our shown window titled as given alone in front of our others: the guard may lift a banner or the list after
    // the owner, and a hidden tip shows again below them.
    required property var lift

    // The button under the pointer, and its name.
    property Item at: null
    property string name: ""

    readonly property var sh: lk.shape.tip
    // Its owner's screen in Qt units, for a button on no screen; the bubble may sit on another screen than the keyboard.
    readonly property rect area: {
        const sc = transientParent.screen;
        return Qt.rect(sc.virtualX, sc.virtualY, sc.width, sc.height);
    }

    flags: Qt.ToolTip | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    title: res.tr("window.tip")
    color: "transparent"
    visible: false
    width: label.implicitWidth + 2 * sh.pad_px[1]
    height: label.implicitHeight + 2 * sh.pad_px[0]

    // New texts or a new look close it, so it never shows a word in the last language (ADR-0020).
    onResChanged: hint("", null)

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

    // Centres it above the button, or below when its screen's top is too close, and keeps it on that screen.
    function place() {
        const p = spot(at.mapToGlobal(at.width / 2, 0), at.mapToGlobal(at.width / 2, at.height), Qt.application.screens);
        x = p.x;
        y = p.y;
    }
    // Where it goes, in Qt units, for a button whose top and bottom centres are `top` and `bottom`: on the one of
    // `screens` under the button, so a keyboard across two screens keeps each tip by its button.
    function spot(top, bottom, screens) {
        const a = boxOf(top, screens);
        const above = top.y - sh.gap_px - height;
        return {
            x: Math.max(a.x, Math.min(top.x - width / 2, a.x + a.width - width)),
            y: above >= a.y ? above : bottom.y + sh.gap_px
        };
    }
    // The box of the one of `screens` holding Qt point `p`, or its owner's screen when none does.
    function boxOf(p, screens) {
        const holds = sc => {
            const inX = p.x >= sc.virtualX && p.x < sc.virtualX + sc.width;
            return inX && p.y >= sc.virtualY && p.y < sc.virtualY + sc.height;
        };
        const sc = Array.from(screens).find(holds);
        return sc ? Qt.rect(sc.virtualX, sc.virtualY, sc.width, sc.height) : area;
    }

    Timer {
        id: wait
        interval: tip.ms
        onTriggered: {
            if (!tip.at)
                return;
            tip.place();
            tip.visible = true;
            // Only a shown window is found by its title.
            tip.lift(tip.title);
        }
    }
    Rectangle {
        anchors.fill: parent
        radius: tip.sh.radius_px
        color: tip.lk.palette.pop_bg
        border.width: tip.lk.shape.line.px
        border.color: tip.lk.palette.pop_line
    }
    // Tips are keyboard texts, so they take the font of the keyboard's language.
    Label {
        id: label
        anchors.centerIn: parent
        text: tip.name
        horizontalAlignment: Text.AlignHCenter
        font.family: tip.res.fam.ui
        font.pixelSize: tip.sh.font_px
        font.weight: tip.sh.weight
        color: tip.lk.palette.pop_ink
    }
}
