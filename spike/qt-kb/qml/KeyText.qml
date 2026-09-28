// What a key shows: character legends, an icon or a label, the space bar text or the language carousel.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: t

    required property var kv
    required property var ks
    required property var lk
    required property var lang
    required property var res
    required property real s
    required property color ink

    readonly property var look: lk.look
    readonly property var lg: lk.shape.legend
    readonly property color ink2: lk.palette.legend_2
    // Dolch lifts every legend a little (mock-up line 217).
    readonly property real lift: look.cap.kind === "skirt" ? look.cap.legend_lift_px * s : 0
    readonly property bool isChar: kv.kind === "char"
    readonly property bool mainAr: isChar && ks ? ks.main_ar : false
    readonly property bool secondAr: isChar && ks ? ks.second_ar : false

    // A single-line text in a theme font.
    component Line: Text {
        property bool ar: false
        font.family: ar ? t.res.fam.arabic : t.res.fam.latin
        textFormat: Text.PlainText
        Accessible.ignored: true
    }

    // Character keys: the main legend, the other case top left, and AltGr bottom right.
    Line {
        visible: t.isChar
        anchors.centerIn: parent
        anchors.verticalCenterOffset: -t.lift
        ar: t.mainAr
        text: t.ks ? t.ks.main : ""
        font.pixelSize: t.look.legend_px * t.s * (t.mainAr ? t.look.arabic_scale : 1)
        font.weight: t.look.legend_weight
        color: t.ink
    }
    Line {
        visible: t.isChar
        x: t.lg.corner_px[1] * t.s
        y: t.lg.corner_px[0] * t.s - t.lift
        ar: t.secondAr
        text: t.ks ? t.ks.second : ""
        font.pixelSize: t.look.legend_px * t.lg.corner_scale[0] * t.s
        font.weight: t.lg.corner_weight
        color: t.ink2
    }
    Line {
        visible: t.isChar
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.rightMargin: t.lg.corner_px[1] * t.s
        anchors.bottomMargin: t.lg.corner_px[0] * t.s + t.lift
        text: t.ks ? t.ks.third : ""
        font.pixelSize: t.look.legend_px * t.lg.corner_scale[1] * t.s
        color: t.ink2
    }

    // Named keys: an icon, a label, or both side by side; their names show as tooltips instead of text.
    Row {
        visible: !t.isChar && t.kv.kind !== "space" && t.kv.kind !== "lang"
        anchors.centerIn: parent
        anchors.verticalCenterOffset: -t.lift
        spacing: t.lg.gap_px * t.s
        Icon {
            visible: !!t.kv.icon
            anchors.verticalCenter: parent.verticalCenter
            icon: t.kv.icon || ""
            base: t.res.icons
            ext: t.res.ext
            px: t.lg.icon_px[t.kv.side ? 1 : 0] * t.s
            tint: t.ink
        }
        Line {
            visible: !!t.kv.label
            anchors.verticalCenter: parent.verticalCenter
            text: t.kv.label || ""
            font.pixelSize: t.look.mod_legend_px * t.s
            color: t.ink
        }
    }

    // The space bar: language and layout.
    Line {
        visible: t.kv.kind === "space"
        anchors.fill: parent
        anchors.leftMargin: t.lg.space_pad_px * t.s
        anchors.rightMargin: t.lg.space_pad_px * t.s
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
        ar: t.lang.rtl
        text: t.lang.space || ""
        font.pixelSize: t.lg.space_px[t.lang.rtl ? 1 : 0] * t.s
        color: t.ink2
    }

    // The language key: previous, current and next layout, the current one large.
    Row {
        id: langRow
        visible: t.kv.kind === "lang"
        readonly property var sh: t.lk.shape.lang
        anchors.centerIn: parent
        anchors.verticalCenterOffset: -t.lift
        spacing: sh.gap_px * t.s
        Repeater {
            model: [t.lang.prev, t.lang.cur, t.lang.next]
            delegate: Line {
                required property string modelData
                required property int index
                readonly property bool cur: index === 1
                readonly property var sh: langRow.sh
                anchors.verticalCenter: parent ? parent.verticalCenter : undefined
                ar: t.lang.ar ? t.lang.ar[index] : false
                text: modelData
                font.pixelSize: (cur ? sh.font_px[ar ? 1 : 0] : sh.side[0]) * t.s
                font.weight: cur ? sh.weight : Font.Normal
                opacity: cur ? 1 : sh.side[1]
                color: t.ink
            }
        }
    }
}
