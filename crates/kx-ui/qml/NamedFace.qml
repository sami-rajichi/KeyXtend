// A named key's face: an icon, a label, or both side by side; their names show as tooltips instead of text. Anchors
// centre an icon on whole pixels, so it draws crisp; a label centres exactly, as that rounding can push a fitted label
// past its room.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: n

    // The key's face (KeyText): its look, icons, fonts, size and legend colour.
    required property Item t

    // The icon file, else the translated label.
    readonly property string icon: t.icon
    readonly property string label: t.label
    // The label's room: the cap less its padding, or a sculpted top plus what may spill onto the skirt.
    readonly property real labelRoom: t.sculpted
        ? width - (t.look.cap.top_inset_px[1] + t.look.cap.top_inset_px[3] - 2 * t.look.cap.label_spill_px) * t.s
        : width - 2 * t.lg.label_pad_px * t.s
    // The face never clips.
    readonly property bool clips: false

    Row {
        id: named
        // How far right of the key's middle the row centres: on a sculpted cap, its top's middle.
        readonly property real shift: n.t.sculpted ? (n.t.look.cap.top_inset_px[3] - n.t.look.cap.top_inset_px[1]) * n.t.s / 2 : 0
        anchors.horizontalCenter: n.label ? undefined : parent.horizontalCenter
        anchors.horizontalCenterOffset: shift
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: -n.t.lift
        spacing: n.t.lg.gap_px * n.t.s
        // A label's exact x; an icon's anchor holds x instead, so nothing is restored when a label goes.
        Binding on x {
            when: !!n.label
            value: (n.width - named.width) / 2 + named.shift
            restoreMode: Binding.RestoreNone
        }
        Icon {
            id: glyph
            visible: !!n.icon
            anchors.verticalCenter: parent.verticalCenter
            icon: n.icon
            base: n.t.res.icons
            ext: n.t.res.ext
            px: n.t.lg.icon_px[n.t.kv.side ? 1 : 0] * n.t.s
            tint: n.t.ink
        }
        // The label shrinks to fit its room, as real keycaps print long words smaller.
        Legend {
            visible: !!n.label
            anchors.verticalCenter: parent.verticalCenter
            width: n.labelRoom - (glyph.visible ? glyph.width + named.spacing : 0)
            horizontalAlignment: Text.AlignHCenter
            fontSizeMode: Text.HorizontalFit
            minimumPixelSize: n.t.lg.label_min_px
            fam: n.t.res.fam
            text: n.label
            font.pixelSize: n.t.look.mod_legend_px * n.t.s
            color: n.t.ink
        }
    }
}
