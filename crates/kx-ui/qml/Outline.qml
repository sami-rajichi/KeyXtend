// The scroll pad's outline around its target (spec §5.2, mock-up .scroll-target): four thin click-through edge windows
// inside the target's box, cut to the screen holding most of it, made only while the hub names a box, guarded and laid
// just under the keyboard. One window over the whole target would cost megabytes of buffers (ADR-0029).
pragma ComponentBehavior: Bound
import QtQuick

QtObject {
    id: outline

    // The hub's Ui object, and the keyboard window: its look, texts, spots, title, whether it shows, and its guard.
    required property QtObject ui
    required property var face

    // The scroll state as last read (`Ui.scrollJson`), null before the first read.
    property var st: null
    // The box to outline in Qt units, null for none.
    readonly property var box: st && st.area && face.lk && face.spots ? cut(st.area) : null
    // The ring colour, or Highlight in high contrast, as a key under the pointer is outlined there.
    readonly property color colour: face.lk ? (face.lk.contrast ? face.lk.hover_line : face.lk.palette.ring) : "transparent"
    readonly property real px: face.lk ? face.lk.shape.outline.px : 0
    // The edge windows' one title, which the guard and `layUnder` find them by.
    readonly property string title: face.res ? face.res.tr("window.outline") : ""
    // The edge windows, by side, made only while there is a box.
    property Instantiator edges: Instantiator {
        active: outline.box !== null
        model: ["top", "bottom", "left", "right"]
        delegate: Edge {
            required property string modelData
            lk: outline.face.lk
            title: outline.title
            colour: outline.colour
            at: outline.edge(modelData)
        }
        // The first of a set shows; the guard and the keyboard take them once all four are made.
        onObjectAdded: Qt.callLater(outline.lay)
    }
    // A wake reads the state on the event loop's next pass, which lets the hub's next change wake again.
    property Connections woke: Connections {
        target: outline.ui
        function onScrollChanged() {
            Qt.callLater(outline.read);
        }
    }
    // A keyboard shown again shows at its old place, which may be under the edges.
    property Connections shows: Connections {
        target: outline.face
        enabled: outline.edges.count > 0
        function onVisibleChanged() {
            if (outline.face.visible)
                Qt.callLater(outline.lay);
        }
    }

    Component.onCompleted: Qt.callLater(outline.read)

    function read() {
        st = JSON.parse(ui.scrollJson());
    }
    // The edges join the guard's quiet run, as they draw nothing over our windows, and lie just under the keyboard.
    function lay() {
        if (edges.count === 0)
            return;
        face.guardQuiet();
        ui.layUnder(title, face.title);
    }
    // Side `side` of the outline, inside the box, in Qt units.
    function edge(side) {
        const b = box || Qt.rect(0, 0, 0, 0);
        const tall = Math.max(0, b.height - 2 * px);
        const sides = {
            top: Qt.rect(b.x, b.y, b.width, px),
            bottom: Qt.rect(b.x, b.y + b.height - px, b.width, px),
            left: Qt.rect(b.x, b.y + px, px, tall),
            right: Qt.rect(b.x + b.width - px, b.y + px, px, tall)
        };
        return sides[side];
    }
    // Physical box `r` cut to the screen holding most of it, in Qt units at that screen's scale; null off every screen.
    function cut(r) {
        const parts = Array.from(face.spots.screens).map(sc => part(r, sc)).filter(p => p.w > 0 && p.h > 0);
        if (parts.length === 0)
            return null;
        const p = parts.reduce((a, b) => b.w * b.h > a.w * a.h ? b : a);
        const [sc, d] = [p.sc, p.sc.devicePixelRatio];
        return Qt.rect(sc.virtualX + (p.x - sc.virtualX) / d, sc.virtualY + (p.y - sc.virtualY) / d, p.w / d, p.h / d);
    }
    // The part of physical box `r` on screen `sc`, in physical pixels; Qt keeps a screen's corner unscaled.
    function part(r, sc) {
        const d = sc.devicePixelRatio;
        const [x, y] = [Math.max(r.x, sc.virtualX), Math.max(r.y, sc.virtualY)];
        const right = Math.min(r.x + r.w, sc.virtualX + sc.width * d);
        const bottom = Math.min(r.y + r.h, sc.virtualY + sc.height * d);
        return { sc: sc, x: x, y: y, w: right - x, h: bottom - y };
    }
}
