// A home row (mock-up home()): the page's tile and its name over what it holds now; a click opens that page.
import QtQuick

VoiceRow {
    id: row

    tile: entry ? lk.icons.voice.panel.pages[entry.tab] || "" : ""
    name: entry ? entry.label : ""
    line: entry ? entry.value : ""
    onClicked: host.send(host.names.acts.open, entry.tab, 0)
}
