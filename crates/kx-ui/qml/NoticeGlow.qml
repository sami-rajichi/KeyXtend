// The notice banner over its shadow, as App shows it: above the keyboard, or at the bottom of the main screen while
// the keyboard is away, so not owned by the keyboard's shadow, and placed again as the keyboard shows, hides, moves or
// resizes.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: glow

    // The app, whose Ui object, keyboard window and tray menu the banner uses.
    required property var app

    // The banner's own window, for App and the tests.
    readonly property alias notice: notice
    readonly property Window face: app.face

    transientParent: null
    lk: face.lk
    res: face.res
    box: Qt.rect(notice.x, notice.y, notice.width, notice.height)
    radius: notice.radius
    follow: notice.body

    Notice {
        id: notice
        ui: glow.app.ui
        lk: glow.face.lk
        res: glow.face.res
        s: glow.face.s
        widest: glow.face.running ? glow.face.drawn.width : glow.face.lk.shape.panel.width_px
        spot: (w, h) => glow.face.spots.popSpot(w, h, NaN)
        onShowing: on => glow.app.shadow(glow, on, notice)
        onTouched: glow.app.menu.shut()
    }

    Connections {
        target: glow.face
        function onAtChanged() {
            notice.follow();
        }
        function onVisibleChanged() {
            notice.follow();
        }
        function onDrawnChanged() {
            notice.follow();
        }
    }
}
