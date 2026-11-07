// What a Quick-fill page shows over its rows: its heading in the first line, then Back and its other first buttons at
// the start of the second and its last ones at the end, so Back keeps its place from page to page.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: top

    // The panel's window; the heading's icon and name (`{text, rtl, arabic}`), or a null name for no heading; the
    // buttons at each end, as QuickFillButtons takes them.
    required property var box
    property string icon: ""
    property var title: null
    property var starts: []
    property var ends: []

    // Each end's buttons, for what the page puts between them.
    readonly property alias lead: lead
    readonly property alias trail: trail

    width: box.width
    height: box.topH

    QuickFillHeading {
        visible: !!top.title
        x: top.box.side
        y: top.box.fs.top_px[0] * top.box.s
        height: top.box.lineH
        icon: top.icon
        title: top.title
        room: top.width - 2 * top.box.side
        lk: top.box.lk
        res: top.box.res
        s: top.box.s
    }
    QuickFillButtons {
        id: lead
        x: top.box.side
        y: top.box.bandY
        buttons: top.starts
        lk: top.box.lk
        res: top.box.res
        s: top.box.s
        onClicked: id => top.box.press(id)
    }
    QuickFillButtons {
        id: trail
        anchors.right: parent.right
        anchors.rightMargin: top.box.side
        y: top.box.bandY
        buttons: top.ends
        lk: top.box.lk
        res: top.box.res
        s: top.box.s
        onClicked: id => top.box.press(id)
    }
}
