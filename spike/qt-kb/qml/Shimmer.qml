// The shimmer while the words are on their way (mock-up .shimmer): a light band slides along a rounded bar, over and over.
import QtQuick
import QtQuick.Shapes

Shape {
    id: bar

    // Bar and band colours, corner radius, the move for one pass, and the band's repeat in bar widths.
    required property color base
    required property color band
    required property real radius
    required property var move
    required property real span

    // How far through one pass the band is, 0 to 1; its geometry follows the width at every step.
    property real phase: 0
    readonly property real period: span * width

    // The default renderer leaves the round ends jagged without multisampling.
    preferredRendererType: Shape.CurveRenderer
    Accessible.ignored: true

    ShapePath {
        strokeWidth: -1
        fillGradient: LinearGradient {
            x1: bar.phase * bar.period
            x2: bar.phase * bar.period + bar.period
            spread: ShapeGradient.RepeatSpread
            GradientStop {
                position: 0
                color: bar.base
            }
            GradientStop {
                position: 0.5
                color: bar.band
            }
            GradientStop {
                position: 1
                color: bar.base
            }
        }
        PathRectangle {
            width: bar.width
            height: bar.height
            radius: bar.radius
        }
    }
    // Loops never run at 0 ms, so reduced motion leaves a still bar.
    Tween on phase {
        move: bar.move
        running: bar.visible && bar.move.ms > 0
        loops: Animation.Infinite
        from: 0
        to: 1
        onStopped: bar.phase = 0
    }
}
