// The frame log's watch (`[ui] frame_log`): the times of each press and each frame step of our Quick windows, on the
// performance counter, handed to Rust as they come. It reads no key, position or text; src/bridges/frames.rs makes it.
#pragma once

#include <QtCore/QAbstractNativeEventFilter>
#include <QtCore/QByteArray>
#include <QtCore/QCoreApplication>
#include <QtCore/QEvent>
#include <QtCore/QHash>
#include <QtCore/QObject>
#include <QtGui/QGuiApplication>
#include <QtGui/QWindow>
#include <QtQuick/QQuickWindow>

#if defined(_WIN32)
#include <QtCore/qt_windows.h>
#endif

#include "rust/cxx.h"

#include <chrono>
#include <cstddef>
#include <cstdint>
#include <memory>
#include <utility>

namespace kxui {

// Now in µs: MSVC's steady clock reads the performance counter, as kx-gates and PresentMon do.
inline std::int64_t now_us()
{
    using namespace std::chrono;
    return duration_cast<microseconds>(steady_clock::now().time_since_epoch()).count();
}

// A press reached `window`: its message left the queue at `msg` (0 unseen), and it reached the window at `qt`.
using Pressed = rust::Fn<void(std::size_t window, std::int64_t msg, std::int64_t qt)>;
// Window `window` ended a frame: its update request, begin, sync, drawn scene, hand-over and end, 0 where none came.
using Framed = rust::Fn<void(std::size_t window, std::int64_t update, std::int64_t begin, std::int64_t sync,
                             std::int64_t rendered, std::int64_t swap, std::int64_t done)>;

// Times each press and frame of our Quick windows while it lives, on the GUI thread, which Qt's basic loop draws on.
class FrameWatch : public QObject, public QAbstractNativeEventFilter
{
public:
    FrameWatch(Pressed pressed, Framed framed) : pressed_(pressed), framed_(framed)
    {
        QCoreApplication::instance()->installEventFilter(this);
        QCoreApplication::instance()->installNativeEventFilter(this);
        for (QWindow *window : QGuiApplication::topLevelWindows())
            hook(qobject_cast<QQuickWindow *>(window));
    }

    ~FrameWatch() override
    {
        if (QCoreApplication *app = QCoreApplication::instance()) {
            app->removeNativeEventFilter(this);
            app->removeEventFilter(this);
        }
    }

    bool eventFilter(QObject *target, QEvent *event) override
    {
        const QEvent::Type type = event->type();
        if (type != QEvent::MouseButtonPress && type != QEvent::UpdateRequest && type != QEvent::PlatformSurface)
            return false;
        auto *quick = qobject_cast<QQuickWindow *>(target);
        if (!quick)
            return false;
        const std::int64_t at = now_us();
        hook(quick);
        if (type == QEvent::MouseButtonPress)
            pressed_(id(quick), std::exchange(msg_, 0), at);
        else if (type == QEvent::UpdateRequest)
            steps_[quick].update = at;
        return false;
    }

    // Notes when a button-down message leaves the queue, before Qt makes its press.
    bool nativeEventFilter(const QByteArray &type, void *message, qintptr *) override
    {
#if defined(_WIN32)
        if (type != "windows_generic_MSG")
            return false;
        const UINT msg = static_cast<const MSG *>(message)->message;
        if (msg == WM_LBUTTONDOWN || msg == WM_LBUTTONDBLCLK || msg == WM_POINTERDOWN)
            msg_ = now_us();
#else
        Q_UNUSED(type);
        Q_UNUSED(message);
#endif
        return false;
    }

private:
    // One frame's steps so far.
    struct Steps
    {
        std::int64_t update = 0;
        std::int64_t begin = 0;
        std::int64_t sync = 0;
        std::int64_t rendered = 0;
        std::int64_t swap = 0;
    };

    static std::size_t id(const QQuickWindow *window)
    {
        return reinterpret_cast<std::size_t>(window);
    }

    // Times window `window`'s frame steps from now on, once; the signals come on the GUI thread, as the frame runs.
    void hook(QQuickWindow *window)
    {
        if (!window || steps_.contains(window))
            return;
        steps_.insert(window, Steps{});
        const auto stamp = [this, window](std::int64_t Steps::*step) {
            return [this, window, step] { steps_[window].*step = now_us(); };
        };
        connect(window, &QQuickWindow::beforeFrameBegin, this, stamp(&Steps::begin), Qt::DirectConnection);
        connect(window, &QQuickWindow::beforeSynchronizing, this, stamp(&Steps::sync), Qt::DirectConnection);
        connect(window, &QQuickWindow::afterRendering, this, stamp(&Steps::rendered), Qt::DirectConnection);
        connect(window, &QQuickWindow::frameSwapped, this, stamp(&Steps::swap), Qt::DirectConnection);
        connect(window, &QQuickWindow::afterFrameEnd, this, [this, window] { ended(window); }, Qt::DirectConnection);
        connect(window, &QObject::destroyed, this, [this, window] { steps_.remove(window); });
    }

    void ended(QQuickWindow *window)
    {
        const std::int64_t done = now_us();
        const Steps s = std::exchange(steps_[window], Steps{});
        framed_(id(window), s.update, s.begin, s.sync, s.rendered, s.swap, done);
    }

    Pressed pressed_;
    Framed framed_;
    // When the last button-down message left the queue, until its press takes it.
    std::int64_t msg_ = 0;
    QHash<QQuickWindow *, Steps> steps_;
};

// Starts the frame log's watch on the GUI thread, once Qt's app object exists.
inline std::unique_ptr<FrameWatch> frame_watch_new(Pressed pressed, Framed framed)
{
    return std::make_unique<FrameWatch>(pressed, framed);
}

} // namespace kxui
