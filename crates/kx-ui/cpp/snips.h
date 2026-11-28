// Snip's picture provider (SnipPictures) serves each frozen screen and the editor's picture over the hub's own shared
// pixels, never a copy, as its own texture. Beside it are the window maker, the show watch (ShowWatch) and a grab's one
// copy, which src/bridges/snips.rs registers, with test probes in snipprobes.h for debug builds.
#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QCoreApplication>
#include <QtCore/QEvent>
#include <QtCore/QList>
#include <QtCore/QObject>
#include <QtCore/QSize>
#include <QtCore/QString>
#include <QtGui/QGuiApplication>
#include <QtGui/QImage>
#include <QtGui/QWindow>
#include <QtQml/QQmlApplicationEngine>
#include <QtQuick/QQuickImageProvider>
#include <QtQuick/QQuickWindow>
#include <QtQuick/QSGTexture>

#include <climits>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <functional>
#include <memory>
#include <utility>

#include "kx-ui-build.h"
#include "rust/cxx.h"
#include "tint.h"

namespace kxui {

// What hands over picture `id`'s shared pixels, `BGRX_BYTES` each, rows from the top, none at size 0; C++ owns them.
template <typename Pixels>
using SnipFetch = rust::Fn<Pixels *(rust::Str)>;
// Bytes in one BGRX pixel; it mirrors `BGRX_BYTES` in kx-module-api's freeze port, which checks every `Shot`.
constexpr std::size_t snip_pixel_bytes = 4;

// Lets the pixels a picture held go once Qt drops its last copy of the image.
template <typename Pixels>
void drop_snip_pixels(void *held) noexcept
{
    const rust::Box<Pixels> gone = rust::Box<Pixels>::from_raw(static_cast<Pixels *>(held));
    (void)gone;
}

// A picture's own texture, outside Qt's shared atlas. A frozen screen's unused byte is made opaque as it loads.
class SnipTexture : public QQuickTextureFactory
{
public:
    SnipTexture(QImage art, bool frozen) : art(std::move(art)), frozen(frozen) {}

    QSGTexture *createTexture(QQuickWindow *window) const override
    {
        QQuickWindow::CreateTextureOptions options;
        options.setFlag(QQuickWindow::TextureIsOpaque, frozen);
        return window->createTextureFromImage(loaded(), options);
    }
    QSize textureSize() const override
    {
        return art.size();
    }
    int textureByteCount() const override
    {
        return int(art.sizeInBytes());
    }
    QImage image() const override
    {
        return art;
    }

    // The picture as its texture loads it: a frozen one made opaque, a clear pixel as it is.
    QImage loaded() const
    {
        return frozen ? art.convertToFormat(QImage::Format_RGBX8888) : art;
    }

private:
    const QImage art;
    const bool frozen;
};

// Serves `image://<provider>/<generation>/<place>` over the pixels `fetch` hands over. An old generation, a missing
// picture or pixels that do not fill it give one clear pixel.
template <typename Pixels>
class SnipPictures : public QQuickImageProvider
{
public:
    explicit SnipPictures(SnipFetch<Pixels> fetch) : QQuickImageProvider(QQuickImageProvider::Texture), fetch(fetch) {}

    QQuickTextureFactory *requestTexture(const QString &id, QSize *size, const QSize &) override
    {
        SnipTexture *served = serve(id);
        if (size)
            *size = served->textureSize();
        return served;
    }

