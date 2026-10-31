// The window guard's runs for the keyboard window `win` (ADR-0013): each try guards our shown windows, has the keyboard's
// entries watched while the scroll module runs and the open list or clipboard panel marked as paging, cuts the keyboard
// and puts its backdrop on, and a try that finds a window not shown yet tries again after the delay.
import QtQuick

Timer {
    id: guard

    // The keyboard window, whose keyboard object makes each try.
    required property Window win
    // Our window to land in front until the run ends; its title is read at each try, so a new language still finds it.
    property Window top: null

    interval: win.cfg ? win.cfg.guardDelayMs : 0
    onTriggered: {
        const again = win.keyboard.guard(top ? top.title : "");
        win.watch();
        if (win.panel)
            win.panel.list.mark();
        if (win.clips)
            win.clips.pane.mark();
        win.reshape();
        win.frosted = win.ui.backdrop(win.title);
        if (again)
            restart();
        else
            top = null;
    }

    // Starts a run, or the running one again; window `pop`, if given, lands in front.
    function run(pop) {
        if (pop)
            top = pop;
        restart();
    }
}
