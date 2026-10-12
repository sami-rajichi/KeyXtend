// The keyboard's own frames on the Qt GUI thread: a press frame drawn at once (`press_frame = "now"`), never from
// inside another frame; src/bridges/frames.rs calls these.
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QEvent>
#include <QtCore/QLatin1String>
#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtGui/QGuiApplication>
#include <QtGui/QWindow>
#include <QtQuick/QQuickWindow>
#include <rhi/qrhi.h>

#include "kx-ui-build.h"
#include "rust/cxx.h"

namespace kxui {

// The object name of a window's frame mark.
inline constexpr char FRAME_MARK[] = "kxFrameMark";

// Marks its Quick window mid-frame while the window handles an update request or an exposure, the events that draw
// its frames, by handing each to the window itself in between.
class FrameMark : public QObject
{
public:
    explicit FrameMark(QQuickWindow *window) : QObject(window)
    {
        setObjectName(QLatin1String(FRAME_MARK));
        window->installEventFilter(this);
    }

    bool drawing() const
    {
        return depth_ > 0;
    }

    bool eventFilter(QObject *target, QEvent *event) override
    {
        if (event->type() != QEvent::UpdateRequest && event->type() != QEvent::Expose)
            return false;
        ++depth_;
        target->event(event);
        --depth_;
        return true;
    }

private:
    int depth_ = 0;
};

// Window `window`'s frame mark, made on first use.
inline FrameMark *frame_mark(QQuickWindow *window)
{
    QObject *mark = window->findChild<QObject *>(QLatin1String(FRAME_MARK), Qt::FindDirectChildrenOnly);
    return mark ? static_cast<FrameMark *>(mark) : new FrameMark(window);
}

// Our shown Quick window titled `title`, or none.
inline QQuickWindow *shown_quick(const QString &title)
{
    for (QWindow *window : QGuiApplication::topLevelWindows()) {
        auto *quick = qobject_cast<QQuickWindow *>(window);
        if (quick && quick->isExposed() && quick->title() == title)
            return quick;
    }
    return nullptr;
}

// True while `window` draws a frame: it handles a frame's event, or its device records a frame, as before the mark
// first saw an event.
inline bool mid_frame(QQuickWindow *window)
{
    QRhi *rhi = window->rhi();
    return frame_mark(window)->drawing() || (rhi && rhi->isRecordingFrame());
}

// Draws our shown Quick window titled `title` now, as its next update would, and presents it if a change is pending;
// false when none shows or it is mid-frame.
inline bool draw_now(const QString &title)
{
    QQuickWindow *quick = shown_quick(title);
    if (!quick || mid_frame(quick))
        return false;
    QEvent update(QEvent::UpdateRequest);
    QCoreApplication::sendEvent(quick, &update);
    return true;
}

#if defined(KX_UI_PROBES)
// Draws window `title` at once and, from inside that frame, asks for it again; the inner answers. Debug builds only.
inline rust::Vec<bool> mid_frame_draws(const QString &title)
{
    rust::Vec<bool> answers;
    QQuickWindow *quick = shown_quick(title);
    if (!quick)
        return answers;
    const auto inner = [&answers, &title] { answers.push_back(draw_now(title)); };
    const QMetaObject::Connection asked =
        QObject::connect(quick, &QQuickWindow::beforeRendering, quick, inner, Qt::DirectConnection);
    draw_now(title);
    QObject::disconnect(asked);
    return answers;
}
#endif

} // namespace kxui
