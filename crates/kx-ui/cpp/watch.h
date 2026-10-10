// What makes the windows again besides a lost device (ADR-0021); src/bridges/device.rs calls these on the Qt GUI thread.
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QEvent>
#include <QtCore/QMetaObject>
#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtCore/QThread>
#include <QtCore/QTimer>
#include <QtGui/QGuiApplication>
#include <QtGui/QPlatformSurfaceEvent>
#include <QtGui/QWindow>
#include <QtQuick/QQuickWindow>

#include "kx-ui-build.h"

#include <algorithm>
#include <atomic>
#include <cstdint>
#include <memory>

namespace kxui {

// Ends Qt's loop once, queued, so it also comes when the caller runs outside the loop.
inline void end_loop()
{
    const auto end = [] { QCoreApplication::exit(0); };
    QMetaObject::invokeMethod(QCoreApplication::instance(), end, Qt::QueuedConnection);
}

// Watches each Qt Quick window for a scene graph that could not start, which Qt would otherwise end the app for, and
// ends Qt's loop instead; it also times the next try at a new device, so the loop keeps running meanwhile.
class Watch : public QObject
{
public:
    Watch()
    {
        QCoreApplication::instance()->installEventFilter(this);
        timer.setSingleShot(true);
        connect(&timer, &QTimer::timeout, this, [this] {
            due.store(true);
            QCoreApplication::exit(0);
        });
    }

    ~Watch() override
    {
        if (auto *app = QCoreApplication::instance())
            app->removeEventFilter(this);
    }

    // True once a window's scene graph could not start since the last call, which clears it.
    bool take_failed() { return failed.exchange(false); }

    // Ends Qt's loop after `ms`, for the next try; `ready` turns true then.
    void retry(std::uint64_t ms)
    {
        due.store(false);
        timer.start(int(std::min<std::uint64_t>(ms, std::uint64_t(INT32_MAX))));
    }

    // True once the timer of the last `retry` ran out.
    bool ready() const { return due.load(); }

#if defined(KX_UI_PROBES)
    // Has the first Qt Quick window report a scene graph that could not start, as a failed device would; debug builds
    // only.
    void fail()
    {
        for (QWindow *window : QGuiApplication::allWindows()) {
            if (auto *quick = qobject_cast<QQuickWindow *>(window)) {
                Q_EMIT quick->sceneGraphError(QQuickWindow::ContextNotAvailable, QString());
                return;
            }
        }
    }
#endif

protected:
    bool eventFilter(QObject *watched, QEvent *event) override
    {
        if (event->type() == QEvent::PlatformSurface) {
            auto *surface = static_cast<QPlatformSurfaceEvent *>(event);
            auto *quick = qobject_cast<QQuickWindow *>(watched);
            if (quick && surface->surfaceEventType() == QPlatformSurfaceEvent::SurfaceCreated) {
                const auto how = Qt::ConnectionType(Qt::DirectConnection | Qt::UniqueConnection);
                connect(quick, &QQuickWindow::sceneGraphError, this, &Watch::errored, how);
            }
        }
        return QObject::eventFilter(watched, event);
    }

private:
    // A connected handler keeps Qt from ending the app; the first error since the last take ends Qt's loop.
    void errored(QQuickWindow::SceneGraphError, const QString &)
    {
        if (!failed.exchange(true))
            end_loop();
    }

    QTimer timer;
    std::atomic<bool> failed{false};
    std::atomic<bool> due{false};
};

// True on the thread of Qt's app object, once it exists.
inline bool on_app_thread()
{
    const QCoreApplication *app = QCoreApplication::instance();
    return app && QThread::currentThread() == app->thread();
}

// A watch over every Qt Quick window; null before Qt's app object exists or off its thread.
inline std::unique_ptr<Watch> watch_new()
{
    return on_app_thread() ? std::make_unique<Watch>() : nullptr;
}

} // namespace kxui
