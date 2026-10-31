// The preview page's files: their paths as a list, rows per page as every list has (ADR-0008), each with the file icon
// and the path shortened in its middle, so its drive and name both show.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: files

    // The paths, rows per page and a row's height, the look, fonts, icons and size.
    required property var paths
    required property int rows
    required property real rowH
    required property var lk
    required property var res
    required property real s

    readonly property int per: Math.max(1, rows)
    // Where each page ends in the paths, and the page shown (from 0).
    readonly property int pages: Math.max(1, Math.ceil(paths.length / per))
    readonly property var ends: [...Array(pages).keys()].map(i => Math.min(paths.length, (i + 1) * per))
    property int at: 0
    readonly property int count: ends.length
    // The paths on the shown page.
    readonly property var shown: paths.slice(at * per, ends[at])
    readonly property var pn: lk.shape.panel

    onPathsChanged: at = 0

    Column {
        width: parent.width
        spacing: files.pn.row_px[1] * files.s
        Repeater {
            model: files.shown
            delegate: Item {
                id: line
                required property string modelData
                width: parent.width
                height: files.rowH
                Accessible.role: Accessible.ListItem
                Accessible.name: modelData

                Icon {
                    id: glyph
                    x: files.pn.row_px[2] * files.s
                    anchors.verticalCenter: parent.verticalCenter
                    icon: files.lk.icons.clip.kind_file
                    base: files.res.icons
                    ext: files.res.ext
                    px: files.lk.shape.clip.thumb_px[3] * files.s
                    tint: files.lk.palette.pop_muted
                }
                Label {
                    anchors.left: glyph.right
                    anchors.leftMargin: files.lk.shape.clip.row_gap_px * files.s
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: line.modelData
                    elide: Text.ElideMiddle
                    font.family: files.res.fam.latin
                    font.pixelSize: files.pn.row_text_px[0] * files.s
                    color: files.lk.palette.pop_ink
                }
            }
        }
    }
}
