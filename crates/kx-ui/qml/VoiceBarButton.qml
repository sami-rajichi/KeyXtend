// One of the dictation bar's buttons (VoiceBar.qml): round when it leads or ends the row, or for the gear; a pill with
// its word otherwise, as each act `under` a note's line is.
import QtQuick

VoiceButton {
    // The bar, its button as the hub writes it with its place in its row, and whether it sits under a note's line.
    required property var host
    required property var modelData
    required property int index
    property bool under: false

    anchors.verticalCenter: parent ? parent.verticalCenter : undefined
    button: modelData
    round: !under && (host.front.indexOf(modelData) >= 0 || modelData.id === host.names.gear || index === host.back.length - 1)
    lk: host.lk
    res: host.res
    s: host.s
    ar: host.ar
    uid: host.res.uid(host.names.group, modelData.id)
    done: modelData.id === host.names.copy && host.view && host.copiedAt === host.view.round ? host.view.words.copied : ""
    onClicked: host.press(modelData)
}
