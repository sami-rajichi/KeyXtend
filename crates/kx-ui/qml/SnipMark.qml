// One drawing on the editor's picture (snip-editor.js drawings()), in the picture's own pixels: a pen's or the
// Highlighter's path, a box, circle, arrow, line or cover from one click to the next, typed text from its top left (top
// right when it reads right to left), or a numbered disc centred where clicked. Text being typed shows its outline and,
// while the keyboard's letters come to Snip, its caret.
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

Item {
    id: mark

    // The picture it lies on, the drawing (`{mark, ink, size}`), and the caret in UTF-16 units of text being typed, or
    // -1 for a drawing done.
    required property var canvas
    required property var drawn
    property int caret: -1

    readonly property var n: canvas.n
    readonly property var sh: canvas.sh
    readonly property var m: drawn.mark
    readonly property color ink: canvas.lk.common[n.ink + drawn.ink]
    // Its thickness step and text size, the list's last past its end.
    readonly property var thick: canvas.step(canvas.sent ? canvas.sent.thick : [], drawn.size)
    readonly property var size: canvas.step(canvas.sent ? canvas.sent.text : [], drawn.size)
    readonly property bool shape: m.kind === n.marks.shape
    readonly property bool marker: m.kind === n.marks.highlight
    readonly property bool circle: shape && m.shape === n.shapes.circle
    readonly property bool cover: shape && m.shape === n.shapes.cover
    readonly property real px: !thick ? 0 : marker ? thick.marker : thick.pen
    // The box from one click to the next.
    readonly property rect span: shape ? Qt.rect(Math.min(m.from.x, m.to.x), Math.min(m.from.y, m.to.y), Math.abs(m.to.x - m.from.x), Math.abs(m.to.y - m.from.y)) : Qt.rect(0, 0, 0, 0)
    // The typed text, which a click on it maps to a caret, and the lines' stroke, the circle and disc, for the tests.
    readonly property alias words: words
    readonly property alias stroke: stroke
    readonly property alias arc: arc
    readonly property alias disc: disc

    width: canvas.whole.w
    height: canvas.whole.h

    // The lines it draws, each a list of points: a path, a box's sides, a line, or an arrow's shaft and head.
    function lines() {
        const pt = p => Qt.point(p.x, p.y);
        if (m.kind === n.marks.pen || marker)
            return [m.points.map(pt)];
        if (!shape)
            return [];
        const [a, b] = [m.from, m.to];
        if (m.shape === n.shapes.box)
            return [[pt(a), Qt.point(b.x, a.y), pt(b), Qt.point(a.x, b.y), pt(a)]];
        if (m.shape === n.shapes.line)
            return [[pt(a), pt(b)]];
        return m.shape === n.shapes.arrow ? [[pt(a), pt(b)], head(a, b)] : [];
    }
    // The arrow's head at `b`: two strokes `head` long, each opening `head_deg` from the shaft.
    function head(a, b) {
        const back = Math.atan2(a.y - b.y, a.x - b.x);
        const open = sh.head_deg * Math.PI / 180;
        const side = s => Qt.point(b.x + thick.head * Math.cos(back + s * open), b.y + thick.head * Math.sin(back + s * open));
        return [side(-1), Qt.point(b.x, b.y), side(1)];
    }

    Shape {
        id: outline
        anchors.fill: parent
        visible: !mark.cover && mark.px > 0
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            id: stroke
            strokeColor: mark.marker ? Qt.rgba(mark.ink.r, mark.ink.g, mark.ink.b, mark.sh.marker_share) : mark.ink
            strokeWidth: mark.px
            fillColor: "transparent"
            capStyle: mark.marker ? ShapePath.SquareCap : ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathMultiline {
                paths: mark.lines()
            }
        }
        // A circle inscribed in the box from one click to the next.
        ShapePath {
            strokeColor: mark.circle ? mark.ink : "transparent"
            strokeWidth: mark.px
            fillColor: "transparent"
            PathAngleArc {
                id: arc
                centerX: mark.span.x + mark.span.width / 2
                centerY: mark.span.y + mark.span.height / 2
                radiusX: mark.span.width / 2
                radiusY: mark.span.height / 2
                sweepAngle: 360
            }
        }
    }
    Rectangle {
        visible: mark.cover
        x: mark.span.x
        y: mark.span.y
        width: mark.span.width
        height: mark.span.height
        color: mark.ink
    }
    TextEdit {
        id: words
        visible: mark.m.kind === mark.n.marks.text
        x: !visible ? 0 : mark.m.rtl ? mark.m.at.x - width : mark.m.at.x
        y: visible ? mark.m.at.y : 0
        text: visible ? mark.m.text : ""
        readOnly: true
        enabled: false
        activeFocusOnPress: false
        selectByMouse: false
        textFormat: Text.PlainText
        renderType: TextEdit.CurveRendering
        horizontalAlignment: mark.m.rtl ? TextEdit.AlignRight : TextEdit.AlignLeft
        font.family: mark.m.rtl ? mark.canvas.res.fam.arabic : mark.canvas.res.fam.latin
        font.pixelSize: mark.size ? mark.size.height : 1
        font.weight: mark.sh.mark_weight
        color: mark.ink

        // The text being typed: its outline, dashed as the mock-up's in window pixels, and its caret while the letters
        // come to Snip; neither goes into a flattened picture.
        Shape {
            id: typed
            objectName: "typing"
            readonly property alias path: dashes
            // The gap to the text in the picture's pixels, so it stays the same in window pixels at any scale.
            readonly property real gap: mark.canvas.k > 0 ? mark.sh.typing_gap_px / mark.canvas.k : mark.sh.typing_gap_px
            visible: mark.caret >= 0 && !mark.canvas.flattening
            anchors.fill: parent
            anchors.margins: -gap
            ShapePath {
                id: dashes
                strokeColor: mark.ink
                strokeWidth: mark.canvas.line
                fillColor: "transparent"
                strokeStyle: ShapePath.DashLine
                dashPattern: mark.sh.dash_px.map(d => d / mark.canvas.lk.shape.line.px)
                PathPolyline {
                    path: [Qt.point(0, 0), Qt.point(typed.width, 0), Qt.point(typed.width, typed.height), Qt.point(0, typed.height), Qt.point(0, 0)]
                }
            }
        }
        Rectangle {
            readonly property rect at: words.positionToRectangle(Math.max(0, mark.caret))
            objectName: "caret"
            visible: mark.caret >= 0 && mark.canvas.snips.aimed && !mark.canvas.flattening
            x: at.x
            y: at.y
            width: mark.canvas.line
            height: at.height
            color: mark.ink
        }
    }
    Rectangle {
        id: disc
        readonly property real d: mark.thick ? mark.thick.disc : 0
        visible: mark.m.kind === mark.n.marks.step
        x: visible ? mark.m.at.x - d / 2 : 0
        y: visible ? mark.m.at.y - d / 2 : 0
        width: d
        height: d
        radius: d / 2
        color: mark.ink
        Label {
            objectName: "number"
            anchors.centerIn: parent
            text: parent.visible ? String(mark.m.n) : ""
            font.family: mark.canvas.res.fam.latin
            font.pixelSize: Math.max(1, parent.d * mark.sh.step_share)
            font.weight: mark.sh.mark_weight
            color: mark.canvas.lk.common[mark.n.number + mark.drawn.ink]
        }
    }
}
