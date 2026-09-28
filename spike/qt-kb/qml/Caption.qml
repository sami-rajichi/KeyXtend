// The caption bar (mock-up .capbar): what the mic hears, then the words it typed; it never takes focus and lets clicks pass.
// Its window reaches past the bar by the shadow's room, which is safe since clicks pass through.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: bar

    // The look and fonts, and the bar's largest size in Qt units.
    required property var lk
    required property var res
    required property real barWidth
    required property real barHeight
    // The caption shown: text, rec, busy, ar and hide_ms, as the voice worker sends it.
    property var cap: ({ text: "", rec: false, busy: false, ar: false, hide_ms: null })

    readonly property var sh: lk.shape.caption
    readonly property var mv: lk.motion.moves
    readonly property real m: lk.margin
    // A status such as "Listening…" stays up and is a short pill; words and notes fill the box.
    readonly property bool status: cap.hide_ms === null || cap.hide_ms === undefined
    // The words are on their way, so a shimmer runs (mock-up .shimmer).
    readonly property bool busy: cap.busy === true

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
        | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    visible: false
    color: "transparent"
    width: barWidth + 2 * m
    height: barHeight + 2 * m

    // It pops in each time it shows (mock-up .capbar).
    onVisibleChanged: if (visible) pop.restart()

    // Shows caption `r` with the bar's top-left at `at` (Qt units); hides it after `r.hide_ms` when set.
    function say(r, at) {
        bar.cap = r;
        bar.x = at.x - m;
        bar.y = at.y - m;
        bar.visible = true;
        if (r.hide_ms) {
            hide.interval = r.hide_ms;
            hide.restart();
        } else {
            hide.stop();
        }
    }

    Timer {
        id: hide
        onTriggered: bar.visible = false
    }
    PopIn {
        id: pop
        target: popped
        move: bar.mv.caption
        amount: bar.lk.motion.amount
    }
    Item {
        id: popped
        width: bar.width
        height: bar.height

        // The bar sits on the bottom of its box, like the mock-up's bar above the keyboard.
        Item {
            id: body
            readonly property real radius: bar.status ? height / 2 : bar.sh.words_radius_px
            x: bar.m
            width: bar.barWidth
            height: bar.status ? bar.sh.bar_px : bar.barHeight
            y: bar.m + bar.barHeight - height

            Repeater {
                model: bar.lk.palette.window_shadow.filter(l => !l.inset)
                delegate: Shade {
                    required property var modelData
                    anchors.fill: parent
                    shadow: modelData
                    radius: body.radius
                }
            }
            // High contrast has no shadow, so the bar gets an edge instead.
            Rectangle {
                anchors.fill: parent
                radius: body.radius
                color: bar.lk.palette.pop_bg
                border.width: bar.lk.contrast ? bar.lk.shape.line.px : 0
                border.color: bar.lk.palette.pop_line
            }
            Row {
                readonly property var pad: bar.status ? bar.sh.pad_px : Array(3).fill(bar.sh.words_pad_px)
                anchors.fill: parent
                anchors.topMargin: pad[0]
                anchors.bottomMargin: pad[0]
                anchors.leftMargin: pad[1]
                anchors.rightMargin: pad[2]
                spacing: bar.sh.gap_px

                Rectangle {
                    id: dot
                    visible: bar.cap.rec
                    anchors.verticalCenter: parent.verticalCenter
                    width: bar.sh.dot_px
                    height: width
                    radius: width / 2
                    color: bar.lk.common.rec_dot

                    // It pulses while the mic records (mock-up .rec-dot); loops never run at 0 ms.
                    SequentialAnimation on opacity {
                        running: bar.visible && bar.cap.rec && bar.mv.dot.ms > 0
                        loops: Animation.Infinite
                        onStopped: dot.opacity = 1

                        Tween {
                            move: bar.mv.dot
                            share: 0.5
                            to: bar.lk.motion.amount.pulse_low
                        }
                        Tween {
                            move: bar.mv.dot
                            share: 0.5
                            to: 1
                        }
                    }
                }
                Text {
                    id: words
                    width: bar.busy ? Math.min(implicitWidth, parent.width) : parent.width - (dot.visible ? dot.width + parent.spacing : 0)
                    height: parent.height
                    text: bar.cap.text
                    textFormat: Text.PlainText
                    font.family: bar.cap.ar ? bar.res.fam.arabic : bar.res.fam.latin
                    font.pixelSize: bar.status ? bar.sh.status_px : bar.sh.words_px[bar.cap.ar ? 1 : 0]
                    font.weight: bar.status ? bar.sh.status_weight : Font.Normal
                    color: bar.lk.palette.pop_ink
                    wrapMode: Text.Wrap
                    elide: Text.ElideRight
                    verticalAlignment: Text.AlignVCenter
                    Accessible.role: Accessible.StaticText
                    Accessible.name: text
                }
                Shimmer {
                    visible: bar.busy && bar.visible && words.implicitWidth + parent.spacing < parent.width
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.max(0, parent.width - words.width - parent.spacing)
                    height: bar.sh.shimmer_px[0]
                    radius: bar.sh.shimmer_px[1]
                    span: bar.sh.shimmer_span
                    base: bar.lk.palette.pop_hover
                    band: bar.lk.palette.pop_line
                    move: bar.mv.shimmer
                }
            }
        }
    }
}
