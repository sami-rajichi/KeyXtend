// One graphics device for every Qt Quick window (ADR-0021); src/bridges/device.rs calls these on the Qt GUI thread.
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QEvent>
#include <QtCore/QList>
#include <QtCore/QObject>
#include <QtCore/QSet>
#include <QtCore/QThread>
#include <QtGui/QGuiApplication>
#include <QtGui/QPlatformSurfaceEvent>
#include <QtGui/QWindow>
#include <QtQuick/QQuickGraphicsDevice>
#include <QtQuick/QQuickWindow>
#include <rhi/qrhi.h>

#include <atomic>
#include <cstddef>
#include <memory>

namespace kxui {

// The shared QRhi. An app-wide filter has each Qt Quick window adopt it as its native window is made, before its first
// frame.
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

    // True when the device is gone.
    bool lost() const { return shared->isDeviceLost(); }

    // How many Qt Quick windows draw on the device.
    std::size_t windows() const
    {
        std::size_t count = 0;
        for (QWindow *window : QGuiApplication::allWindows()) {
            auto *quick = qobject_cast<QQuickWindow *>(window);
            count += quick && quick->rhi() == shared.get() ? 1 : 0;
        }
        return count;
    }

    // True while every frame on the device has ended on the GUI thread, as the basic render loop draws.
    bool gui_only() const { return !offThread.load(); }

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

    // After each frame, also a failed one: notes a frame off the GUI thread.
    void framed()
    {
        if (QThread::currentThread() != thread())
            offThread = true;
    }

    std::unique_ptr<QRhi> shared;
    std::atomic<bool> offThread{false};
};

// A device on the adapter Qt's variables pin, or null when Direct3D 11 makes none; other systems share none yet.
inline std::unique_ptr<Device> device_new()
{
#if defined(Q_OS_WIN)
    QRhiD3D11InitParams params;
    QRhi *rhi = QRhi::create(QRhi::D3D11, &params);
    return rhi ? std::make_unique<Device>(rhi) : nullptr;
#else
    return nullptr;
#endif
}

// The Qt Quick windows that have drawn, on any device.
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

// Deletes the objects that wait for a later delete, as windows a gone engine left, so none outlives its device.
inline void flush_deletes()
{
    QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
}

} // namespace kxui
