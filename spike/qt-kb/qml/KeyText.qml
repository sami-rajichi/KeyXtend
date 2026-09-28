// What a key shows: character legends, an icon or a label, the space bar text or the language carousel.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: t

    // The key's box and look, its changing state, the look, the language key, fonts and icons, size and legend colour.
    required property var kv
    required property var ks
    required property var lk
    required property var lang
    required property var res
    required property real s
    required property color ink

    readonly property var look: lk.look
    readonly property var lg: lk.shape.legend
    readonly property var mv: lk.motion.moves
    readonly property var amt: lk.motion.amount
    readonly property color ink2: lk.palette.legend_2
    // Dolch lifts every legend a little (mock-up line 217).
    readonly property real lift: look.cap.kind === "skirt" ? look.cap.legend_lift_px * s : 0
    readonly property bool isChar: kv.kind === "char"
    readonly property bool isSpace: kv.kind === "space"
    readonly property bool isLang: kv.kind === "lang"
    readonly property bool mainAr: isChar && ks ? ks.main_ar : false
    readonly property bool secondAr: isChar && ks ? ks.second_ar : false
    readonly property real spacePx: lg.space_px[lang.rtl ? 1 : 0] * s
    readonly property real mainPx: look.legend_px * s * (mainAr ? look.arabic_scale : 1)
    // The layout of the app in front.
    readonly property string cur: lang.cur

    // The cap hides what slides past its edge (mock-up .cap overflow); other keys clip only while their legend rises.
    clip: isLang || swap.running

    // After a language change the main legend rises in; the language key's names slide in the way the key stepped.
    onCurChanged: {
        if (isLang && lang.turn !== 0) {
            turn.from = lang.turn * amt.carousel_shift * langRow.width;
            turn.restart();
        } else if (isChar || isSpace) {
            swap.restart();
        }
    }

    // A single-line text in a theme font.
    component Line: Text {
        property bool ar: false
        font.family: ar ? t.res.fam.arabic : t.res.fam.latin
        textFormat: Text.PlainText
        Accessible.ignored: true
    }

    ParallelAnimation {
        id: swap
        Tween {
            target: rise
            property: "y"
            from: t.amt.rise * (t.isSpace ? t.spacePx : t.mainPx)
            to: 0
            move: t.mv.legends
        }
        Tween {
            target: legends
            property: "opacity"
            from: 0
            to: 1
            move: t.mv.legends
        }
    }
    Tween {
        id: turn
        target: slide
        property: "x"
        to: 0
        move: t.mv.carousel
    }

    // Character keys: the other case top left, and AltGr bottom right; they change at once (mock-up .lg2, .lg3).
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

    // What rises in after a language change (mock-up .swap .lg, .sp): the main legend or the space bar text.
    Item {
        id: legends
        anchors.fill: parent
        transform: Translate {
            id: rise
        }

        Line {
            visible: t.isChar
            anchors.centerIn: parent
            anchors.verticalCenterOffset: -t.lift
            ar: t.mainAr
            text: t.ks ? t.ks.main : ""
            font.pixelSize: t.mainPx
            font.weight: t.look.legend_weight
            color: t.ink
        }
        // The space bar: language and layout.
        Line {
            visible: t.isSpace
            anchors.fill: parent
            anchors.leftMargin: t.lg.space_pad_px * t.s
            anchors.rightMargin: t.lg.space_pad_px * t.s
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
            ar: t.lang.rtl
            text: t.lang.space || ""
            font.pixelSize: t.spacePx
            color: t.ink2
        }
    }

    // Named keys: an icon, a label, or both side by side; their names show as tooltips instead of text.
    Row {
        visible: !t.isChar && !t.isSpace && !t.isLang
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

    // The language key: previous, current and next layout, the current one large.
    Row {
        id: langRow
        visible: t.isLang
        readonly property var sh: t.lk.shape.lang
        anchors.centerIn: parent
        anchors.verticalCenterOffset: -t.lift
        spacing: sh.gap_px * t.s
        transform: Translate {
            id: slide
        }
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
