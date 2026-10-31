// The clipboard panel's picture provider (ClipPictures): it serves a row's thumbnail and the preview image of the kept
// view, copied from Rust for each request, each as its own texture. src/bridges/clip.rs registers it.
#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QSize>
#include <QtCore/QString>
#include <QtGui/QImage>
#include <QtQml/QQmlApplicationEngine>
#include <QtQuick/QQuickImageProvider>
#include <QtQuick/QQuickWindow>
#include <QtQuick/QSGTexture>

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <memory>
#include <utility>

#include "kx-ui-build.h"
#include "rust/cxx.h"
#include "tint.h"

namespace kxui {

// What copies picture `id`'s RGBA8 bytes, row by row from the top, and sets its width and height; empty for none.
using Fetch = rust::Fn<rust::Vec<std::uint8_t>(rust::Str, std::uint32_t &, std::uint32_t &)>;
// Bytes in one RGBA8 pixel.
constexpr std::size_t clip_pixel_bytes = 4;

// A picture's own texture, outside Qt's shared atlas, so a dropped view's pictures leave with their images.
class ClipTexture : public QQuickTextureFactory
{
public:
    explicit ClipTexture(QImage art) : art(std::move(art)) {}

    QSGTexture *createTexture(QQuickWindow *window) const override
    {
        return window->createTextureFromImage(art);
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

private:
    const QImage art;
};

// Serves `image://<provider>/<round>/<id>` and `image://<provider>/<round>/preview` from what `fetch` copies. An old
// round, a missing picture or bytes that do not fill the picture give one clear pixel, so QML shows the kind's icon.
class ClipPictures : public QQuickImageProvider
{
public:
    explicit ClipPictures(Fetch fetch) : QQuickImageProvider(QQuickImageProvider::Texture), fetch(fetch) {}

    QQuickTextureFactory *requestTexture(const QString &id, QSize *size, const QSize &) override
    {
        QImage art = serve(id);
        if (size)
            *size = art.size();
        return new ClipTexture(std::move(art));
    }

    // Picture `id` at its own size, or one clear pixel.
    QImage serve(const QString &id) const
    {
        const QByteArray utf8 = id.toUtf8();
        std::uint32_t width = 0;
        std::uint32_t height = 0;
        const rust::Vec<std::uint8_t> rgba = fetch(rust::Str(utf8.constData(), std::size_t(utf8.size())), width, height);
        const std::size_t row = std::size_t(width) * clip_pixel_bytes;
        if (width == 0 || height == 0 || rgba.size() != row * height)
            return clear(QSize());
        QImage art(int(width), int(height), QImage::Format_RGBA8888);
        if (art.isNull())
            return clear(QSize());
        for (std::uint32_t y = 0; y < height; ++y)
            std::memcpy(art.scanLine(int(y)), rgba.data() + std::size_t(y) * row, row);
        return art;
    }

private:
    const Fetch fetch;
};

// Registers a provider over `fetch` on `engine` as `name`; the engine owns it.
inline void serve_clips(QQmlApplicationEngine &engine, rust::Str name, Fetch fetch)
{
    engine.addImageProvider(QString::fromUtf8(name.data(), qsizetype(name.size())), new ClipPictures(fetch));
}

#if defined(KX_UI_PROBES)
// What a provider over `fetch` serves for `id`, as RGBA8 bytes row by row, with its width; debug builds only.
inline rust::Vec<std::uint8_t> clip_served(rust::Str id, Fetch fetch, std::uint32_t &width)
{
    const std::unique_ptr<ClipPictures> pictures(new ClipPictures(fetch));
    const QImage art = pictures->serve(QString::fromUtf8(id.data(), qsizetype(id.size())))
                           .convertToFormat(QImage::Format_RGBA8888);
    width = std::uint32_t(art.width());
    rust::Vec<std::uint8_t> out;
    const std::size_t row = std::size_t(art.width()) * clip_pixel_bytes;
    for (int y = 0; y < art.height(); ++y) {
        const auto *line = art.constScanLine(y);
        for (std::size_t x = 0; x < row; ++x)
            out.push_back(line[x]);
    }
    return out;
}
#endif

} // namespace kxui
