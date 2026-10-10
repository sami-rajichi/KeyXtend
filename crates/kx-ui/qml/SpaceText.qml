// The space bar's text (mock-up .sp): the language and the layout, without bidi marks, so the first letter sets the
// reading order; it rises in after a language change.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: sp

    // The key's face (KeyText): its look, language, fonts and size.
    required property Item t

    readonly property real px: t.lg.space_px[t.lang.cur.rtl ? 1 : 0] * t.s
    // The layout of the app in front.
    readonly property string cur: t.lang.cur.short
    // The cap clips while the text rises (mock-up .cap overflow).
    readonly property bool clips: words.running

    onCurChanged: words.play()

    Rise {
        id: words
        anchors.fill: parent
        move: sp.t.mv.legends
        from: sp.t.amt.rise * sp.px

        Legend {
            anchors.fill: parent
            anchors.leftMargin: sp.t.lg.space_pad_px * sp.t.s
            anchors.rightMargin: sp.t.lg.space_pad_px * sp.t.s
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
            fam: sp.t.res.fam
            ar: sp.t.lang.cur.rtl
            text: sp.t.res.space(sp.t.lang.space)
            font.pixelSize: sp.px
            color: sp.t.ink2
        }
    }
}
