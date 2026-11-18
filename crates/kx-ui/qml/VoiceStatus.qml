// What the bar does while it listens or writes (design/voice-bar.html .st, .lc): the red dot and Listening, or Nothing
// heard yet in its place, then the language's chip; or the audio mark and Transcribing; then the cloud mark while a
// service takes the voice. Words follow the keyboard's language, the chip the language's own script.
import QtQuick

Row {
    id: st

    // The bar it sits on, whose view, look, fonts and size it reads.
    required property var host

    readonly property var shown: host.shown
    readonly property var cap: host.cap
    readonly property var chip: host.lk.shape.chip
    readonly property real scale: host.ar ? host.lk.look.arabic_scale : 1
    readonly property var lang: shown && shown.lang ? shown.lang : null
    readonly property var cloud: host.view && host.view.cloud ? host.view.cloud : null

    spacing: host.gap

    Row {
        anchors.verticalCenter: parent.verticalCenter
        spacing: st.host.gap
        leftPadding: st.cap.inset_px * st.host.s
        rightPadding: leftPadding
        VoiceDot {
            visible: st.host.listens
            anchors.verticalCenter: parent.verticalCenter
            lk: st.host.lk
            s: st.host.s
        }
        Icon {
            visible: st.host.writes
            anchors.verticalCenter: parent.verticalCenter
            icon: st.host.lk.icons.voice.transcribing
            base: st.host.res.icons
            ext: st.host.res.ext
            px: st.cap.icon_px * st.host.s
            tint: st.host.p.pop_muted
        }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            text: !st.shown || !st.shown.word ? "" : (st.shown.hint || st.shown.word)
            font.family: st.host.res.fam.ui
            font.pixelSize: st.cap.status_px * st.host.s * st.scale
            font.weight: st.cap.status_weight
            color: st.host.p.pop_ink
        }
    }
    // High contrast draws a chip's word in ButtonText, as on the chip lane (drawn/voice.rs).
    Rectangle {
        visible: !!st.lang
        anchors.verticalCenter: parent.verticalCenter
        width: name.width + 2 * st.chip.pad_px * st.host.s
        height: st.chip.height_px * st.host.s
        radius: height / 2
        color: st.host.p.chip_bg
        border.width: st.host.lk.shape.line.px
        border.color: st.host.p.pop_line
        Label {
            id: name
            anchors.centerIn: parent
            text: st.lang ? st.lang.name : ""
            font.family: st.lang && st.lang.arabic ? st.host.res.fam.arabic : st.host.res.fam.latin
            font.pixelSize: st.chip.font_px[st.lang && st.lang.arabic ? 1 : 0] * st.host.s
            font.weight: st.chip.weight
            color: st.host.lk.contrast ? st.host.p.legend : st.host.p.pop_ink
        }
    }
    // The cloud mark: the service the voice goes to at Stop, by its own name, which readers hear in a sentence.
    Rectangle {
        visible: !!st.cloud
        anchors.verticalCenter: parent.verticalCenter
        width: where.width + 2 * st.chip.pad_px * st.host.s
        height: st.chip.height_px * st.host.s
        radius: height / 2
        color: st.host.p.chip_bg
        border.width: st.host.lk.shape.line.px
        border.color: st.host.p.pop_line
        Accessible.role: Accessible.StaticText
        Accessible.name: st.cloud ? st.cloud.said : ""
        Row {
            id: where
            anchors.centerIn: parent
            spacing: st.chip.gap_px * st.host.s
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                icon: st.host.lk.icons.voice.cloud
                base: st.host.res.icons
                ext: st.host.res.ext
                px: st.chip.icon_px * st.host.s
                tint: st.host.lk.contrast ? st.host.p.legend : st.host.p.ring
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: st.cloud ? st.cloud.name : ""
                font.family: st.host.ar ? st.host.res.fam.arabic : st.host.res.fam.latin
                font.pixelSize: st.chip.font_px[st.host.ar ? 1 : 0] * st.host.s
                font.weight: st.chip.weight
                color: st.host.lk.contrast ? st.host.p.legend : st.host.p.pop_ink
            }
        }
    }
}
