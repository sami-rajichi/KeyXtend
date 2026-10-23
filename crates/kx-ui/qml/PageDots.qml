// One dot per page under the Next key's chevron, or per layout on the language key's Cycle design (mock-up .dots): the
// current one lit, opaque and larger, as an unlit one keeps 3:1 (shape.toml [dots]). No numbers; past `room` they wrap.
pragma ComponentBehavior: Bound
import QtQuick

Grid {
    id: dots

    // How many dots and the lit one from 0, the look, size and ink, and the length a line of dots may take.
    required property int count
    required property int at
    required property var lk
    required property real s
    required property color ink
    required property real room

    readonly property var sh: lk.shape.dots
    readonly property real dot: sh.px * s
    readonly property real dotOn: sh.on_px * s
    // Dots per line, with room for the larger lit dot, so many wrap into more lines inside the cap.
    readonly property int perLine: Math.max(1, Math.floor((room - (dotOn - dot) + spacing) / (dot + spacing)))
    readonly property int line: Math.min(perLine, Math.max(1, count))
    readonly property int lines: Math.ceil(Math.max(1, count) / line)

    spacing: sh.gap_px * s
    // Lines run along `flow`: rows of dots by default, columns on the Cycle design.
    rows: flow === Grid.TopToBottom ? line : lines
    columns: flow === Grid.TopToBottom ? lines : line

    // Cells of one size, so the larger lit dot stays in line with the others.
    Repeater {
        model: dots.count
        delegate: Item {
            id: cell
            required property int index
            width: dots.dot
            height: dots.dot
            Rectangle {
                readonly property bool lit: cell.index === dots.at
                // Exactly centred, so the larger lit dot stays in line.
                anchors.centerIn: parent
                anchors.alignWhenCentered: false
                width: lit ? dots.dotOn : dots.dot
                height: width
                radius: width / 2
                color: dots.ink
                opacity: lit ? 1 : dots.sh.off
            }
        }
    }
}
