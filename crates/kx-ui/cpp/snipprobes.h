// Snip's test-only probes over snips.h, in debug builds only: a show the watch tells while the window is hidden,
// windows made by their title with each owner first, a shadowed pair's native order, and what the provider serves.
// snips.h includes this file last.
#pragma once

#include "snips.h"

#if defined(KX_UI_PROBES)

#include <QtCore/QLatin1String>
#include <QtGui/QPlatformSurfaceEvent>
#include <QtGui/QScreen>

#include <functional>
#include <iterator>
#include <limits>
#include <vector>

#if defined(_WIN32)
#include <QtCore/qt_windows.h>
#if defined(_MSC_VER)
// The probes read a native window's visibility and stacking, which user32 holds.
#pragma comment(lib, "user32")
#endif
#endif

namespace kxui {

// True while `window`'s native window is on screen: Windows' own flag, else Qt's exposure; debug builds only.
inline bool native_shown(QWindow *window)
{
#if defined(_WIN32)
    return IsWindowVisible(reinterpret_cast<HWND>(window->winId())) != FALSE;
#else
    return window->isExposed();
#endif
}

// True when `upper` lies in front of `lower` in Windows' own stack; elsewhere, as offscreen, no stack exists to read.
inline bool native_above(QWindow *upper, QWindow *lower)
{
#if defined(_WIN32)
    if (QGuiApplication::platformName() != QLatin1String("windows"))
        return true;
    const HWND target = reinterpret_cast<HWND>(upper->winId());
    for (HWND h = GetWindow(reinterpret_cast<HWND>(lower->winId()), GW_HWNDPREV); h; h = GetWindow(h, GW_HWNDPREV)) {
        if (h == target)
            return true;
    }
    return false;
#else
    Q_UNUSED(upper);
    Q_UNUSED(lower);
    return true;
#endif
}

// Shows a clear 1-pixel window titled `title`, which takes no focus or click, under a watch over `shown`, then lets it
// go; true when the watch told `shown` while the native window was made but hidden, still hidden once `shown` returned,
// and it showed after. Debug only.
inline bool show_watched(rust::Str title, rust::Fn<void(rust::Str)> shown)
{
    QWindow window;
    window.setFlags(Qt::Tool | Qt::FramelessWindowHint | Qt::WindowDoesNotAcceptFocus |
                    Qt::WindowTransparentForInput);
    window.setOpacity(0.0);
    window.resize(1, 1);
    window.setTitle(QString::fromUtf8(title.data(), qsizetype(title.size())));
    bool early = false;
    const ShowWatch watch([&window, &early, shown](QWindow *seen) {
        if (seen != &window)
            return;
        const bool made = seen->handle() && !native_shown(seen);
        tell_title(seen, shown);
        early = made && !native_shown(seen);
    });
    window.show();
    const bool after = native_shown(&window);
    window.hide();
    return early && after;
}

// Makes each window titled `title` that Qt has not made yet, beside a new hidden one of that title; true when only that
// one was made, and it has its native window and stays hidden. Debug only.
inline bool titled_made(rust::Str title)
{
    QWindow window;
    window.setFlags(Qt::Tool | Qt::FramelessWindowHint | Qt::WindowDoesNotAcceptFocus |
                    Qt::WindowTransparentForInput);
    window.setTitle(QString::fromUtf8(title.data(), qsizetype(title.size())));
    const bool fresh = !window.handle();
    const std::size_t made = make_titled(title);
    return fresh && made == 1 && window.handle() && !window.isVisible() && !native_shown(&window);
}

// Keeps the windows it watches in the order Qt made them, by each one's new native surface.
class MadeOrder : public QObject
{
public:
    std::vector<const QObject *> made;

protected:
    bool eventFilter(QObject *watched, QEvent *event) override
    {
        if (event->type() == QEvent::PlatformSurface &&
            static_cast<QPlatformSurfaceEvent *>(event)->surfaceEventType() == QPlatformSurfaceEvent::SurfaceCreated)
            made.push_back(watched);
        return QObject::eventFilter(watched, event);
    }
};

// Makes a shadow, the window it owns and that window's tip by their one title `title`, listed newest first as Qt lists
// QML's; true when each was made after its owner, all three stay hidden, and on Windows each lies in front of its
// owner. Debug only.
inline bool owners_made_first(rust::Str title)
{
    MadeOrder seen;
    QWindow shadow;
    QWindow window;
    QWindow tip;
    window.setTransientParent(&shadow);
    tip.setTransientParent(&window);
    const QString name = QString::fromUtf8(title.data(), qsizetype(title.size()));
    for (QWindow *w : {&shadow, &window, &tip}) {
        w->setFlags(Qt::Tool | Qt::FramelessWindowHint | Qt::WindowDoesNotAcceptFocus | Qt::WindowTransparentForInput);
        w->setTitle(name);
        w->installEventFilter(&seen);
    }
    const std::size_t made = make_titled(title);
    const bool ordered = made == 3 && seen.made == std::vector<const QObject *>{&shadow, &window, &tip};
    const bool hidden = !native_shown(&shadow) && !native_shown(&window) && !native_shown(&tip);
    return ordered && hidden && native_above(&window, &shadow) && native_above(&tip, &window);
}

// True when each window titled `title` and the window that owns it are both made and hidden; false when none has that
// title. It reads the app's own windows, as the editor's `make` leaves them. Debug only.
inline bool titled_owner_made(rust::Str title)
{
    const QString name = QString::fromUtf8(title.data(), qsizetype(title.size()));
    bool found = false;
    for (QWindow *window : QGuiApplication::allWindows()) {
        if (window->title() != name)
            continue;
        QWindow *owner = window->transientParent();
        if (!window->handle() || !owner || !owner->handle() || native_shown(window) || native_shown(owner))
            return false;
        found = true;
    }
    return found;
}

// True when `owner` owns `window` in Windows' own tree; elsewhere, as offscreen, no owner exists to read.
inline bool native_owned(QWindow *window, QWindow *owner)
{
#if defined(_WIN32)
    if (QGuiApplication::platformName() != QLatin1String("windows"))
        return true;
    return GetWindow(reinterpret_cast<HWND>(window->winId()), GW_OWNER) == reinterpret_cast<HWND>(owner->winId());
#else
    Q_UNUSED(window);
    Q_UNUSED(owner);
    return true;
#endif
}

// The probe pair's side and place in Qt units, clear and away from the screen's corner.
constexpr int glow_probe_side = 40;
// The windows a make ahead by the window's title makes: its shadow first, through its owner, then the window.
constexpr std::size_t glow_pair_made = 2;

// A clear shadow titled `shade`, the window titled `title` it owns and that window's tip, flagged as Glow.qml and
// Tip.qml flag them; the window shows in its owner's show, before Qt makes the owner, as Glow's binding does.
struct GlowPair
{
    QWindow shadow;
    QWindow window;
    QWindow tip;

