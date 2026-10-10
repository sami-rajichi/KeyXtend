// A character key's legends (mock-up .lg, .lg2, .lg3): the main one rises in after a language change; the other case
// top left and AltGr bottom right change at once, each on the theme's Latin baseline, so a taller Arabic or mark font
// keeps the same margin.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: c

    // The key's face (KeyText): its state, look, language, fonts and size.
    required property Item t

    readonly property var ks: t.ks
    readonly property var fam: t.res.fam
    // Arabic legends, and marks on their dotted circle, which the face flags, take the theme's Arabic or mark font.
    readonly property bool mainAr: ks ? ks.main_ar : false
    readonly property real mainPx: t.look.legend_px * t.s * (mainAr ? t.look.arabic_scale : 1)
    // The layout of the app in front.
    readonly property string cur: t.lang.cur.short
    // The cap clips while the main legend rises (mock-up .cap overflow).
    readonly property bool clips: legends.running

    onCurChanged: legends.play()

    Legend {
        x: (c.t.face[3] + c.t.lg.corner_px[2]) * c.t.s
        anchors.baseline: parent.top
        anchors.baselineOffset: (c.t.face[0] + c.t.lg.corner_px[0]) * c.t.s + c.t.look.font_metrics[0] * font.pixelSize
        fam: c.fam
        ar: c.ks ? c.ks.second_ar : false
        mark: c.ks ? c.ks.second_mark : false
        text: c.ks ? c.ks.second : ""
        font.pixelSize: c.t.look.legend_px * c.t.lg.corner_scale[0] * c.t.s
        font.weight: c.t.lg.corner_weight
        color: c.t.ink2
    }
    Legend {
        anchors.right: parent.right
        anchors.rightMargin: (c.t.face[1] + c.t.lg.corner_px[2]) * c.t.s
        anchors.baseline: parent.bottom
        anchors.baselineOffset: -(c.t.face[2] + c.t.lg.corner_px[1]) * c.t.s - c.t.look.font_metrics[1] * font.pixelSize
        fam: c.fam
        ar: c.ks ? c.ks.third_ar : false
        mark: c.ks ? c.ks.third_mark : false
        text: c.ks ? c.ks.third : ""
        font.pixelSize: c.t.look.legend_px * c.t.lg.corner_scale[1] * c.t.s
        color: c.t.ink2
    }
    Rise {
        id: legends
        anchors.fill: parent
        move: c.t.mv.legends
        from: c.t.amt.rise * c.mainPx

        Legend {
            anchors.centerIn: parent
            anchors.verticalCenterOffset: -c.t.lift
            fam: c.fam
            ar: c.mainAr
            mark: c.ks ? c.ks.main_mark : false
            text: c.ks ? c.ks.main : ""
            font.pixelSize: c.mainPx
            font.weight: c.t.look.legend_weight
            color: c.t.ink
        }
    }
}
