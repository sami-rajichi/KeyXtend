// One of the hub's Voice panel buttons (mock-up .btn), drawn as the clipboard's: its label and its icon by id, in its
// tone's look, or its icon alone for the header's Back and Close; one that cannot act shows dimmed, never hidden.
import QtQuick

ClipButton {
    id: b

    // The panel window, whose look, names and fonts it takes; the hub's button (`{kind, id, arg, label, on, tone}`),
    // null for none; whether it shows its icon alone; and an icon in place of its id's.
    required property var host
    required property var button
    property bool bare: false
    property string alt: ""

    readonly property var hub: button || ({ kind: "", id: "", arg: 0, label: "", on: false, tone: host.names.plain })
    readonly property var names: host.names

    // Its gate id: a question's answer, Compare's Start and a download's button by their meaning, the rest by id.
    function uidOf(h) {
        const ids = names.ids;
        if (h.kind === names.acts.confirm)
            return h.arg ? ids.yes : ids.no;
        if (h.kind === names.acts.compare)
            return ids.start;
        return h.kind === names.acts.fetch ? h.id + "-" + h.arg : h.id;
    }

    label: bare ? "" : hub.label
    icon: alt || lk.icons.voice.panel.buttons[hub.kind === names.acts.compare ? names.acts.compare : hub.id] || ""
    tone: bare ? "flat" : names.tones[hub.tone]
    can: hub.on
    lk: host.lk
    res: host.res
    s: host.s
    uid: res.uid(names.group, uidOf(hub))
    Accessible.name: hub.label
}
