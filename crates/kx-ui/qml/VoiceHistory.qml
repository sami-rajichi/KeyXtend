// A dictated text (mock-up history()): its tick number, the text in its own direction and font over when, the
// language and the model, and the pin; readers hear its tick and pin too.
import QtQuick

VoiceRow {
    id: row

    tick: entry ? entry.tick : undefined
    name: entry ? entry.text : ""
    line: entry ? entry.meta : ""
    rtl: !!entry && entry.rtl
    arabic: !!entry && entry.arabic
    Accessible.name: entry ? entry.name : ""
    onClicked: host.send(host.names.acts.row, "", place)

    Icon {
        visible: !!row.entry && row.entry.pinned
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        icon: row.lk.icons.voice.panel.pinned
        base: row.res.icons
        ext: row.res.ext
        px: row.lk.shape.clip.pin_px * row.s
        tint: row.lit ? row.lk.sel_ink : row.p.ring
    }
}
