// A service on the Cloud page (mock-up cloudPage()): one tick, its name over its free use or what the Custom row asks
// for, then Untested, the warning and where it stands. While cloud services are off its row only shows, unticked.
pragma ComponentBehavior: Bound
import QtQuick

VoiceRow {
    id: row

    readonly property bool live: host.sheet.switch.on
    readonly property var tag: entry && entry.tag ? [{ id: host.names.untested, text: entry.tag }] : []

    tick: entry && live ? entry.tick : undefined
    name: entry ? entry.name : ""
    line: entry ? entry.line : ""
    takes: live
    onClicked: host.send(host.names.acts.row, "", place)

    Repeater {
        model: row.entry ? row.tag.concat(row.entry.marks) : []
        delegate: VoiceMark {
            required property var modelData
            anchors.verticalCenter: parent ? parent.verticalCenter : undefined
            host: row.host
            mark: modelData
            lit: row.lit
        }
    }
}
