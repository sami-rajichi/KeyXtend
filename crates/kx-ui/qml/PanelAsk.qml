// A list panel's one question, as the clipboard's Delete all and Quick-fill's questions ask it: stacked, it takes the
// list's place; laid flat, it takes one row's, with a tile for its icon; and it rises in. Only a click on Yes acts, as
// a destructive answer is never the default, and No keeps all.
import QtQuick

Item {
    id: ask

    // The question, or empty while none shows; Yes's and No's labels and gate ids; the question's icon, and No's; the
    // move it rises with; its inset at the sides; the look, fonts, icons and size.
    required property string question
    required property string yes
    required property string no
    required property string yesId
    required property string noId
    required property string icon
    required property string noIcon
    required property var move
    property real side: 0
    required property var lk
    required property var res
    required property real s
    // Yes's icon, the question's unless given, and its look: danger, or Enter's where Yes loses nothing.
    property string yesIcon: icon
    property string yesTone: "danger"
    // Laid flat on one row, its text from the language's start side.
    property bool flat: false
    property bool rtl: false

    // Yes (true) or No (false) answered it.
    signal answered(bool agreed)

    readonly property var pn: lk.shape.panel

    visible: !!question
    onVisibleChanged: if (visible) rise.play()
    Accessible.role: Accessible.AlertMessage
    Accessible.name: question
    Accessible.ignored: !question

    // Takes every press and hover in the list's place while it shows.
    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
    }
    Rise {
        id: rise
        anchors.fill: parent
        move: ask.move
        from: ask.lk.motion.amount.pop_rise_px * ask.s

        Grid {
            id: stack
            anchors.centerIn: parent
            width: parent.width - 2 * ask.side
            columns: ask.flat ? 3 : 1
            columnSpacing: ask.pn.row_gap_px * ask.s
            rowSpacing: ask.pn.question_gap_px * ask.s
            horizontalItemAlignment: Grid.AlignHCenter
            verticalItemAlignment: Grid.AlignVCenter
            PanelTile {
                id: tile
                visible: ask.flat
                icon: ask.icon
                lk: ask.lk
                res: ask.res
                s: ask.s
            }
            Icon {
                visible: !ask.flat
                icon: ask.icon
                base: ask.res.icons
                ext: ask.res.ext
                px: ask.pn.empty_px * ask.s
                tint: ask.lk.palette.pop_muted
            }
            // Flat, a long name ends in an ellipsis so the row keeps its height; readers still hear the whole question.
            Label {
                width: ask.flat ? stack.width - tile.width - answers.width - 2 * stack.columnSpacing : stack.width
                text: ask.question
                wrapMode: Text.Wrap
                maximumLineCount: ask.flat ? ask.pn.question_lines : undefined
                elide: ask.flat ? Text.ElideRight : Text.ElideNone
                horizontalAlignment: !ask.flat ? Text.AlignHCenter : ask.rtl ? Text.AlignRight : Text.AlignLeft
                font.family: ask.res.fam.ui
                font.pixelSize: ask.pn.title_px * ask.s
                color: ask.lk.palette.pop_ink
            }
            Row {
                id: answers
                spacing: ask.lk.shape.clip.foot_gap_px * ask.s
                ClipButton {
                    label: ask.no
                    icon: ask.noIcon
                    tone: "ghost"
                    can: true
                    lk: ask.lk
                    res: ask.res
                    s: ask.s
                    uid: ask.noId
                    onClicked: ask.answered(false)
                }
                ClipButton {
                    label: ask.yes
                    icon: ask.yesIcon
                    tone: ask.yesTone
                    can: true
                    lk: ask.lk
                    res: ask.res
                    s: ask.s
                    uid: ask.yesId
                    onClicked: ask.answered(true)
                }
            }
        }
    }
}
