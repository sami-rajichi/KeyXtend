// The keyboard's own frames on the Qt GUI thread: a press frame drawn at once (`press_frame = "now"`);
// src/bridges/frames.rs calls these.
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QEvent>
#include <QtCore/QString>
#include <QtGui/QGuiApplication>
#include <QtGui/QWindow>
#include <QtQuick/QQuickWindow>

namespace kxui {

// Draws our shown Quick window titled `title` now, as its next update would, and presents it if a change is pending;
// false when none shows.
inline bool draw_now(const QString &title)
{
    for (QWindow *window : QGuiApplication::topLevelWindows()) {
        auto *quick = qobject_cast<QQuickWindow *>(window);
        if (!quick || !quick->isExposed() || quick->title() != title)
            continue;
        QEvent update(QEvent::UpdateRequest);
        QCoreApplication::sendEvent(quick, &update);
        return true;
    }
    return false;
}

} // namespace kxui
