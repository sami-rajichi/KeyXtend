// The Voice panel's pager (mock-up .pg) beside a page's body, a list's or a card's: Back and Next turn the hub's page on
// the round it was drawn from, shown only with more than one page, dimmed at either end and while the pages are held.
import QtQuick

Pager {
    // The panel window, whose view, look, names and Ui object it takes.
    required property var host
    // The height its page keeps for it: its own while it shows, so it never spills past the page at few rows.
    readonly property real least: pages > 1 ? implicitHeight : 0

    page: host.view.pages.at
    pages: host.view.pages.count
    canBack: host.view.pages.at > 0 && !host.held
    canOn: host.view.pages.at < host.view.pages.count - 1 && !host.held
    ui: host.ui
    lk: host.lk
    res: host.res
    s: host.s
    backId: host.res.uid(host.names.group, host.names.ids.pageBack)
    onId: host.res.uid(host.names.group, host.names.ids.pageOn)
    onTurn: by => host.send(host.names.acts.page, "", by)
}
