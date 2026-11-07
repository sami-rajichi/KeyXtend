// A panel button (mock-up .btn), as the clipboard and Quick-fill draw it: an icon and a label, the icon alone or the
// label alone, on a ghost, danger, Enter-coloured, shown or flat fill. One with nothing to act on shows dimmed, never
// hidden, and it acts at let-go (ADR-0024); a switch reads as on or off.
import QtQuick

Item {
    id: b

    // Its label (empty for the icon alone), icon (empty for the label alone), look (`ghost`, `danger`, `enter`, `shown`
    // or `flat`), whether it can act, the look, fonts, icons and size, its gate id, what readers hear about it, and
    // whether a switch is on (null for a plain button).
    required property string label
    required property string icon
    required property string tone
    required property bool can
    required property var lk
    required property var res
    required property real s
    property string uid: ""
    property string about: ""
    property var checked: null

    signal clicked

    readonly property var sh: lk.shape.clip.button_px
    readonly property var p: lk.palette
    readonly property bool hot: can && area.containsMouse
    readonly property bool flat: tone === "flat"
    // Each look's fill and ink; kx-theme checks each pair, under the pointer too (drawn/clip.rs, fill.rs, plate.rs).
    readonly property var looks: ({
            danger: [p.danger, p.danger_ink],
            enter: [p.enter, p.enter_ink],
            ghost: [p.pop_hover, lk.sel_ink],
            shown: [p.pop_sel, lk.sel_ink],
            flat: ["transparent", hot ? lk.sel_ink : p.pop_ink]
        })
    readonly property color fill: looks[tone][0]
    readonly property color ink: looks[tone][1]

    width: label ? row.width + 2 * sh[2] * s : height
    height: sh[0] * s
    enabled: can
    opacity: can ? 1 : lk.shape.panel.off_share
    Accessible.role: checked === null ? Accessible.Button : Accessible.CheckBox
    Accessible.name: label
    Accessible.id: uid
    Accessible.description: about
    Accessible.checkable: checked !== null
    Accessible.checked: checked === true
    Accessible.onPressAction: if (b.can) b.clicked()
    Accessible.onToggleAction: if (b.can && b.checked !== null) b.clicked()

    // Outlined as the page buttons are, so a fill near the panel's colour still shows its edge.
    Rectangle {
        anchors.fill: parent
        radius: b.sh[1] * b.s
        color: b.fill
        border.width: b.flat ? 0 : b.lk.shape.line.px
        border.color: b.lk.palette.pop_line
    }
    // A flat button takes pop_hover under the pointer; the others take the theme's hover tint over their fill.
    Hover {
        lk: b.lk
        radius: b.sh[1] * b.s
        visible: b.hot
        tint: b.flat ? b.p.pop_hover : b.p.hover
    }
    Row {
        id: row
        anchors.centerIn: parent
        spacing: b.sh[5] * b.s
        Icon {
            visible: !!b.icon
            anchors.verticalCenter: parent.verticalCenter
            icon: b.icon
            base: b.res.icons
            ext: b.res.ext
            px: b.sh[4] * b.s
            tint: b.ink
        }
        Label {
            visible: !!b.label
            anchors.verticalCenter: parent.verticalCenter
            text: b.label
            font.family: b.res.fam.ui
            font.pixelSize: b.sh[3] * b.s
            font.weight: b.lk.shape.clip.button_weight
            color: b.ink
        }
    }
    Act {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onActed: if (b.can) b.clicked()
    }
}
