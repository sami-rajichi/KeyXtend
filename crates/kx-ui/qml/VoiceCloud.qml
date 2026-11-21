// A service on the Cloud page (mock-up cloudPage()): one tick, its name with the Untested tag beside it over its free
// allowance or what the Custom row asks for, then the warning, where it stands and a sealed key's Remove key. While
// cloud services are off its row only shows, unticked and dimmed but for Remove key, which still acts (mock-up .pr.off).
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
    fade: live ? 1 : pn.off_share
    onClicked: host.send(host.names.acts.row, "", place)

    Repeater {
        model: row.entry ? row.entry.marks : []
        delegate: VoiceMark {
            required property var modelData
            anchors.verticalCenter: parent ? parent.verticalCenter : undefined
            opacity: row.fade
            host: row.host
            mark: modelData
            lit: row.lit
        }
    }
    // Its icon alone, so the free allowance's line keeps its width in French and Arabic; readers hear its label.
    VoicePress {
        visible: !!row.entry && !!row.entry.remove
        bare: true
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        host: row.host
        button: row.entry ? row.entry.remove : null
        onClicked: row.host.press(button)
    }
}
