// The dictation bar over its shadow, as App shows it: not owned by the keyboard's shadow, so it stays while the
// keyboard hides (edge 15), placed by the keyboard's box and again as the keyboard shows, hides, moves or resizes. The
// Voice panel opens above it while the view carries one.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: glow

    // The app, whose Voice object, keyboard window and tray menu the bar uses.
    required property var app

    // The bar's own window and the Voice panel with its shadow (null while closed), for App and the tests.
    readonly property alias bar: bar
    readonly property var panel: panels.item
    readonly property Window face: app.face
    // The last view that carried a panel, which the panel keeps while a view without one takes it away.
    property var held: null

    transientParent: null
    lk: face.lk
    box: Qt.rect(bar.x, bar.y, bar.width, bar.height)
    radius: bar.body.radius
    follow: bar.body

    VoiceBar {
        id: bar
        voice: glow.app.voice
        lk: glow.face.lk
        res: glow.face.res
        s: glow.face.s
        ar: !!glow.face.tongue && !!glow.face.tongue.rtl
        box: () => glow.face.spots.beside(NaN)[0]
        logical: (x, y) => glow.face.spots.logical(x, y)
        onShowing: on => glow.app.shadow(glow, on, bar)
        onTouched: glow.app.menu.shut()
        onViewChanged: if (view && view.panel) glow.held = view
    }
    // Made while the view carries a panel and freed once it does not.
    Loader {
        id: panels
        active: glow.face.ready && !!bar.view && !!bar.view.panel && glow.held !== null
        sourceComponent: VoicePanel {
            win: glow.face
            bar: glow.bar
            view: glow.held.panel
            round: glow.held.round
            onTouched: glow.app.menu.shut()
        }
    }

    Connections {
        target: glow.face
        function onAtChanged() {
            bar.follow();
        }
        function onVisibleChanged() {
            bar.follow();
        }
        function onDrawnChanged() {
            bar.follow();
        }
    }
}
