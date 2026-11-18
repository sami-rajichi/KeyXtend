// One of the hub's Voice panel buttons (mock-up .btn), drawn as the clipboard's: its label and its icon by id, in its
// tone's look, or its icon alone for the header's Back and Close; one that cannot act shows dimmed, never hidden.
import QtQuick

ClipButton {
    id: b

    // The panel window, whose look, names and fonts it takes; the hub's button (`{kind, id, arg, label, on, tone}`, and
    // a `name` for readers when its label is not enough), null for none; whether it shows its icon alone; and an icon in
    // place of its id's.
    required property var host
    required property var button
    property bool bare: false
    property string alt: ""

    readonly property var hub: button || ({ kind: "", id: "", arg: 0, label: "", name: "", on: false, tone: host.names.plain })
    readonly property var names: host.names

    // Its gate id: a question's answer, Compare's Start, a line's Save and a download's button by their meaning, the rest
    // by id.
    function uidOf(h) {
        const ids = names.ids;
        if (h.kind === names.acts.confirm)
            return h.arg ? ids.yes : ids.no;
        if (h.kind === names.acts.compare)
            return ids.start;
        if (h.kind === names.acts.save)
            return ids.save + h.id;
        return h.kind === names.acts.fetch ? h.id + "-" + h.arg : h.id;
    }

    // Compare's Start and a line's Save take their kind's icon, as their id names what they act on.
    readonly property string iconId: [names.acts.compare, names.acts.save].indexOf(hub.kind) >= 0 ? hub.kind : hub.id

    label: bare ? "" : hub.label
    icon: alt || lk.icons.voice.panel.buttons[iconId] || ""
    tone: bare ? "flat" : names.tones[hub.tone]
    can: hub.on
    lk: host.lk
    res: host.res
    s: host.s
    uid: res.uid(names.group, uidOf(hub))
    Accessible.name: hub.name || hub.label
}
