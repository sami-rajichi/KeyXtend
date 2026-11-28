// The countdown before a video (design/snip-parts.css .count): a dark disc at the part's centre with the seconds left
// and "Recording starts in", and a Cancel keycap under it that ends the snip in one click. A press here never aims the
// keyboard's letters at Snip.
import QtQuick

Window {
    id: count

    // Snip's windows.
    required property var snips

    readonly property var lk: snips.lk
    readonly property var res: snips.res
    readonly property var sh: lk.shape.snip
    readonly property var n: snips.names
    readonly property var v: snips.view
    readonly property var cn: v && v.stage === n.stages.count ? v.count : null
    // The seconds left as last shown, kept while it hides, and the draws it asked and got since it last showed.
    property int left: 0
    property int draws: 0
    readonly property rect part: cn ? snips.box(cn.part) : Qt.rect(0, 0, 0, 0)

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus | (snips.low ? 0 : Qt.WindowStaysOnTopHint)
    title: res.tr("window.snip-count")
    color: "transparent"
    visible: !!cn && snips.up(n.stages.count, n.capture.count)
    x: Math.round(part.x + (part.width - width) / 2)
    y: Math.round(part.y + (part.height - sh.count_px) / 2)
    width: Math.ceil(Math.max(sh.count_px, cancel.width + 2 * sh.plate_pad_px))
    height: Math.ceil(sh.count_px + sh.count_gap_px + sh.key_px + 2 * sh.plate_pad_px)

    onCnChanged: if (cn) left = cn.left_s
    // Made and kept out of every capture before it ever shows.
    Component.onCompleted: snips.snip.shield(n.capture.count, title)
    onVisibleChanged: visible ? showed() : hid()

    // It shows and pops in, and its pop rests on time even if no frame comes; Windows' expose draws its first frame.
    function showed() {
        draws = 0;
        if (lk.motion.moves.snip_count.ms > 0)
            pop.restart();
        rest.restart();
        tell(left);
    }
    // It hid: its pop rests, so no later show starts from a pop cut short, and the debug log hears 0 s.
    function hid() {
        rest.stop();
        pop.complete();
        tell(0);
    }
    // Tells the debug log second `s`, 0 once hidden, with numbers only: its draws since it showed, whether Cancel is
    // held, and the frozen screens' title, by which the hub counts those Qt still shows.
    function tell(s) {
        snips.snip.counted(s, draws, cancel.down, snips.screenTitle);
    }
    // Draws it now, counting the draw when it took, so the debug log needs no handler on every frame.
    function drawn() {
        if (snips.snip.draw(title))
            draws += 1;
    }
    // A later second's digit is set: it draws in its turn, then the debug log hears that second.
    function ticked() {
        drawn();
        tell(left);
    }

    // Its pop rests at its end even if no frame drew it meanwhile, then it draws again; with reduced motion, at once.
    Timer {
        id: rest
        interval: count.lk.motion.moves.snip_count.ms
        onTriggered: {
            pop.complete();
            count.drawn();
        }
    }

    Item {
        id: body
        width: count.width
        height: count.height
        // Hidden, it lets go of a press, as Windows then sends the let-go elsewhere or never.
        enabled: count.visible

        Rectangle {
            id: disc
            anchors.horizontalCenter: parent.horizontalCenter
            width: count.sh.count_px
            height: width
            radius: width / 2
            color: count.snips.tag
            Accessible.role: Accessible.StaticText
            Accessible.name: word.text + count.res.tr("snip.join") + count.left
            Column {
                anchors.centerIn: parent
                Label {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: count.left
                    font.family: count.res.fam.latin
                    font.pixelSize: count.sh.count_num_px
                    font.weight: count.sh.count_weight
                    color: count.lk.common.snip_tag_ink
                    // Each later second draws in its turn once its digit is set here, so a lost update request never
                    // leaves an old second up.
                    onTextChanged: if (count.visible) count.ticked()
                }
                Label {
                    id: word
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: Math.min(implicitWidth, count.sh.count_word_w_px)
                    text: count.res.tr("snip.count")
                    wrapMode: Text.Wrap
                    horizontalAlignment: Text.AlignHCenter
                    font.family: count.res.fam.ui
                    font.pixelSize: count.sh.count_word_px
                    color: count.lk.common.snip_tag_ink
                }
            }
        }
        // Cancel on its plate, under the disc (Task 10's ruling: one click ends the countdown).
        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter
            y: disc.height + count.sh.count_gap_px
            width: cancel.width + 2 * count.sh.plate_pad_px
            height: cancel.height + 2 * count.sh.plate_pad_px
            radius: count.lk.look.window_radius_px
            color: count.lk.palette.plate
            border.width: count.lk.contrast ? count.lk.shape.line.px : 0
            border.color: count.lk.palette.pop_line
            SnipKey {
                id: cancel
                x: count.sh.plate_pad_px
                y: count.sh.plate_pad_px
                lk: count.lk
                res: count.res
                group: count.n.acts.rec
                key: count.n.cancel
                word: count.res.tr("snip.rec-cancel")
                icon: count.lk.icons.snip.rec.cancel
                onHit: count.snips.act(count.n.acts.rec, count.n.cancel)
            }
        }
    }
    PopIn {
        id: pop
        target: body
        move: count.lk.motion.moves.snip_count
        amount: count.lk.motion.amount
    }
}
