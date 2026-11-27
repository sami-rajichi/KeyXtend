// The theme's sizes Snip places, steps and aims by (`Snip.fit`): the strip's and record bar's heights and their gap in
// logical pixels, the editor's thickness steps and text sizes in the picture's pixels with each size's average letter
// width from the font's metrics, a click's slop in window pixels, and a frame handle's click area in logical pixels.
pragma ComponentBehavior: Bound
import QtQuick

QtObject {
    id: fit

    // The look and fonts; the text is the keyboard's language's, as the editor types in it.
    required property var lk
    required property var res

    readonly property var sh: lk.shape.snip

    // The sizes as `Snip.fit` reads them, in whole pixels: `{strip, bar, gap, thick: [{pen, marker, disc, head}],
    // text: [{height, letter}], slop, handle}`. The editor's steps take display scale `scale` (1 when left out), as a
    // picture holds its screen's own pixels; a step past a shorter list's end takes its last size.
    function sizes(scale) {
        const k = scale || 1;
        const px = v => Math.ceil(v * k);
        const at = (list, i) => px(list[Math.min(i, list.length - 1)]);
        const thick = sh.pen_px.map((pen, i) => ({ pen: px(pen), marker: at(sh.marker_px, i), disc: at(sh.disc_px, i), head: at(sh.head_px, i) }));
        const text = sh.text_px.map((h, i) => ({ height: px(h), letter: letter(i, k) }));
        return {
            strip: Math.ceil(sh.key_px + 2 * sh.plate_pad_px),
            bar: Math.ceil(sh.rec_px),
            gap: Math.ceil(sh.gap_px),
            thick: thick,
            text: text,
            slop: Math.ceil(sh.slop_px),
            handle: Math.ceil(sh.handle_hit_px)
        };
    }
    // Text size `i`'s average letter width at scale `k`, at least one pixel: the wider of the Latin and Arabic mark
    // fonts, as typed text draws in either.
    function letter(i, k) {
        const m = letters.objectAt(i);
        const wide = m ? Math.max(m.latin.averageCharacterWidth, m.arabic.averageCharacterWidth) : 0;
        return Math.max(1, Math.round(wide * k));
    }

    // Each text size's metrics in the marks' fonts and weight (SnipMark.qml).
    property Instantiator letters: Instantiator {
        model: fit.sh.text_px
        delegate: QtObject {
            id: size
            required property real modelData
            readonly property FontMetrics latin: FontMetrics {
                font: Qt.font({ family: fit.res.fam.latin, pixelSize: size.modelData, weight: fit.sh.mark_weight })
            }
            readonly property FontMetrics arabic: FontMetrics {
                font: Qt.font({ family: fit.res.fam.arabic, pixelSize: size.modelData, weight: fit.sh.mark_weight })
            }
        }
    }
}
