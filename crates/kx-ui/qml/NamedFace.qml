// A named key's face, made of only what it shows: an icon, a label, both side by side, or the layer's Next chevron with
// its page dots; their names show as tooltips instead of text. Anchors centre an icon on whole pixels, so it draws
// crisp; a label centres exactly, as that rounding can push a fitted label past its room.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: n

    // The key's face (KeyText): its look, icons, fonts, size and legend colour.
    required property Item host

    // The icon file, else the translated label.
    readonly property string icon: host.icon
    readonly property string label: host.label
    // The label's room: the cap less its padding, or a sculpted top plus what may spill onto the skirt.
    readonly property real labelRoom: host.sculpted
        ? width - (host.look.cap.top_inset_px[1] + host.look.cap.top_inset_px[3] - 2 * host.look.cap.label_spill_px) * host.s
        : width - 2 * host.lg.label_pad_px * host.s
    // The icon's size, and its gap to a label beside it.
    readonly property real iconPx: host.lg.icon_px[host.kv.side ? 1 : 0] * host.s
    readonly property real gap: host.lg.gap_px * host.s
    // The face never clips.
    readonly property bool clips: false
    // The shortcuts layer's Next key, by the face's id for it, shows one dot per page under its chevron.
    readonly property bool paged: host.kv.id === "layer-next" && !!host.paging && !!icon

    // What the key shows, centred as one row.
    Loader {
        id: named
        // How far right of the key's middle the row centres: on a sculpted cap, its top's middle.
        readonly property real shift: n.host.sculpted ? (n.host.look.cap.top_inset_px[3] - n.host.look.cap.top_inset_px[1]) * n.host.s / 2 : 0
        anchors.horizontalCenter: n.label ? undefined : parent.horizontalCenter
        anchors.horizontalCenterOffset: shift
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: -n.host.lift
        sourceComponent: n.paged ? pagedC : n.label ? (n.icon ? bothC : labelC) : n.icon ? iconC : null
        // A label's exact x; an icon's anchor holds x instead, so nothing is restored when a label goes.
        Binding on x {
            when: !!n.label
            value: (n.width - named.width) / 2 + named.shift
            restoreMode: Binding.RestoreNone
        }
    }
    Component {
        id: iconC
        Icon {
            icon: n.icon
            base: n.host.res.icons
            ext: n.host.res.ext
            px: n.iconPx
            tint: n.host.ink
        }
    }
    // The label shrinks to fit its room, less an icon beside it, as real keycaps print long words smaller.
    Component {
        id: labelC
        Legend {
            width: n.labelRoom - (n.icon ? n.iconPx + n.gap : 0)
            horizontalAlignment: Text.AlignHCenter
            fontSizeMode: Text.HorizontalFit
            minimumPixelSize: n.host.lg.label_min_px
            fam: n.host.res.fam
            text: n.label
            font.pixelSize: n.host.look.mod_legend_px * n.host.s
            color: n.host.ink
        }
    }
    // The chevron with the page dots one gap under it, centred together as kx-theme's drawn check places them.
    Component {
        id: pagedC
        Column {
            spacing: n.host.lk.shape.dots.gap_px * n.host.s
            Loader {
                anchors.horizontalCenter: parent.horizontalCenter
                sourceComponent: iconC
            }
            PageDots {
                anchors.horizontalCenter: parent.horizontalCenter
                count: n.host.paging.pages
                at: n.host.paging.page
                lk: n.host.lk
                s: n.host.s
                ink: n.host.ink
                room: n.labelRoom
            }
        }
    }
    // Both side by side, which no key on today's boards shows.
    Component {
        id: bothC
        Row {
            spacing: n.gap
            Loader {
                anchors.verticalCenter: parent.verticalCenter
                sourceComponent: iconC
            }
            Loader {
                anchors.verticalCenter: parent.verticalCenter
                sourceComponent: labelC
            }
        }
    }
}
