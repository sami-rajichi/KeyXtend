// A model in Compare's run (mock-up comparing()): its name over why a cloud model failed, then its mark, or Writing
// over the moving line while it writes; the row takes no click.
pragma ComponentBehavior: Bound
import QtQuick

VoiceRow {
    id: row

    readonly property var step: entry ? entry.mark : null
    readonly property bool writes: !!step && step.id === host.names.writing

    name: entry ? entry.name : ""
    line: entry ? entry.why || "" : ""
    takes: false

    Repeater {
        model: row.step && !row.writes ? [row.step] : []
        delegate: VoiceMark {
            required property var modelData
            anchors.verticalCenter: parent ? parent.verticalCenter : undefined
            host: row.host
            mark: modelData
            lit: row.lit
        }
    }
    VoiceDoing {
        visible: row.writes
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        owner: row
        text: row.writes ? row.step.text : ""
        VoiceProgress {
            width: parent.width
            share: 0
            running: true
            lit: row.lit
            lk: row.lk
            s: row.s
        }
    }
}
