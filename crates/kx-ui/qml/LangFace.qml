// The language key's face in its design: Carousel (previous, current and next layout, the current one large, which
// turn the way the key stepped), Cycle (LangCycle) or List (the current name and a chevron that opens the list).
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: f

    // The key's face (KeyText): its language view and design, look, icons, fonts, size and legend colour.
    required property Item host

    readonly property var sh: host.lk.shape.lang
    // The cap always hides the names that slide past its edge (mock-up .cap overflow).
    readonly property bool clips: true

    Loader {
        anchors.centerIn: parent
        anchors.verticalCenterOffset: -f.host.lift
        sourceComponent: f.host.design === "cycle" ? cycleC : f.host.design === "list" ? listC : carouselC
    }

    Component {
        id: cycleC
        LangCycle {
            lang: f.host.lang
            lk: f.host.lk
            res: f.host.res
            s: f.host.s
            ink: f.host.ink
            room: f.height
        }
    }
    // The List design (mock-up .lst).
    Component {
        id: listC
        Row {
            spacing: f.sh.icon_gap_px * f.host.s
            Legend {
                anchors.verticalCenter: parent.verticalCenter
                fam: f.host.res.fam
                ar: f.host.lang.cur.arabic
                text: f.host.lang.cur.short
                font.pixelSize: f.sh.font_px[ar ? 1 : 0] * f.host.s
                font.weight: f.sh.weight
                color: f.host.ink
            }
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                icon: f.host.lk.icons.lang.list
                base: f.host.res.icons
                ext: f.host.res.ext
                px: f.sh.list.icon_px * f.host.s
                tint: f.host.ink
                share: f.sh.list.icon_share
            }
        }
    }
    // The Carousel design (mock-up .car).
    Component {
        id: carouselC
        Row {
            id: names
            // The layout of the app in front.
            readonly property string cur: f.host.lang.cur.short
            spacing: f.sh.gap_px * f.host.s
            transform: Translate {
                id: slide
            }
            // The names slide in the way the key stepped.
            onCurChanged: {
                if (f.host.lang.turn !== 0) {
                    turn.from = f.host.lang.turn * f.host.amt.carousel_shift * names.width;
                    turn.restart();
                }
            }
            Repeater {
                model: [f.host.lang.prev || f.host.lang.cur, f.host.lang.cur, f.host.lang.next || f.host.lang.cur]
                delegate: Legend {
                    required property var modelData
                    required property int index
                    readonly property bool cur: index === 1
                    anchors.verticalCenter: parent ? parent.verticalCenter : undefined
                    fam: f.host.res.fam
                    ar: modelData.arabic
                    text: modelData.short
                    font.pixelSize: (cur ? f.sh.font_px[ar ? 1 : 0] : f.sh.side[0]) * f.host.s
                    font.weight: cur ? f.sh.weight : f.sh.side_weight
                    opacity: cur ? 1 : f.sh.side[1]
                    color: f.host.ink
                }
            }
            Tween {
                id: turn
                target: slide
                property: "x"
                to: 0
                move: f.host.mv.carousel
            }
        }
    }
}
