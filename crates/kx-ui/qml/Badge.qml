// The pointer badge (mock-up .badge, updateBadge): the active mode's icon, the side key's, with a word while Grab holds
// a press down, and while the scroll pad scrolls its arrow and "slow" or "fast", after a faded dot beside a mode. It
// fades in and out, readers hear what it shows, and in Arabic only the words read right to left.
import QtQuick

Item {
    id: badge

    // The look, fonts, icons and texts, the mode the hub shows by its side key's id (empty for none), Grab's state
    // ("off", "armed" or "carrying"), the pad's scroll part ({dir, fast}, or null), and whether it may show.
    required property var lk
    required property var res
    required property string mode
    required property string grab
    required property var scroll
    required property bool shown

    readonly property var sh: lk.shape.badge
    readonly property bool carrying: grab === "carrying"
    readonly property bool moded: mode !== ""
    readonly property bool scrolling: !!scroll && !!scroll.dir
    readonly property Label speed: scrolling && scroll.fast ? fast : slow
    // Each part's width as it shows: the mode's icon and word, the dot and its gaps, the arrow and its word.
    readonly property real modeW: moded ? sh.icon_px + (carrying ? sh.gap_px + holding.implicitWidth : 0) : 0
    readonly property real sepW: moded && scrolling ? sep.width + 2 * sh.gap_px : 0
    readonly property real scrollW: scrolling ? sh.icon_px + sh.gap_px + speed.implicitWidth : 0
    // Its width with every part and the wider speed word, the widest it gets, which the overlay's window keeps room for.
    readonly property real words: holding.implicitWidth + Math.max(slow.implicitWidth, fast.implicitWidth)
    readonly property real widest: 2 * sh.pad_px[1] + 2 * sh.icon_px + 4 * sh.gap_px + sep.width + words

    width: 2 * sh.pad_px[1] + (moded || scrolling ? modeW + sepW + scrollW : sh.icon_px)
    height: sh.icon_px + 2 * sh.pad_px[0]
    opacity: shown && (moded || scrolling) ? 1 : 0

    // Readers hear the mode, else the arrow, then the rest after the language's comma.
    Accessible.role: Accessible.StaticText
    Accessible.name: moded ? res.tr("label." + mode) : scrolling ? res.dpad(scroll.dir) : ""
    Accessible.description: {
        const rest = [carrying ? holding.text : "", moded && scrolling ? res.dpad(scroll.dir) : "", scrolling ? speed.text : ""];
        return rest.filter(t => t !== "").join(res.tr("badge.join"));
    }

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
        visible: badge.moded
        icon: badge.moded ? badge.lk.icons.keys[badge.mode] : ""
        base: badge.res.icons
        ext: badge.res.ext
        px: badge.sh.icon_px
        tint: badge.lk.palette.badge_ink
    }
    // Keyboard texts, so they take the font of the keyboard's language; the mock-up's mono font has no Arabic.
    Label {
        id: holding
        x: badge.sh.pad_px[1] + badge.sh.icon_px + badge.sh.gap_px
        visible: badge.moded && badge.carrying
        text: badge.res.tr("badge.holding")
        anchors.verticalCenter: parent.verticalCenter
        font.family: badge.res.fam.ui
        font.pixelSize: badge.sh.font_px
        font.weight: badge.sh.weight
        color: badge.lk.palette.badge_ink
    }
    // The dot between the mode and the scroll part (mock-up's · at opacity .4).
    Rectangle {
        id: sep
        x: badge.sh.pad_px[1] + badge.modeW + badge.sh.gap_px
        anchors.verticalCenter: parent.verticalCenter
        visible: badge.moded && badge.scrolling
        width: badge.sh.sep_px
        height: width
        radius: width / 2
        color: badge.lk.palette.badge_ink
        opacity: badge.sh.sep_share
    }
    Icon {
        id: arrow
        x: badge.sh.pad_px[1] + badge.modeW + badge.sepW
        anchors.verticalCenter: parent.verticalCenter
        visible: badge.scrolling
        icon: badge.scrolling ? badge.lk.icons.dpad[badge.scroll.dir] : ""
        base: badge.res.icons
        ext: badge.res.ext
        px: badge.sh.icon_px
        tint: badge.lk.palette.badge_ink
    }
    Label {
        id: slow
        x: arrow.x + badge.sh.icon_px + badge.sh.gap_px
        visible: badge.scrolling && badge.speed === slow
        text: badge.res.tr("badge.slow")
        anchors.verticalCenter: parent.verticalCenter
        font.family: badge.res.fam.ui
        font.pixelSize: badge.sh.font_px
        font.weight: badge.sh.weight
        color: badge.lk.palette.badge_ink
    }
    Label {
        id: fast
        x: slow.x
        visible: badge.scrolling && badge.speed === fast
        text: badge.res.tr("badge.fast")
        anchors.verticalCenter: parent.verticalCenter
        font.family: badge.res.fam.ui
        font.pixelSize: badge.sh.font_px
        font.weight: badge.sh.weight
        color: badge.lk.palette.badge_ink
    }
}
