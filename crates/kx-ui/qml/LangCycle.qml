// The language key's Cycle design (mock-up .cyc and .dots): a globe, the current layout's short name, which rises in
// when it changes, and one dot per layout with the current one lit and larger (PageDots). A click always steps on one.
pragma ComponentBehavior: Bound
import QtQuick

Row {
    id: cyc

    // The language key's view, the look, fonts and icons, the size, the legend colour, and the cap's height.
    required property var lang
    required property var lk
    required property var res
    required property real s
    required property color ink
    required property real room

    readonly property var sh: lk.shape.lang
    readonly property var cy: sh.cycle
    readonly property bool ar: lang.cur.arabic
    readonly property string cur: lang.cur.short

    spacing: sh.icon_gap_px * s
    onCurChanged: rise.restart()

    Icon {
        anchors.verticalCenter: parent.verticalCenter
        icon: cyc.lk.icons.lang.cycle
        base: cyc.res.icons
        ext: cyc.res.ext
        px: cyc.cy.icon_px * cyc.s
        tint: cyc.ink
        share: cyc.cy.icon_share
    }
    Label {
        id: name
        anchors.verticalCenter: parent.verticalCenter
        text: cyc.cur
        font.family: cyc.ar ? cyc.res.fam.arabic : cyc.res.fam.latin
        font.pixelSize: cyc.sh.font_px[cyc.ar ? 1 : 0] * cyc.s
        font.weight: cyc.sh.weight
        color: cyc.ink
        transform: Translate {
            id: lift
        }
    }
    // One dot per layout in a column, wrapping into more columns inside the cap.
    PageDots {
        anchors.verticalCenter: parent.verticalCenter
        flow: Grid.TopToBottom
        count: cyc.lang.count
        at: cyc.lang.at
        lk: cyc.lk
        s: cyc.s
        ink: cyc.ink
        room: cyc.room
    }

    // The new name rises in (mock-up langIn).
    ParallelAnimation {
        id: rise
        Tween {
            target: lift
            property: "y"
            from: cyc.lk.motion.amount.rise * name.height
            to: 0
            move: cyc.lk.motion.moves.carousel
        }
        Tween {
            target: name
            property: "opacity"
            from: 0
            to: 1
            move: cyc.lk.motion.moves.carousel
        }
    }
}
