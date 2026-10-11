// One graphics device for every Qt Quick window (ADR-0021); src/bridges/device.rs calls these on the Qt GUI thread.
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QEvent>
#include <QtCore/QList>
#include <QtCore/QMetaMethod>
#include <QtCore/QMetaObject>
#include <QtCore/QObject>
#include <QtCore/QSet>
#include <QtCore/QThread>
#include <QtGui/QGuiApplication>
#include <QtGui/QPlatformSurfaceEvent>
#include <QtGui/QWindow>
#include <QtQuick/QQuickGraphicsDevice>
#include <QtQuick/QQuickWindow>
#include <rhi/qrhi.h>

#include "kx-ui-build.h"
#include "watch.h"

#include <atomic>
#include <cstddef>
#include <memory>

namespace kxui {

// The shared QRhi. An app-wide filter has each Qt Quick window adopt it as its native window is made, before its first
// frame; the first frame that finds it lost ends Qt's loop, so the app can make the windows again.
class Device : public QObject
{
public:
    explicit Device(QRhi *rhi) : shared(rhi) { QCoreApplication::instance()->installEventFilter(this); }

    // The app drops every window first, so no window draws on the QRhi this deletes.
    ~Device() override
    {
        if (auto *app = QCoreApplication::instance())
            app->removeEventFilter(this);
    }

    // True once a frame found the device gone or ran off the GUI thread.
    bool lost() const { return found.load(); }

    // True while every frame on the device has ended on the GUI thread, as the basic render loop draws.
    bool gui_only() const { return !offThread.load(); }

#if defined(KX_UI_PROBES)
    // How many Qt Quick windows draw on the device; debug builds only.
    std::size_t windows() const
    {
        std::size_t count = 0;
        for (QWindow *window : QGuiApplication::allWindows()) {
            auto *quick = qobject_cast<QQuickWindow *>(window);
            count += quick && quick->rhi() == shared.get() ? 1 : 0;
        }
        return count;
    }

    // Marks the device gone and asks each window for a frame, as a driver reset would; debug builds only.
    void lose()
    {
        forced.store(true);
        redraw();
    }

    // Has the next frame count as drawn off the GUI thread, as Qt's threaded loop would draw; debug builds only.
    void stray()
    {
        offThread.store(true);
        redraw();
    }

    // Asks each Qt Quick window for a frame.
    static void redraw()
    {
        for (QWindow *window : QGuiApplication::allWindows()) {
            if (auto *quick = qobject_cast<QQuickWindow *>(window))
                quick->update();
        }
    }
#endif

protected:
    bool eventFilter(QObject *watched, QEvent *event) override
    {
        if (event->type() == QEvent::PlatformSurface) {
            auto *surface = static_cast<QPlatformSurfaceEvent *>(event);
            auto *quick = qobject_cast<QQuickWindow *>(watched);
            if (quick && surface->surfaceEventType() == QPlatformSurfaceEvent::SurfaceCreated)
                adopt(quick);
        }
        return QObject::eventFilter(watched, event);
    }

private:
    void adopt(QQuickWindow *window)
    {
        window->setGraphicsDevice(QQuickGraphicsDevice::fromRhi(shared.get()));
        const auto how = Qt::ConnectionType(Qt::DirectConnection | Qt::UniqueConnection);
        connect(window, &QQuickWindow::afterFrameEnd, this, &Device::framed, how);
    }

    // After each frame, also a failed one: ends Qt's loop once the device is gone or a frame ran off the GUI thread,
    // which one device cannot take. The end is queued, so it also comes when the frame ran outside the loop.
    void framed()
    {
        if (QThread::currentThread() != thread())
            offThread = true;
        // Each exit also emits aboutToQuit, so nothing of ours may hang work on that signal.
        if ((offThread.load() || gone()) && !found.exchange(true))
            end_loop();
    }

    // True when the device is gone, or a debug build marked it so.
    bool gone() const
    {
#if defined(KX_UI_PROBES)
        if (forced.load())
            return true;
#endif
        return shared->isDeviceLost();
    }

    std::unique_ptr<QRhi> shared;
    std::atomic<bool> offThread{false};
    std::atomic<bool> found{false};
#if defined(KX_UI_PROBES)
    std::atomic<bool> forced{false};
#endif
};

// A device on the adapter Qt's variables pin; null before Qt's app object exists, off its thread, or when Direct3D 11
// makes none. Other systems share none yet.
inline std::unique_ptr<Device> device_new()
{
#if defined(Q_OS_WIN)
    if (!on_app_thread())
        return nullptr;
    QRhiD3D11InitParams params;
    QRhi *rhi = QRhi::create(QRhi::D3D11, &params);
    return rhi ? std::make_unique<Device>(rhi) : nullptr;
#else
    return nullptr;
#endif
}

#if defined(KX_UI_PROBES)
// The Qt Quick windows that have drawn, on any device; debug builds only.
inline QList<QQuickWindow *> drawn()
{
    QList<QQuickWindow *> found;
    for (QWindow *window : QGuiApplication::allWindows()) {
        auto *quick = qobject_cast<QQuickWindow *>(window);
        if (quick && quick->rhi())
            found.append(quick);
    }
    return found;
}

// How many Qt Quick windows draw.
inline std::size_t drawing_windows()
{
    return std::size_t(drawn().size());
}

// How many devices those windows draw on.
inline std::size_t drawing_devices()
{
    QSet<QRhi *> devices;
    for (QQuickWindow *window : drawn())
        devices.insert(window->rhi());
    return std::size_t(devices.size());
}

// Reads QObject's protected isSignalConnected through a member pointer, which a derived class may form.
struct Peek : QObject
{
    static bool heard(const QObject *object, const QMetaMethod &signal)
    {
        const auto connected = &Peek::isSignalConnected;
        return (object->*connected)(signal);
    }
};

// How many drawn Qt Quick windows have sceneGraphError connected, as Qt checks before it ends the app over one.
inline std::size_t wired_windows()
{
    const QMetaMethod error = QMetaMethod::fromSignal(&QQuickWindow::sceneGraphError);
    std::size_t count = 0;
    for (QQuickWindow *window : drawn())
        count += Peek::heard(window, error) ? 1 : 0;
    return count;
}
#endif

// Deletes the objects that wait for a later delete, as windows a gone engine left, so none outlives its device.
inline void flush_deletes()
{
    QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
}

} // namespace kxui
