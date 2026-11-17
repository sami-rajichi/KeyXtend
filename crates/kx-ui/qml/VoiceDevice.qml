// A device row (mock-up device()): one tick, its kind's tile, the name over its memory, then its mark, or the check's
// line over its progress while it runs.
pragma ComponentBehavior: Bound
import QtQuick

VoiceRow {
    id: row

    readonly property var check: entry ? entry.check : null

    tick: entry ? entry.tick : undefined
    tile: entry ? lk.icons.voice.panel.kinds[entry.kind] || "" : ""
    name: entry ? entry.name : ""
    line: entry ? entry.line : ""
    onClicked: host.send(host.names.acts.row, "", place)

    Repeater {
        model: row.entry && row.entry.mark && !row.check ? [row.entry.mark] : []
        delegate: VoiceMark {
            required property var modelData
            anchors.verticalCenter: parent ? parent.verticalCenter : undefined
            host: row.host
            mark: modelData
            lit: row.lit
        }
    }
    VoiceDoing {
        visible: !!row.check
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        owner: row
        text: row.check ? row.check.line : ""
        VoiceProgress {
            width: parent.width
            share: row.check ? row.check.share : 0
            lit: row.lit
            lk: row.lk
            s: row.s
        }
    }
}
