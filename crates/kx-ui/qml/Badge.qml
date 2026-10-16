// The mode badge beside the pointer (mock-up .badge): the active mode's icon, the side key's, on a pill, with a word
// while Grab holds a press down. It fades in and out, and readers hear the mode.
import QtQuick

Item {
    id: badge

    // The look, fonts, icons and texts, the mode the hub shows by its side key's id (empty for none), Grab's state
    // ("off", "armed" or "carrying"), and whether it may show.
    required property var lk
    required property var res
    required property string mode
    required property string grab
    required property bool shown

    readonly property var sh: lk.shape.badge
    readonly property bool carrying: grab === "carrying"
    // Its width with the word, the widest it gets, which the overlay's window keeps room for.
    readonly property real widest: 2 * sh.pad_px[1] + sh.icon_px + sh.gap_px + word.implicitWidth

    width: carrying ? widest : 2 * sh.pad_px[1] + sh.icon_px
    height: sh.icon_px + 2 * sh.pad_px[0]
    opacity: shown && mode !== "" ? 1 : 0

    Accessible.role: Accessible.StaticText
    Accessible.name: mode !== "" ? res.tr("label." + mode) : ""
    Accessible.description: carrying ? word.text : ""

    Behavior on opacity {
        Tween {
            move: badge.lk.motion.moves.badge
        }
    }

    // Its own small shadow (mock-up .badge box-shadow), which high contrast drops.
    Repeater {
        model: badge.lk.palette.badge_shadow.filter(l => !l.inset)
        delegate: Shade {
            required property var modelData
            shadow: modelData
            radius: badge.height / 2
        }
    }
    // A pill (mock-up border-radius 999px).
    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: badge.lk.palette.badge_bg
    }
    Icon {
        x: badge.sh.pad_px[1]
        anchors.verticalCenter: parent.verticalCenter
        icon: badge.mode !== "" ? badge.lk.icons.keys[badge.mode] : ""
        base: badge.res.icons
        ext: badge.res.ext
        px: badge.sh.icon_px
        tint: badge.lk.palette.badge_ink
    }
    // A keyboard text, so it takes the font of the keyboard's language; the mock-up's mono font has no Arabic.
    Label {
        id: word
        x: badge.sh.pad_px[1] + badge.sh.icon_px + badge.sh.gap_px
        anchors.verticalCenter: parent.verticalCenter
        visible: badge.carrying
        text: badge.res.tr("badge.holding")
        font.family: badge.res.fam.ui
        font.pixelSize: badge.sh.font_px
        font.weight: badge.sh.weight
        color: badge.lk.palette.badge_ink
    }
}
