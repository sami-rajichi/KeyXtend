// A model row (mock-up models()): one tick, the name over its size or why it does not fit, then its marks; a download
// shows its progress under its line, with Pause or Resume and Cancel at its end.
pragma ComponentBehavior: Bound
import QtQuick

VoiceRow {
    id: row

    readonly property var fetch: entry ? entry.fetch : null

    tick: entry ? entry.tick : undefined
    name: entry ? entry.name : ""
    line: entry ? entry.line : ""
    under: VoiceProgress {
        visible: !!row.fetch
        width: parent ? parent.width : 0
        share: row.fetch ? row.fetch.share : 0
        lit: row.lit
        lk: row.lk
        s: row.s
    }
    onClicked: host.send(host.names.acts.row, "", place)

    Repeater {
        model: row.entry && !row.fetch ? row.entry.marks : []
        delegate: VoiceMark {
            required property var modelData
            anchors.verticalCenter: parent ? parent.verticalCenter : undefined
            host: row.host
            mark: modelData
            lit: row.lit
        }
    }
    Repeater {
        model: row.fetch ? row.fetch.buttons : []
        delegate: VoicePress {
            required property var modelData
            anchors.verticalCenter: parent ? parent.verticalCenter : undefined
            host: row.host
            button: modelData
            onClicked: row.host.press(modelData)
        }
    }
}
