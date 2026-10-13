// A pointer area that acts on the press it got (ADR-0024): at the press, as a key does, or at the let-go wherever it
// lands, so a slip of the hand never cancels it or acts on what it slipped onto. Qt's cancel acts on nothing.
import QtQuick

MouseArea {
    id: act

    // It acts at the press, as a key does; else at the let-go.
    property bool atPress: false
    // The second click of a double click acts at its let-go too; off, it acts on nothing.
    property bool doubles: true
    // Where the press landed, in this area's units.
    property point spot: Qt.point(0, 0)
    // This press is the second click of a double click that acts on nothing.
    property bool skip: false

    // The press acts, with `at` where it landed.
    signal acted(point at)

    onPressed: mouse => {
        spot = Qt.point(mouse.x, mouse.y);
        skip = false;
        if (atPress)
            acted(spot);
    }
    onDoubleClicked: skip = !doubles
    onReleased: {
        if (!atPress && !skip)
            acted(spot);
        skip = false;
    }
    onCanceled: skip = false
}
