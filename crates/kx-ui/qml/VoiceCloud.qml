// A service on the Cloud page (mock-up cloudPage()): one tick, its name with the Untested tag beside it over its free
// allowance or what the Custom row asks for, then the warning and where it stands, Off among them. Its sealed key's
// Remove key is in the footer, for the ticked row (owner, 2026-11-21); with cloud services off the page lists none.
pragma ComponentBehavior: Bound
import QtQuick

VoiceRow {
    id: row

    // Read with a guard, as a view with no switch may reach the row before the list drops it.
    readonly property bool live: !!host.sheet.switch && host.sheet.switch.on

    tick: entry && live ? entry.tick : undefined
    name: entry ? entry.name : ""
    line: entry ? entry.line : ""
    tag: entry && entry.tag ? entry.tag : ""
    takes: live
    onClicked: host.send(host.names.acts.row, "", place)

    Repeater {
        model: row.entry ? row.entry.marks : []
        delegate: VoiceMark {
            required property var modelData
            anchors.verticalCenter: parent ? parent.verticalCenter : undefined
            host: row.host
            mark: modelData
            lit: row.lit
        }
    }
}
