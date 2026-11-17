// A drive or folder to keep the models in (P11): its folder tile and its path, which reads left to right in every
// language; a click opens it.
import QtQuick

VoiceRow {
    id: row

    tile: lk.icons.voice.panel.folder
    name: entry || ""
    rtl: false
    arabic: false
    onClicked: host.send(host.names.acts.row, "", place)
}
