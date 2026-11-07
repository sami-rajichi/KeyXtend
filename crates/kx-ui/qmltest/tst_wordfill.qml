// Quick-fill on the chip lane: in a password box the lock and its note stay and, while the vault runs, a Quick-fill
// chip follows them, whose click asks the hub for Quick-fill once; no other lane shows it, saved emails wear the key
// icon as chips and rows, and a long Arabic note elides before the chip does.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "lanes.js" as Lanes
import "fixtures/face.js" as Face
import "fixtures/ui-ar.js" as Ar
import "fixtures/ui-en.js" as En
import "fixtures/words.js" as Words
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    // The chips lane's saved email, which ends it.
    readonly property int saved: Words.lanes.chips.chips.length - 1

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: barC; TopBar {} }

    TestCase {
        name: "wordfill"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A keyboard window over fresh stand-ins whose hub gives `lane`, shown; the real pointer passes through it.
        function win(lane) {
            const ui = createTemporaryObject(uiC, root, { lane: lane });
            const w = createTemporaryObject(winC, root, { ui: ui, keyboard: createTemporaryObject(kbC, root) });
            Pointer.through([w]);
            w.visible = true;
            waitForRendering(w.contentItem);
            return w;
        }
        // The shown controls under `item` with UI Automation id `id`, and the one control with it.
        function ids(item, id) {
            return Board.all(item, o => o.visible && o.Accessible.id === id);
        }
        function keyed(item, id) {
            const hits = ids(item, id);
            compare(hits.length, 1, id);
            return hits[0];
        }
        function iconOf(item) {
            return Board.all(item, o => o.tint !== undefined && o.icon !== undefined)[0].icon;
        }
        // The calls since the last look that make the hub sound or act.
        function said(w) {
            const acts = c => c.startsWith("pickChip") || c.startsWith("wordsAsk") || c === "tapped";
            const calls = w.ui.calls.filter(acts);
            w.ui.calls = [];
            return calls;
        }

        function test_a_password_box_keeps_the_lock_and_note_and_adds_the_quick_fill_chip() {
            const w = win(Words.lanes.locked);
            const note = keyed(w.contentItem, "words.locked");
            compare(note.Accessible.name, En.texts["chip.locked"], "the note stays");
            compare(iconOf(note), NL.lk.icons.words.locked, "the lock stays");
            const fill = keyed(w.contentItem, "words.fill");
            const title = w.res.tr("fill.title");
            compare([fill.Accessible.role, fill.Accessible.name, fill.chip.text], [Accessible.Button, title, title]);
            compare(iconOf(fill), NL.lk.icons.words.fill);
            const [at, end] = [note.mapToItem(null, 0, 0).x, fill.mapToItem(null, 0, 0).x];
            verify(end >= at + note.width, "the chip follows the note: " + at + ", " + end);
            const row = fill.parent;
            verify(fill.x + fill.width <= row.width, "the chip shows whole");
        }

        function test_a_click_on_the_quick_fill_chip_asks_for_quick_fill_once() {
            const w = win(Words.lanes.locked);
            said(w);
            mouseClick(keyed(w.contentItem, "words.fill"));
            compare(said(w), ["wordsAsk " + Words.asks.fill + " 0"]);
        }

        // While the vault is not live the chip is gone and the note stays; it comes back when the vault says it runs.
        function test_a_password_box_shows_no_quick_fill_chip_until_the_vault_runs() {
            const w = win(Words.lanes.lockedOff);
            keyed(w.contentItem, "words.locked");
            compare(ids(w.contentItem, "words.fill").length, 0, "the chip is gone");
            w.ui.lane = Words.lanes.locked;
            waitForRendering(w.contentItem);
            keyed(w.contentItem, "words.fill");
            w.ui.lane = Words.lanes.lockedOff;
            waitForRendering(w.contentItem);
            compare(ids(w.contentItem, "words.fill").length, 0, "it goes again");
        }

        // Only a password box shows the chip; the words menu takes its place there too.
        function test_no_other_lane_shows_the_quick_fill_chip() {
            const lanes = ["chips", "forget", "list", "off", "empty", "stopped", "start"];
            const w = win(Words.lanes.chips);
            for (const name of lanes) {
                w.ui.lane = Words.lanes[name];
                waitForRendering(w.contentItem);
                compare(ids(w.contentItem, "words.fill").length, 0, name);
            }
            w.ui.lane = Words.lanes.locked;
            keyed(w.contentItem, "words.fill");
            mouseClick(keyed(w.contentItem, "words.menu"));
            compare(ids(w.contentItem, "words.fill").length, 0, "the menu takes the row");
        }

        function test_a_saved_email_wears_the_key_icon_as_a_chip_and_a_row() {
            const w = win(Lanes.some([3, root.saved]));
            const [known, saved] = [keyed(w.contentItem, "words.chip-0"), keyed(w.contentItem, "words.chip-1")];
            compare(saved.chip.text, Words.lanes.chips.chips[root.saved].text);
            compare([known.chip.icon, saved.chip.icon], [NL.lk.icons.words.email, NL.lk.icons.words.saved]);
            compare(iconOf(saved), NL.lk.icons.words.saved);
            w.ui.lane = Words.lanes.list;
            waitForRendering(w.contentItem);
            const kinds = Words.lanes.list.list.items.map(i => i.kind);
            const icons = kinds.map(k => k === "saved" ? NL.lk.icons.words.saved : NL.lk.icons.words.email);
            verify(kinds.indexOf("saved") >= 0 && kinds.indexOf("email") >= 0, "both kinds on the page");
            const rows = kinds.map((k, i) => keyed(w.contentItem, "words.list-row-" + i));
            compare(rows.map(iconOf), icons);
        }

        // A long Arabic note elides, never the Quick-fill chip after it, which keeps its whole width inside the row.
        function test_a_long_arabic_note_elides_and_the_quick_fill_chip_stays_whole() {
            const props = { box: Face.board.bar, lk: NL.lk, res: Board.resOf(NL.lk, Ar), s: 1, preset: 0, presets: Face.board.presets.length, sizes: Ar.lang.sizes, dark: false, lane: Words.lanes.locked, rtl: true };
            const b = createTemporaryObject(barC, root, props);
            waitForRendering(b);
            const box = keyed(b, "words.locked");
            const note = Board.all(box, o => o.truncated !== undefined)[0];
            compare(note.text, Ar.texts["chip.locked"]);
            const fill = keyed(b, "words.fill");
            const row = fill.parent;
            const room = box.width - note.width + note.implicitWidth / 2 + row.spacing + fill.full;
            b.box = Object.assign({}, Face.board.bar, { w: Face.board.bar.w - (row.width - room) });
            tryVerify(() => Math.abs(row.width - room) < 0.01, 1000, "room for half the note");
            waitForRendering(b);
            verify(note.truncated && note.width < note.implicitWidth, "the note elides");
            verify(Math.abs(fill.width - fill.full) < 0.01, "the chip keeps its whole width");
            verify(fill.x + fill.width <= row.width + 0.01, "the chip shows whole: " + fill.x + ", " + row.width);
        }
    }
}
