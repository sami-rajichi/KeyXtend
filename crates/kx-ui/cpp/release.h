// After a look change, each Qt Quick window drops what Qt cached for the last look, then the freed memory goes back to
// the system (ADR-0023); src/bridges/device.rs calls these on the Qt GUI thread.
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QMetaObject>
#include <QtCore/QObject>
#include <QtGui/QGuiApplication>
#include <QtGui/QWindow>
#include <QtQuick/QQuickWindow>

#include "kx-ui-build.h"
#include "rust/cxx.h"

#include <atomic>
#include <cstddef>
#include <optional>

namespace kxui {

#if defined(KX_UI_PROBES)
// How many windows dropped Qt's cached graphics after a frame so far; debug builds only.
inline std::atomic<std::size_t> releases{0};

inline std::size_t released_windows()
{
    return releases.load();
}
#endif

// The property that marks a window waiting for the frame after which it drops Qt's cached graphics.
inline constexpr char RELEASING[] = "kxReleasing";

// What runs once no window waits to drop its cached graphics after a look change: the freed memory goes back.
using Settled = rust::Fn<void()>;

// Drops its window's cached graphics once the window's next frame is out of the way, or once it hid if it hides first,
// then goes. The window owns it, so a window that goes first takes it along. All of this runs on the GUI thread.
class Release : public QObject
{
public:
    explicit Release(QQuickWindow *window) : QObject(window), window(window)
    {
        ++owed;
        window->setProperty(RELEASING, true);
        connect(window, &QQuickWindow::frameSwapped, this, &Release::later, Qt::SingleShotConnection);
        connect(window, &QWindow::visibleChanged, this, [this](bool shown) {
            if (!shown)
                later();
        });
        window->update();
    }

    ~Release() override
    {
        if (!done)
            --owed;
    }

    // Runs the last look change's settled call, once no window waits to drop its cached graphics.
    static void settle()
    {
        if (owed > 0 || !settled)
            return;
        const Settled call = *settled;
        settled.reset();
        call();
    }

    // The call that waits for the windows, kept until they all dropped their cached graphics.
    static inline std::optional<Settled> settled;

private:
    void later() { QMetaObject::invokeMethod(this, &Release::now, Qt::QueuedConnection); }

    void now()
    {
        if (done)
            return;
        done = true;
        --owed;
        window->setProperty(RELEASING, false);
        window->releaseResources();
#if defined(KX_UI_PROBES)
        ++releases;
#endif
        deleteLater();
        settle();
    }

    // How many windows wait to drop their cached graphics.
    static inline int owed = 0;

    QQuickWindow *window;
    bool done = false;
};

// Has each shown Qt Quick window drop Qt's cached graphics after its next frame: pipelines, shaders and upload data,
// which look changes would otherwise pile up. A window asked already waits for that frame. Once none waits, `then` runs
// once, the call of the last look change only.
inline void release_after_frame(Settled then)
{
    Release::settled = then;
    for (QWindow *window : QGuiApplication::topLevelWindows()) {
        auto *quick = qobject_cast<QQuickWindow *>(window);
        if (quick && quick->isVisible() && !quick->property(RELEASING).toBool())
            new Release(quick);
    }
    QMetaObject::invokeMethod(QCoreApplication::instance(), &Release::settle, Qt::QueuedConnection);
}

} // namespace kxui
