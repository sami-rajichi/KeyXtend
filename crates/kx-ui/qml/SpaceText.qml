// The space bar's text (mock-up .sp): the language and the layout, without bidi marks, so the first letter sets the
// reading order; it rises in after a language change.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: sp

    // The key's face (KeyText): its look, language, fonts and size.
    required property Item host

    readonly property real px: host.lg.space_px[host.lang.cur.rtl ? 1 : 0] * host.s
    // The layout of the app in front.
    readonly property string cur: host.lang.cur.short
    // The cap clips while the text rises (mock-up .cap overflow).
    readonly property bool clips: words.running

    onCurChanged: words.play()

    Rise {
        id: words
        anchors.fill: parent
        move: sp.host.mv.legends
        from: sp.host.amt.rise * sp.px

        Legend {
            anchors.fill: parent
            anchors.leftMargin: sp.host.lg.space_pad_px * sp.host.s
            anchors.rightMargin: sp.host.lg.space_pad_px * sp.host.s
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
            fam: sp.host.res.fam
            ar: sp.host.lang.cur.rtl
            text: sp.host.res.space(sp.host.lang.space)
            font.pixelSize: sp.px
            color: sp.host.ink2
        }
    }
}
