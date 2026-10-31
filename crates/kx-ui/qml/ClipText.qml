// The preview page's long text, paged to its box as the pages are needed, never scrolled (ADR-0008): each page ends
// where the first line that does not fit starts, found with an editor's `positionAt` at the box's bottom and checked
// against the box, so every page fits and the pages join back to the whole text. Arabic reads from the right in the
// Arabic font.
import QtQuick

Item {
    id: pages

    // The whole text, whether it is Arabic and reads right to left, the look, fonts and size.
    required property string text
    required property bool arabic
    required property bool rtl
    required property var lk
    required property var res
    required property real s

    // Where each page laid out so far ends in the text, the page shown (from 0), and whether the last is laid out.
    property var ends: []
    property int at: 0
    property bool done: true
    readonly property int count: ends.length
    readonly property int from: at > 0 ? ends[at - 1] || 0 : 0
    // The shown page's label, for the tests.
    readonly property alias page: shown
    readonly property real px: lk.shape.clip.preview_px[arabic ? 1 : 0] * s

    // Pages the text again for the box it has now: up to the page shown, which stays as near as it can, and the next
    // at once; `stepper` lays out the rest. With no room it calls nothing, as a closing panel's cut runs late.
    function cut() {
        if (!(width > 0 && height > 0 && text.length > 0)) {
            done = true;
            ends = [text.length];
            return;
        }
        lay([], at + 2);
        at = Math.min(at, ends.length - 1);
    }
    // Lays out pages after `out` until there are `want` or the text ends, then shows them.
    function lay(out, want) {
        let last = false;
        while (!last && out.length < want) {
            out.push(pageEnd(out.length > 0 ? out[out.length - 1] : 0));
            last = out[out.length - 1] >= text.length;
        }
        done = last;
        ends = out;
    }
    // Lays out pages until there are `want`, unless every page is.
    function grow(want) {
        if (!done && ends.length < want)
            lay(ends.slice(), want);
    }
    // The pages no longer fit the text or the box: none grows until they are cut again on the next turn.
    function again() {
        done = true;
        Qt.callLater(cut);
    }
    // Whether the text from `from` to `end` fits the box.
    function fits(from, end) {
        measure.text = text.slice(from, end);
        return measure.contentHeight <= height;
    }
    // Where the page from `from` ends: a chunk grows until it overflows, the overflowing line's start ends the page,
    // and a check against the box backs off if a mixed line placed it wrong. Each page holds a character at least.
    function pageEnd(from) {
        let size = Math.max(1, Math.ceil(width * height / (px * px)));
        while (fits(from, from + size)) {
            if (from + size >= text.length)
                return text.length;
            size *= 2;
        }
        probe.text = measure.text;
        let end = from + probe.positionAt(rtl ? probe.width - 1 : 0, height);
        if (end > from + 1 && !fits(from, end))
            end = largest(from, end - 1);
        return Math.max(end, from + 1);
    }
    // The largest end up to `high` whose text from `from` fits, a character at least, moved back to the start of a word
    // it would split, as Text wraps there.
    function largest(from, high) {
        let low = from + 1;
        while (low < high) {
            const mid = Math.ceil((low + high) / 2);
            if (fits(from, mid))
                low = mid;
            else
                high = mid - 1;
        }
        const word = Math.max(text.lastIndexOf(" ", low - 1), text.lastIndexOf("\n", low - 1)) + 1;
        return word > from ? word : low;
    }

    onTextChanged: {
        again();
        at = 0;
    }
    onWidthChanged: again()
    onHeightChanged: again()
    onPxChanged: again()
    // A turn lays out the page after the shown one, so Next is on whenever there is more.
    onAtChanged: grow(at + 2)
    Component.onCompleted: cut()
    clip: true

    // Lays out one more page each frame while shown until the last, so a long text never holds up a frame.
    FrameAnimation {
        id: stepper
        running: pages.visible && !pages.done
        onTriggered: pages.grow(pages.ends.length + 1)
    }

    Label {
        id: shown
        width: parent.width
        text: pages.text.slice(pages.from, pages.ends[pages.at] || 0)
        wrapMode: Text.Wrap
        horizontalAlignment: pages.rtl ? Text.AlignRight : Text.AlignLeft
        font.family: pages.arabic ? pages.res.fam.arabic : pages.res.fam.latin
        font.pixelSize: pages.px
        color: pages.lk.palette.pop_ink
        Accessible.ignored: false
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
    // Laid out as the shown page is, to measure each page before it shows.
    Label {
        id: measure
        visible: false
        width: parent.width
        wrapMode: Text.Wrap
        horizontalAlignment: shown.horizontalAlignment
        font: shown.font
    }
    // An editor laid out the same, as only an editor maps a point to a character; it never shows or takes focus.
    TextEdit {
        id: probe
        visible: false
        enabled: false
        readOnly: true
        activeFocusOnPress: false
        width: parent.width
        wrapMode: TextEdit.Wrap
        textFormat: Text.PlainText
        horizontalAlignment: shown.horizontalAlignment
        font: shown.font
    }
}
