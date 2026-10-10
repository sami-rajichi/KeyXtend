// The language key's face in its design: Carousel (previous, current and next layout, the current one large, which
// turn the way the key stepped), Cycle (LangCycle) or List (the current name and a chevron that opens the list).
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: f

    // The key's face (KeyText): its language view and design, look, icons, fonts, size and legend colour.
    required property Item t

    readonly property var sh: t.lk.shape.lang
    // The cap always hides the names that slide past its edge (mock-up .cap overflow).
    readonly property bool clips: true

    Loader {
        anchors.centerIn: parent
        anchors.verticalCenterOffset: -f.t.lift
        sourceComponent: f.t.design === "cycle" ? cycleC : f.t.design === "list" ? listC : carouselC
    }

    Component {
        id: cycleC
        LangCycle {
            lang: f.t.lang
            lk: f.t.lk
            res: f.t.res
            s: f.t.s
            ink: f.t.ink
            room: f.height
        }
    }
    // The List design (mock-up .lst).
    Component {
        id: listC
        Row {
            spacing: f.sh.icon_gap_px * f.t.s
            Legend {
                anchors.verticalCenter: parent.verticalCenter
                fam: f.t.res.fam
                ar: f.t.lang.cur.arabic
                text: f.t.lang.cur.short
                font.pixelSize: f.sh.font_px[ar ? 1 : 0] * f.t.s
                font.weight: f.sh.weight
                color: f.t.ink
            }
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                icon: f.t.lk.icons.lang.list
                base: f.t.res.icons
                ext: f.t.res.ext
                px: f.sh.list.icon_px * f.t.s
                tint: f.t.ink
                opacity: f.sh.list.icon_share
            }
        }
    }
    // The Carousel design (mock-up .car).
    Component {
        id: carouselC
        Row {
            id: names
            // The layout of the app in front.
            readonly property string cur: f.t.lang.cur.short
            spacing: f.sh.gap_px * f.t.s
            transform: Translate {
                id: slide
            }
            // The names slide in the way the key stepped.
            onCurChanged: {
                if (f.t.lang.turn !== 0) {
                    turn.from = f.t.lang.turn * f.t.amt.carousel_shift * names.width;
                    turn.restart();
                }
            }
            Repeater {
                model: [f.t.lang.prev || f.t.lang.cur, f.t.lang.cur, f.t.lang.next || f.t.lang.cur]
                delegate: Legend {
                    required property var modelData
                    required property int index
                    readonly property bool cur: index === 1
                    anchors.verticalCenter: parent ? parent.verticalCenter : undefined
                    fam: f.t.res.fam
                    ar: modelData.arabic
                    text: modelData.short
                    font.pixelSize: (cur ? f.sh.font_px[ar ? 1 : 0] : f.sh.side[0]) * f.t.s
                    font.weight: cur ? f.sh.weight : f.sh.side_weight
                    opacity: cur ? 1 : f.sh.side[1]
                    color: f.t.ink
                }
            }
            Tween {
                id: turn
                target: slide
                property: "x"
                to: 0
                move: f.t.mv.carousel
            }
        }
    }
}