    GlowPair(const QString &title, const QString &shade, const QRect &box)
    {
        window.setTransientParent(&shadow);
        tip.setTransientParent(&window);
        const Qt::WindowFlags owned =
            Qt::Tool | Qt::FramelessWindowHint | Qt::WindowStaysOnTopHint | Qt::WindowDoesNotAcceptFocus;
        shadow.setFlags(owned | Qt::WindowTransparentForInput);
        window.setFlags(owned);
        tip.setFlags(Qt::ToolTip | owned | Qt::WindowTransparentForInput);
        for (QWindow *w : {&shadow, &window, &tip}) {
            w->setOpacity(0.0);
            w->setGeometry(box);
        }
        shadow.setTitle(shade);
        window.setTitle(title);
        QObject::connect(&shadow, &QWindow::visibleChanged, &window, &QWindow::setVisible);
    }

    // Stays on top or leaves it, the shadow first, as Glow.owns orders a system dialog's drop and return.
    void top(bool on)
    {
        shadow.setFlag(Qt::WindowStaysOnTopHint, on);
        window.setFlag(Qt::WindowStaysOnTopHint, on);
    }

    // Both take box `r`, as a maximise or a restore does.
    void place(const QRect &r)
    {
        shadow.setGeometry(r);
        window.setGeometry(r);
    }

    // The window lies over its shadow and is owned by it, under its tip while that shows.
    bool kept()
    {
        const bool tipped = !tip.isVisible() || native_above(&tip, &window);
        return native_above(&window, &shadow) && native_owned(&window, &shadow) && tipped;
    }
};

// Shows a GlowPair, its shadow titled `shade`, made ahead by `title` when `ahead`, then drops it from the top and back
// as a system dialog does, maximises and restores it, and shows and hides its tip. Returns 0 when the window stays over
// its shadow and owned by it, the most a byte holds when a make ahead missed the pair, else the first step that broke,
// from 1. Debug only.
inline std::uint8_t glow_pair_broken(rust::Str title, rust::Str shade, bool ahead)
{
    const QRect box(glow_probe_side, glow_probe_side, glow_probe_side, glow_probe_side);
    const QScreen *screen = QGuiApplication::primaryScreen();
    const QRect big = screen ? screen->availableGeometry() : box.adjusted(0, 0, glow_probe_side, glow_probe_side);
    const auto text = [](rust::Str s) { return QString::fromUtf8(s.data(), qsizetype(s.size())); };
    GlowPair pair(text(title), text(shade), box);
    if (ahead && make_titled(title) != glow_pair_made)
        return std::numeric_limits<std::uint8_t>::max();
    const std::function<void()> steps[] = {
        [&pair] { pair.shadow.show(); }, [&pair] { pair.top(false); },     [&pair] { pair.top(true); },
        [&pair, big] { pair.place(big); }, [&pair, box] { pair.place(box); }, [&pair] { pair.tip.show(); },
        [&pair] { pair.tip.hide(); }};
    std::uint8_t broken = 0;
    for (std::size_t i = 0; i < std::size(steps) && broken == 0; ++i) {
        steps[i]();
        if (!pair.kept())
            broken = std::uint8_t(i + 1);
    }
    pair.shadow.hide();
    return broken;
}

// What a provider over `fetch` serves for `id`, as RGBA8 bytes as its texture loads them, with its width, and whether
// it draws the shared pixels themselves; debug builds only.
template <typename Pixels>
rust::Vec<std::uint8_t> snip_served(rust::Str id, SnipFetch<Pixels> fetch, std::uint32_t &width, bool &shared)
{
    const SnipPictures<Pixels> pictures(fetch);
    const std::unique_ptr<SnipTexture> served(pictures.serve(QString::fromUtf8(id.data(), qsizetype(id.size()))));
    const rust::Box<Pixels> again = rust::Box<Pixels>::from_raw(fetch(id));
    const rust::Slice<const std::uint8_t> bytes = again->bgrx();
    shared = bytes.size() > 0 && served->image().constBits() == bytes.data();
    const QImage shown = served->loaded().convertToFormat(QImage::Format_RGBA8888);
    width = std::uint32_t(shown.width());
    rust::Vec<std::uint8_t> out;
    const std::size_t row = std::size_t(shown.width()) * snip_pixel_bytes;
    for (int y = 0; y < shown.height(); ++y) {
        const auto *line = shown.constScanLine(y);
        for (std::size_t x = 0; x < row; ++x)
            out.push_back(line[x]);
    }
    return out;
}

} // namespace kxui

#endif
