// One keycap on the selection strip (design 4, .d-keys .key): the keyboard's own cap, one key row tall and as wide as
// its legend, with the chip's icon and word, or the tick and done word once clicked. It acts at the press, as keys do.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: sk

    // The keycap as the hub writes it (`{id, word, done}`), whether it was clicked, whether the strip takes clicks,
    // which legend parts show, the look, fonts and icons, size, the key row's height and whether the words are Arabic.
    required property var chip
    required property bool acted
    required property bool live
    required property bool icons
    required property bool words
    required property var lk
    required property var res
    required property real s
    required property real row
    required property bool ar

    // A press on it while the strip takes clicks.
    signal hit

    readonly property var sh: lk.shape.chips
    readonly property var look: lk.look
    readonly property var p: lk.palette
    // The cap sits half a gap inside its cell, as a key's does.
    readonly property real inset: look.gap_px * s / 2
    readonly property real gap: icons && words ? sh.icon_gap_px * s : 0
    // What the legend takes: the icon, the gap and the word, as each shows.
    readonly property real need: (icons ? glyph.width : 0) + gap + (words ? word.implicitWidth : 0)
    readonly property bool down: area.pressed && live
    property real sink: down ? 1 : 0
    // Dolch's sculpted caps lift the legend to their lighter top's middle, as a key's.
    readonly property real lift: look.cap.kind === "skirt" ? look.cap.legend_lift_px * s : 0
    // A pressed cap's ink in high contrast, else the legend's.
    readonly property color ink: down && p.press_ink ? p.press_ink : p.legend

    width: Math.ceil(need + 2 * (sh.cap_pad_px * s + inset))
    height: row

    Accessible.role: Accessible.Button
    Accessible.name: chip.word
    Accessible.id: res.uid(names.group, chip.id)
    Accessible.onPressAction: if (sk.live) sk.hit()

    SelNames {
        id: names
    }

    // A press sinks the cap at once; only the rise back eases, as a key's.
    Behavior on sink {
        enabled: targetValue < sk.sink
        Tween {
            move: sk.lk.motion.moves.press
        }
    }

    KeyCap {
        x: sk.inset
        y: sk.inset
        width: sk.width - 2 * sk.inset
        height: sk.height - 2 * sk.inset
        lk: sk.lk
        s: sk.s
        topColour: sk.p.key
        skirt: sk.p.skirt ? sk.p.skirt.key : sk.p.key
        pressed: sk.down
        hovered: sk.live && area.containsMouse
        sink: sk.sink

        // Left to right in every language: Arabic flips the text only (ADR-0020).
        Item {
            anchors.centerIn: parent
            anchors.verticalCenterOffset: -sk.lift
            width: sk.need
            height: parent.height

            Icon {
                id: glyph
                visible: sk.icons
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                icon: (sk.acted ? sk.lk.icons.chips.tick : sk.lk.icons.chips[sk.chip.id]) || ""
                base: sk.res.icons
                ext: sk.res.ext
                px: sk.sh.icon_px * sk.s
                tint: sk.ink
            }
            Label {
                id: word
                visible: sk.words
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                text: sk.acted ? sk.chip.done : sk.chip.word
                font.family: sk.ar ? sk.res.fam.arabic : sk.res.fam.latin
                font.pixelSize: sk.sh.font_px * sk.s * (sk.ar ? sk.look.arabic_scale : 1)
                font.weight: sk.look.legend_weight
                color: sk.ink
            }
        }
    }

    // It acts at the press, as a key does; where the pointer goes after changes nothing (ADR-0024).
    Act {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        atPress: true
        enabled: sk.live
        onActed: sk.hit()
    }
}
