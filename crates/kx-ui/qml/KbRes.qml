// The fonts, icons and texts the keyboard window's parts draw with, the keyboard's and the tray's. Its functions read
// only their arguments, so the window's bindings may call them before any binding here would have run.
import QtQuick

QtObject {
    // The parts' kit from the app's Ui object, the look, the keyboard's language and the compiled-in icon folder and
    // type; `texts` and `faces`, bumped on a new language, settings or theme fonts, only rerun the binding that calls it.
    function resources(ui, lk, tongue, assets, texts, faces) {
        const fam = lk ? families(lk, tongue) : {};
        return {
            fam: fam,
            icons: assets.icons,
            ext: assets.ext,
            tr: key => ui.text(key),
            tray: key => ui.trayText(key),
            bar: id => ui.nameOf("bar", id),
            // A control's UI Automation id is its text key, such as `bar.close`, the same in every language.
            uid: (group, id) => group + "." + id,
            dpad: id => ui.nameOf("dpad", id),
            space: parts => ui.space(JSON.stringify(parts)),
            // A tip with a note, such as when a dimmed button comes: its name, then the note, on two lines.
            noteTip: (name, note) => name + "\n" + note
        };
    }
    // The Latin, Arabic and mark families (each list's first that Qt has, else its last), and the keyboard's and
    // tray's, from the look's lists and the keyboard's language.
    function families(lk, tongue) {
        const all = Qt.fontFamilies();
        const pick = list => list.find(n => all.indexOf(n) >= 0) || list[list.length - 1];
        const latin = pick(lk.look.latin_fonts);
        const arabic = pick(lk.look.arabic_fonts);
        const mark = pick(lk.look.mark_fonts);
        const of = l => l && l.rtl ? arabic : latin;
        return { latin: latin, arabic: arabic, mark: mark, ui: of(tongue), tray: of(tongue && tongue.tray) };
    }
}
