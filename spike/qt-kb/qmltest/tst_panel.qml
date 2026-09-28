// Offline check of the G7 panel's list: mirroring, padding rows, picking, hover and the page buttons.
import QtQuick
import QtTest
import "../qml"
import "look-panel-light.js" as L
import "look-panel-dark.js" as K

Item {
    id: root
    width: 520
    height: 360

    readonly property var res: ({ fam: { latin: "Segoe UI", arabic: "Segoe UI" }, icons: Qt.resolvedUrl("../../../target/spike-stage/qt-kb/assets/icons"), ext: "svg" })
    readonly property var fx: ({ title: "list", rows: 5, rtl: true, prev: { icon: "chevron-up", name: "prev" }, next: { icon: "chevron-down", name: "next" } })
    readonly property var pg2: ({ rows: ["a", "b", null, null, null], at: 1, count: 2, label: "2 من 2" })

    Item {
        id: host
        anchors.fill: parent
        LayoutMirroring.enabled: true
        LayoutMirroring.childrenInherit: true
    }
    Component {
        id: listC
        PanelList {
            width: 480
            height: implicitHeight
            res: root.res
            fam: "Segoe UI"
            s: 1
            fx: root.fx
            pg: root.pg2
        }
    }
    SignalSpy { id: spy; signalName: "turn" }
    SignalSpy { id: tips; signalName: "tip" }

    TestCase {
        name: "PanelList"
        when: windowShown

        function make(look) {
            const l = createTemporaryObject(listC, host, { lk: look });
            spy.clear();
            spy.target = l;
            tips.clear();
            tips.target = l;
            return l;
        }
        // The rows column, the pager column, and the delegates in the rows column.
        function parts(l) {
            const rows = l.children[0], pager = l.children[1];
            const cells = [];
            for (let i = 0; i < rows.children.length; i++)
                if (rows.children[i].real !== undefined) cells.push(rows.children[i]);
            return { rows: rows, pager: pager, cells: cells };
        }
        function same(a, b) {
            return Qt.colorEqual(a, b);
        }

        function test_right_to_left_puts_the_list_right_and_the_pager_left() {
            const p = parts(make(L.lk));
            verify(p.pager.x < p.rows.x, "pager at " + p.pager.x + ", rows at " + p.rows.x);
        }
        function test_padding_rows_are_blank_and_hidden_from_readers() {
            const p = parts(make(L.lk));
            compare(p.cells.length, 5);
            verify(p.cells[1].real && !p.cells[2].real);
            verify(p.cells[2].Accessible.ignored);
            verify(same(p.cells[4].color, "transparent"));
        }
        function test_a_picked_row_on_this_page_takes_the_selection_colour() {
            const l = make(L.lk);
            l.picked = 6;
            const c = parts(l).cells;
            verify(c[1].sel && !c[0].sel);
            verify(same(c[1].color, L.lk.palette.pop_sel));
            l.picked = 1;
            verify(!c[1].sel, "item 1 is on page 1");
        }
        function test_hover_lights_a_real_row_only() {
            const l = make(K.lk);
            const c = parts(l).cells;
            mouseMove(c[0], 10, 10);
            tryVerify(() => same(c[0].color, K.lk.palette.pop_hover));
            mouseMove(c[3], 10, 10);
            wait(0);
            verify(same(c[3].color, "transparent"));
        }
        function test_the_last_page_fades_next_and_prev_turns_back() {
            const l = make(L.lk);
            const pager = parts(l).pager;
            const prev = pager.children[0], next = pager.children[2];
            verify(prev.enabled && !next.enabled);
            compare(next.opacity, L.lk.shape.panel.off_share);
            mouseClick(next);
            compare(spy.count, 0, "a faded button does nothing");
            mouseClick(prev);
            compare(spy.count, 1);
            compare(spy.signalArguments[0][0], 0);
        }
        function test_a_framed_button_hovers_to_the_selection_colour() {
            const l = make(L.lk);
            const prev = parts(l).pager.children[0];
            mouseMove(prev, 5, 5);
            tryVerify(() => same(prev.color, L.lk.palette.pop_sel));
            verify(same(prev.border.color, L.lk.palette.pop_line));
        }
        // The light look as Windows high contrast paints it: both fills Highlight, text on them HighlightText.
        function contrast() {
            const hc = JSON.parse(JSON.stringify(L.lk));
            hc.contrast = true;
            hc.palette.pop_hover = hc.palette.pop_sel = "#FF1AEBFF";
            hc.palette.pop_bg = hc.sel_ink = "#FF000000";
            hc.palette.pop_ink = hc.palette.pop_muted = "#FFFFFFFF";
            return hc;
        }
        function test_high_contrast_text_on_a_lit_row_or_button_uses_highlight_text() {
            const hc = contrast();
            const p = parts(make(hc));
            const prev = p.pager.children[0];
            verify(same(prev.color, hc.palette.pop_bg), "a framed button rests on the panel colour");
            mouseMove(prev, 5, 5);
            tryVerify(() => same(prev.children[0].color, hc.sel_ink));
            mouseMove(p.cells[0], 10, 10);
            tryVerify(() => same(p.cells[0].children[0].color, hc.sel_ink));
            verify(same(p.cells[1].children[0].color, hc.palette.pop_ink), "a row at rest");
        }
        function test_page_buttons_name_themselves_on_hover() {
            const prev = parts(make(L.lk)).pager.children[0];
            mouseMove(prev, 5, 5);
            tryCompare(tips, "count", 1);
            compare(tips.signalArguments[0][0], "prev");
            mouseMove(prev, -40, 5);
            tryCompare(tips, "count", 2);
            compare(tips.signalArguments[1][0], "");
        }
        function test_readers_hear_the_rows_as_a_named_list() {
            const rows = parts(make(L.lk)).rows;
            compare(rows.Accessible.role, Accessible.List);
            compare(rows.Accessible.name, "list");
        }
        function test_the_page_label_and_rows_use_the_panel_sizes() {
            const l = make(L.lk);
            const sh = L.lk.shape.panel;
            compare(l.implicitHeight, sh.body_pad_px[0] + sh.body_pad_px[2] + 5 * sh.row_px[0] + 4 * sh.row_px[1]);
            const label = parts(l).pager.children[1];
            compare(label.text, "2 من 2");
            compare(label.font.pixelSize, Math.round(sh.count_px));
        }
    }
}
