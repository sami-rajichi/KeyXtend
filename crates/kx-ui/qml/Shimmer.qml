// The shimmer while the words are on their way (design/voice-bar.html .shim): a light band in pop_line slides right to
// left over pop_hover, once every `[moves] shimmer`, its tile `[caption] shimmer_span` bars wide. Reduced motion holds
// it still, as the mock-up's still view does.
import QtQuick

Rectangle {
    id: shim

    // The look and size.
    required property var lk
    required property real s

    // The tile, in bar widths, and the band's peak along the bar: 1 at the right end, sliding one tile to the left.
    readonly property real span: lk.shape.caption.shimmer_span
    property real peak: 1
    readonly property bool running: slide.running
    readonly property color low: lk.palette.pop_hover
    readonly property color high: lk.palette.pop_line
    // The peak and the trough after it, inside the bar, in order: with the ends they are every turn of the colour.
    readonly property real turnA: Math.min(edge(peak), edge(peak + span / 2))
    readonly property real turnB: Math.max(edge(peak), edge(peak + span / 2))

    radius: lk.shape.caption.shimmer_px[1] * s
    gradient: Gradient {
        orientation: Gradient.Horizontal
        GradientStop {
            position: 0
            color: shim.at(0)
        }
        GradientStop {
            position: shim.turnA
            color: shim.at(shim.turnA)
        }
        GradientStop {
            position: shim.turnB
            color: shim.at(shim.turnB)
        }
        GradientStop {
            position: 1
            color: shim.at(1)
        }
    }

    // `u` kept inside the bar.
    function edge(u) {
        return Math.min(1, Math.max(0, u));
    }
    // The colour at `u` along the bar: pop_line at a peak, easing to pop_hover half a tile away, as the tile repeats.
    function at(u) {
        const x = ((u - peak) % span + span) % span;
        const k = Math.min(1, Math.min(x, span - x) * 2 / span);
        return Qt.rgba(high.r + (low.r - high.r) * k, high.g + (low.g - high.g) * k, high.b + (low.b - high.b) * k, high.a + (low.a - high.a) * k);
    }

    NumberAnimation on peak {
        id: slide
        running: shim.visible && shim.lk.motion.moves.shimmer.ms > 0
        loops: Animation.Infinite
        from: 1
        to: 1 - shim.span
        duration: shim.lk.motion.moves.shimmer.ms
        onStopped: shim.peak = 1
    }
}
