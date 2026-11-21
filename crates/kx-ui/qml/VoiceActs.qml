// A note's acts under the dictation bar's line (design/voice-cloud.js .cna): at the bar's end as the row's buttons,
// pills with their words in their one order, Open service last.
import QtQuick

Row {
    id: acts

    // The bar, whose acts, size and padding it takes.
    required property var host

    x: host.width - host.pad - width
    y: host.row
    spacing: host.gap
    layoutDirection: Qt.LeftToRight
    Repeater {
        model: acts.host.acts
        delegate: VoiceBarButton {
            host: acts.host
            under: true
        }
    }
}
