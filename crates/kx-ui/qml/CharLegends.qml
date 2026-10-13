// A character key's legends (mock-up .lg, .lg2, .lg3): the main one rises in after a language change; the other case
// top left and AltGr bottom right change at once, each on the theme's Latin baseline, so a taller Arabic or mark font
// keeps the same margin.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: c

    // The key's face (KeyText): its state, look, language, fonts and size.
    required property Item host

    readonly property var ks: host.ks
    readonly property var fam: host.res.fam
    // Arabic legends, and marks on their dotted circle, which the face flags, take the theme's Arabic or mark font.
    readonly property bool mainAr: ks ? ks.main_ar : false
    readonly property real mainPx: host.look.legend_px * host.s * (mainAr ? host.look.arabic_scale : 1)
    // The layout of the app in front.
    readonly property string cur: host.lang.cur.short
    // The cap clips while the main legend rises (mock-up .cap overflow).
    readonly property bool clips: legends.running

    onCurChanged: legends.play()

    Legend {
        x: (c.host.face[3] + c.host.lg.corner_px[2]) * c.host.s
        anchors.baseline: parent.top
        anchors.baselineOffset: (c.host.face[0] + c.host.lg.corner_px[0]) * c.host.s + c.host.look.font_metrics[0] * font.pixelSize
        fam: c.fam
        ar: c.ks ? c.ks.second_ar : false
        mark: c.ks ? c.ks.second_mark : false
        text: c.ks ? c.ks.second : ""
        font.pixelSize: c.host.look.legend_px * c.host.lg.corner_scale[0] * c.host.s
        font.weight: c.host.lg.corner_weight
        color: c.host.ink2
    }
    Legend {
        anchors.right: parent.right
        anchors.rightMargin: (c.host.face[1] + c.host.lg.corner_px[2]) * c.host.s
        anchors.baseline: parent.bottom
        anchors.baselineOffset: -(c.host.face[2] + c.host.lg.corner_px[1]) * c.host.s - c.host.look.font_metrics[1] * font.pixelSize
        fam: c.fam
        ar: c.ks ? c.ks.third_ar : false
        mark: c.ks ? c.ks.third_mark : false
        text: c.ks ? c.ks.third : ""
        font.pixelSize: c.host.look.legend_px * c.host.lg.corner_scale[1] * c.host.s
        color: c.host.ink2
    }
    Rise {
        id: legends
        anchors.fill: parent
        move: c.host.mv.legends
        from: c.host.amt.rise * c.mainPx

        Legend {
            anchors.centerIn: parent
            anchors.verticalCenterOffset: -c.host.lift
            fam: c.fam
            ar: c.mainAr
            mark: c.ks ? c.ks.main_mark : false
            text: c.ks ? c.ks.main : ""
            font.pixelSize: c.mainPx
            font.weight: c.host.look.legend_weight
            color: c.host.ink
        }
    }
}