    // Picture `id` over its shared pixels, which it holds until Qt drops the image, or one clear pixel.
    SnipTexture *serve(const QString &id) const
    {
        const QByteArray utf8 = id.toUtf8();
        const rust::Str name(utf8.constData(), std::size_t(utf8.size()));
        rust::Box<Pixels> pixels = rust::Box<Pixels>::from_raw(fetch(name));
        const std::uint32_t width = pixels->width();
        const std::uint32_t height = pixels->height();
        const rust::Slice<const std::uint8_t> bgrx = pixels->bgrx();
        const std::size_t row = std::size_t(width) * snip_pixel_bytes;
        const bool fits = width > 0 && height > 0 && width <= INT_MAX && height <= INT_MAX;
        if (!fits || bgrx.size() != row * height)
            return new SnipTexture(clear(QSize()), false);
        Pixels *held = pixels.into_raw();
        QImage art(bgrx.data(), int(width), int(height), qsizetype(row), QImage::Format_RGB32,
                   &drop_snip_pixels<Pixels>, held);
        if (art.isNull()) {
            drop_snip_pixels<Pixels>(held);
            return new SnipTexture(clear(QSize()), false);
        }
        return new SnipTexture(std::move(art), true);
    }

private:
    const SnipFetch<Pixels> fetch;
};

// Registers a provider over `fetch` on `engine` as `name`; the engine owns it.
template <typename Pixels>
void serve_snips(QQmlApplicationEngine &engine, rust::Str name, SnipFetch<Pixels> fetch)
{
    engine.addImageProvider(QString::fromUtf8(name.data(), qsizetype(name.size())), new SnipPictures<Pixels>(fetch));
}

// Makes `window` after the window that owns it, as a shadow owns its window, so Windows puts it in front of its owner
// from the start; how many windows it made.
inline std::size_t make_owned(QWindow *window)
{
    std::size_t made = 0;
    if (QWindow *owner = window->transientParent(); owner && !owner->handle())
        made += make_owned(owner);
    if (!window->handle()) {
        window->create();
        ++made;
    }
    return made;
}

// Makes each of our windows titled `title` that Qt has not made yet, each after its owner, as Qt lists the newest
// first; its shield can then come before it shows. Returns how many it made.
inline std::size_t make_titled(rust::Str title)
{
    const QString name = QString::fromUtf8(title.data(), qsizetype(title.size()));
    std::size_t made = 0;
    for (QWindow *window : QGuiApplication::allWindows()) {
        if (window->title() == name)
            made += make_owned(window);
    }
    return made;
}

// How many of our windows titled `title` Qt shows now, those it freed but not yet deleted included.
inline std::size_t shown_titled(rust::Str title)
{
    const QString name = QString::fromUtf8(title.data(), qsizetype(title.size()));
    std::size_t shown = 0;
    for (const QWindow *window : QGuiApplication::allWindows())
        shown += window->isVisible() && window->title() == name ? 1 : 0;
    return shown;
}

// Hands each window to `shown` as Qt is about to show it: Qt sends the show after it made the window and before the
// screen shows it, so a shield set then is in no capture.
class ShowWatch : public QObject
{
public:
    explicit ShowWatch(std::function<void(QWindow *)> shown) : shown(std::move(shown))
    {
        if (auto *app = QCoreApplication::instance())
            app->installEventFilter(this);
    }

    ~ShowWatch() override
    {
        if (auto *app = QCoreApplication::instance())
            app->removeEventFilter(this);
    }

protected:
    bool eventFilter(QObject *watched, QEvent *event) override
    {
        if (event->type() == QEvent::Show && watched->isWindowType())
            shown(static_cast<QWindow *>(watched));
        return QObject::eventFilter(watched, event);
    }

private:
    const std::function<void(QWindow *)> shown;
};

// Hands `window`'s title to `shown`.
inline void tell_title(const QWindow *window, rust::Fn<void(rust::Str)> shown)
{
    const QByteArray title = window->title().toUtf8();
    shown(rust::Str(title.constData(), std::size_t(title.size())));
}

// A watch that hands `shown` each window's title as Qt is about to show it; null before Qt's app object exists.
inline std::unique_ptr<ShowWatch> show_watch_new(rust::Fn<void(rust::Str)> shown)
{
    if (!QCoreApplication::instance())
        return nullptr;
    return std::make_unique<ShowWatch>([shown](QWindow *window) { tell_title(window, shown); });
}

// Copies `picture` once into `bgrx`, row by row from the top; false when it is not that size.
inline bool snip_fill(const QImage &picture, rust::Slice<std::uint8_t> bgrx)
{
    const QImage::Format kept = picture.format();
    const bool bgra = kept == QImage::Format_RGB32 || kept == QImage::Format_ARGB32_Premultiplied;
    const QImage art = bgra ? picture : picture.convertToFormat(QImage::Format_RGB32);
    const std::size_t row = std::size_t(art.width()) * snip_pixel_bytes;
    if (art.isNull() || bgrx.size() != row * std::size_t(art.height()))
        return false;
    for (int y = 0; y < art.height(); ++y)
        std::memcpy(bgrx.data() + std::size_t(y) * row, art.constScanLine(y), row);
    return true;
}

} // namespace kxui

#include "snipprobes.h"
