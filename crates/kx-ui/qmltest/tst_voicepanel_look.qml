// The Voice panel's look and place: every mark, progress line, switch, band and row text reads in each theme, light
// and soft dark, and in high contrast; Arabic flips the text while each part keeps its place (ADR-0020); the panel
// opens where the hub puts it by the bar and the keyboard, never takes focus and is paged by the scroll pad; the bar's
// shadow holds it while a view carries one.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/voice.js" as Voice
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "../../kx-theme/fixtures/look-native-dark.js" as ND
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NH
import "../../kx-theme/fixtures/look-native-light-reduced.js" as NR
import "../../kx-theme/fixtures/look-et66-light.js" as EL
import "../../kx-theme/fixtures/look-et66-dark.js" as ED
import "../../kx-theme/fixtures/look-dolch-light.js" as DL
import "../../kx-theme/fixtures/look-dolch-dark.js" as DD

Item {
    id: root
    width: 200
    height: 100

    // The keyboard's top in Qt units, and where the hub puts the bar for each place, in physical pixels: above it.
    readonly property real kbTop: 240
    readonly property var spots: ({ above: { x: 100, y: 100 }, bottom: { x: 100, y: 500 }, top: { x: 100, y: 10 } })

    // The keyboard window, a stand-in Voice object placing the bar above it, the bar and the panel made while `shown`
    // holds a view, made and torn down together as the app's windows are.
    Component {
        id: rigC
        Item {
            id: rig
            property var shown: null
            readonly property alias win: board
            readonly property alias voice: stub
            readonly property alias bar: strip
            readonly property var panel: panels.item
            KbWindow {
                id: board
                ui: FakeUi {}
                keyboard: FakeKeyboard {}
            }
            FakeVoice {
                id: stub
                spots: root.spots
            }
            VoiceBar {
                id: strip
                voice: stub
                lk: board.lk
                res: board.res
                s: board.s
                ar: false
                box: () => board.spots.beside(NaN)[0]
                logical: (x, y) => board.spots.logical(x, y)
            }
            Loader {
                id: panels
                active: rig.shown !== null
                sourceComponent: VoicePanel {
                    win: board
                    bar: strip
                    view: rig.shown.panel
                    round: rig.shown.round
                }
            }
        }
    }
    // The keyboard window, a stand-in Voice object and the app as VoiceGlow reads it, with the bar's shadow over them.
    Component {
        id: glowRigC
        Item {
            readonly property alias win: board
            readonly property alias voice: stub
            readonly property alias glow: shade
            KbWindow {
                id: board
                ui: FakeUi {}
                keyboard: FakeKeyboard {}
            }
            FakeVoice {
                id: stub
            }
            QtObject {
                id: shell
                property QtObject voice: stub
                property Window face: board
                property var menu: ({ shut: () => {} })
                function shadow(glow, on, top) {
                    glow.visible = on;
                }
            }
            VoiceGlow {
                id: shade
                app: shell
            }
        }
    }

    VoiceCase {
        name: "voicepanel_look"
        host: root

        // The colour under a row's parts: the panel, the ticked row's fill, then the hover tint.
        function under(row, p) {
            const base = row.on ? Kit.over(p.pop_sel, p.pop_bg) : p.pop_bg;
            return row.hot ? Kit.over(p.pop_hover, base) : base;
        }
        // Each mark, progress line and text on `row` reads on what lies under it.
        function rowReads(row, p, at) {
            const bg = under(row, p);
            for (const t of [row.nameText, row.lineText].filter(t => t.visible && t.text))
                verify(Kit.ratio(t.color, bg) >= Kit.text, at + ": " + t.text);
            for (const m of marks(row)) {
                const chip = Kit.over(m.fill, bg);
                verify(Kit.ratio(m.ink, chip) >= Kit.text, at + ": the mark " + m.markId);
                verify(!m.icon || Kit.ratio(m.tint, chip) >= Kit.shapes, at + ": the mark " + m.markId + "'s icon");
            }
            for (const b of Board.all(row, o => o.visible && o.share !== undefined && o.track !== undefined))
                verify(Kit.ratio(b.fill, Kit.over(b.track, bg)) >= Kit.shapes, at + ": the progress line");
            for (const s of Board.all(row, o => o.visible && o.knob !== undefined)) {
                const track = Kit.over(s.track, bg);
                verify(s.on ? Kit.ratio(s.knob, track) >= Kit.shapes : Kit.ratio(s.line, bg) >= Kit.shapes, at + ": the switch");
            }
        }
        // Every row and band of view `name` reads in look `lk`, at rest and with the pointer over row `hot`, if it takes clicks.
        function reads(name, lk, hot) {
            const box = make(name, "en", lk);
            const p = lk.palette;
            for (const b of Board.all(box.contentItem, o => o.visible && o.alert !== undefined && o.tint !== undefined)) {
                const bg = Kit.over(b.fill, p.pop_bg);
                verify(Kit.ratio(b.ink, bg) >= Kit.text, name + ": the band " + b.text);
                verify(Kit.ratio(b.tint, bg) >= Kit.shapes, name + ": the band's icon");
            }
            const all = Board.all(box.contentItem, o => o.visible && o.place !== undefined && o.entry !== undefined && o.filled);
            all.forEach((r, i) => rowReads(r, p, name + " row " + i));
            if (!all[hot].takes) {
                box.visible = false;
                return;
            }
            mouseMove(all[hot], all[hot].width / 2, all[hot].height / 2);
            tryVerify(() => all[hot].hot, 2000, name + ": the pointer is over row " + hot);
            rowReads(all[hot], p, name + " row " + hot + " under the pointer");
            box.visible = false;
        }

        function test_every_part_reads_in_each_theme_and_mode_data() {
            const looks = { "native-light": NL, "native-dark": ND, "native-contrast": NH, "native-reduced": NR, "et66-light": EL, "et66-dark": ED, "dolch-light": DL, "dolch-dark": DD };
            return Object.entries(looks).map(([tag, f]) => ({ tag: tag, lk: f.lk }));
        }
        function test_every_part_reads_in_each_theme_and_mode(d) {
            for (const [name, hot] of [["models", 2], ["modelsMore", 1], ["modelsSlow", 0], ["device", 4], ["comparing", 1], ["options", 0], ["optionsMore", 1], ["history", 2], ["results", 0]])
                reads(name, d.lk, hot);
        }

        // Arabic flips the text only (ADR-0020): the tick box stays left, the marks right, and the buttons in order.
        function test_arabic_reads_right_to_left_while_each_part_keeps_its_place() {
            const en = make("models");
            const ar = make("models", "ar");
            const x = o => o.mapToItem(null, 0, 0).x;
            const row = rows(ar)[1];
            compare(row.nameText.effectiveHorizontalAlignment, Text.AlignRight, "the name reads from the right");
            compare(row.lineText.effectiveHorizontalAlignment, Text.AlignRight);
            compare(row.nameText.font.family, ar.res.fam.ui, "the theme's Arabic font");
            const tick = Board.all(row, o => o.visible && o.number !== undefined)[0];
            verify(x(tick) < x(row.nameText), "the tick box stays left");
            verify(x(marks(row)[0]) > x(row.nameText) + row.nameText.width / 2, "the marks stay right");
            compare(buttons(ar.foot).map(b => b.Accessible.id), buttons(en.foot).map(b => b.Accessible.id), "the footer keeps its order");
            verify(x(one(ar, "voice.back")) < x(one(ar, "voice.close")), "Back left, Close right");
            en.visible = ar.visible = false;
        }

        // Shows `win`, low enough for the bar above it, the real pointer passing through it.
        function showKeyboard(win) {
            Pointer.through([win]);
            win.at = Qt.point(0, root.kbTop);
            win.visible = true;
            waitForRendering(win.contentItem);
        }
        // The last place the panel asked the hub for, as `{over, bar, w, h}`.
        function asked(voice) {
            const asks = voice.panelAsks;
            return asks.length > 0 ? asks[asks.length - 1] : null;
        }

        // The hub places it (hub/voice/place.rs): the panel hands it the keyboard's and the bar's boxes and takes its spot
        // and fit, so it shrinks to the room the hub found.
        function test_the_panel_opens_where_the_hub_puts_it_by_the_bar_and_the_keyboard() {
            const rig = createTemporaryObject(rigC, root);
            const [win, bar, voice] = [rig.win, rig.bar, rig.voice];
            showKeyboard(win);
            Pointer.through([bar]);
            voice.page = JSON.parse(JSON.stringify(Voice.views.en.text));
            tryVerify(() => bar.visible, 2000, "the bar shows");
            voice.panelSpot = { x: 120, y: 30, fit: 0.9 };
            rig.shown = Kit.view("en", "home");
            tryVerify(() => !!rig.panel && rig.panel.visible && rig.panel.pane.visible, 2000, "the panel and its shadow show");
            const panel = rig.panel;
            Pointer.through([panel.pane]);
            const a = win.spots.physical(bar.x, bar.y);
            const box = () => JSON.stringify({ x: a.x, y: a.y, w: bar.width * a.dpr, h: bar.height * a.dpr });
            tryVerify(() => JSON.stringify(asked(voice).bar) === box(), 2000, "the bar's box, asked again as its text lays out");
            bar.height += 10;
            tryVerify(() => JSON.stringify(asked(voice).bar) === box(), 2000, "asked again as the bar grows, so it never covers the panel");
            const ask = asked(voice);
            compare(ask.over, JSON.parse(win.spots.beside(NaN)[0]), "the keyboard's box");
            compare(ask.w, panel.wide * win.s, "its full width");
            const at = win.spots.logical(120, 30);
            // A window sits on whole logical pixels, so its spot may round down by one.
            fuzzyCompare(panel.pane.x, at.x, 1, "at the hub's spot, across");
            fuzzyCompare(panel.pane.y, at.y, 1, "at the hub's spot, down");
            compare(panel.fit, 0.9, "at the hub's fit");
            verify(panel.pane.flags & Qt.WindowDoesNotAcceptFocus, "never takes focus");
            compare(panel.transientParent, null, "it stays while the keyboard hides");
            tryVerify(() => panel.pane.paged, 2000, "the scroll pad pages it");
            voice.page = JSON.parse(JSON.stringify(Voice.views.en.hidden));
            tryVerify(() => asked(voice).bar === null, 2000, "with the bar gone, by the keyboard alone");
            win.visible = false;
        }

        function test_the_bars_shadow_holds_the_panel_while_a_view_carries_one() {
            const rig = createTemporaryObject(glowRigC, root);
            const [win, voice, glow] = [rig.win, rig.voice, rig.glow];
            showKeyboard(win);
            compare(glow.panel, null, "none before a view");
            voice.page = Kit.view("en", "home");
            tryVerify(() => !!glow.panel && glow.panel.pane.visible, 2000, "the panel shows");
            Pointer.through([glow.panel.pane]);
            mouseClick(one(glow.panel.pane, "voice.close"));
            compare(voice.calls, ["act button close 0 " + voice.page.round], "its clicks reach the Voice object");
            const closed = Kit.view("en", "home");
            closed.panel = null;
            voice.page = closed;
            tryVerify(() => glow.panel === null, 2000, "a view without one takes it away");
            win.visible = false;
        }
    }
}
