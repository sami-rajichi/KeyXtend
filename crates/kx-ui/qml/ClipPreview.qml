// The preview page (spec §7): the whole entry in the list's place, with Back. Long text and many files are paged with
// the list's page buttons, never scrolled (ADR-0008); the panel pages them itself. An image shows large, never cached,
// with the kind's icon until it comes or when the provider has none.
import QtQuick

Item {
    id: pre

    // What it shows (`{kind: "text", text, chars, arabic, rtl}`, `{kind: "image", url}` or `{kind: "files", paths}`), the
    // buttons' labels, rows per page and a row's height, the click names, the app's Ui object for the page number, the
    // look, fonts, icons and size.
    required property var shown
    required property var words
    required property int rows
    required property real rowH
    required property var names
    required property var ui
    required property var lk
    required property var res
    required property real s

    // A click: its `Ui.clipAct` kind and number.
    signal act(string kind, int arg)

    readonly property var pn: lk.shape.panel
    readonly property var c: lk.shape.clip
    readonly property string kind: shown ? shown.kind : ""
    readonly property real side: pn.foot_pad_px[1] * s
    // The paged part shown now: the text's or the files'.
    readonly property Item paged: kind === "text" ? prose : kind === "files" ? files : null

    ClipButton {
        id: back
        x: pre.c.filters_pad_px[1] * pre.s
        y: pre.c.filters_pad_px[0] * pre.s
        label: pre.words.back
        icon: pre.lk.icons.clip.back
        tone: "ghost"
        can: true
        lk: pre.lk
        res: pre.res
        s: pre.s
        uid: pre.res.uid(pre.names.group, pre.names.ids.back)
        onClicked: pre.act(pre.names.acts.back, 0)
    }
    Label {
        visible: pre.kind === "text"
        anchors.left: back.right
        anchors.leftMargin: pre.pn.head_gap_px * pre.s
        anchors.verticalCenter: back.verticalCenter
        text: visible ? pre.shown.chars : ""
        font.family: pre.res.fam.ui
        font.pixelSize: pre.c.kept_px * pre.s
        color: pre.lk.palette.pop_muted
    }
    Item {
        id: box
        x: pre.side
        y: back.y + back.height + pre.pn.body_gap_px * pre.s
        width: pre.width - 2 * pre.side - (pre.pn.body_gap_px + pre.pn.page_px[0]) * pre.s
        height: pre.height - y - pre.pn.body_pad_px[2] * pre.s

        ClipText {
            id: prose
            anchors.fill: parent
            visible: pre.kind === "text"
            text: visible ? pre.shown.text : ""
            arabic: visible && pre.shown.arabic
            rtl: visible && pre.shown.rtl
            lk: pre.lk
            res: pre.res
            s: pre.s
        }
        ClipFiles {
            id: files
            anchors.fill: parent
            visible: pre.kind === "files"
            paths: visible ? pre.shown.paths : []
            rows: pre.rows
            rowH: pre.rowH
            lk: pre.lk
            res: pre.res
            s: pre.s
        }
        // Its own size at the keyboard's scale, centred, or shrunk to the box; never cached, so an old round's texture
        // goes as the next view comes.
        Image {
            id: picture
            anchors.centerIn: parent
            width: Math.min(parent.width, sourceSize.width * pre.s)
            height: Math.min(parent.height, sourceSize.height * pre.s)
            visible: pre.kind === "image" && status === Image.Ready && sourceSize.width > 1
            source: pre.kind === "image" && pre.shown.url ? pre.shown.url : ""
            cache: false
            fillMode: Image.PreserveAspectFit
            Accessible.ignored: true
        }
        Icon {
            anchors.centerIn: parent
            visible: pre.kind === "image" && !picture.visible
            icon: pre.lk.icons.clip.kind_image
            base: pre.res.icons
            ext: pre.res.ext
            px: pre.c.empty_px * pre.s
            tint: pre.lk.palette.pop_muted
        }
    }
    Pager {
        anchors.right: parent.right
        anchors.rightMargin: pre.side
        anchors.verticalCenter: box.verticalCenter
        page: pre.paged ? pre.paged.at : 0
        pages: pre.paged ? pre.paged.count : 0
        canBack: page > 0
        canOn: page < pages - 1
        ui: pre.ui
        lk: pre.lk
        res: pre.res
        s: pre.s
        backId: pre.res.uid(pre.names.group, pre.names.ids.textBack)
        onId: pre.res.uid(pre.names.group, pre.names.ids.textOn)
        onTurn: by => pre.paged.at += by
    }
}
