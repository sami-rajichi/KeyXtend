// The icon provider (TintIcons): it draws a bundled icon file at the asked size with its ink filled in a tint, so no
// icon needs a shader effect of its own. src/bridges/tint.rs registers it; the Quick tests' plugin shares it.
#pragma once

#include <QtCore/QSize>
#include <QtCore/QString>
#include <QtGui/QColor>
#include <QtGui/QImage>
#include <QtGui/QImageReader>
#include <QtGui/QPainter>
#include <QtQuick/QQuickImageProvider>
#include <QtQuick/QQuickWindow>
#include <QtQuick/QSGTexture>

#include <algorithm>
#include <atomic>
#include <cstdint>
#include <functional>
#include <utility>

// The build kind, which sets the class layout below, comes before it in every kx-ui file; the Quick tests' plugin has
// no such header and builds without the probes.
#if __has_include("kx-ui-build.h")
#include "kx-ui-build.h"
#endif

namespace kxui {

// Hears the id of each icon that was refused or did not read.
using Refused = std::function<void(const QString &)>;
// The first code point past ASCII.
constexpr char16_t ascii_end = 0x80;
// The size served when Qt asks for an empty one.
constexpr QSize one_pixel(1, 1);

// True when `name` is a plain file name of type `ext`: ASCII letters, digits, `-`, `_` and dots, and no leading dot,
// so it never leaves the icons folder.
inline bool bare_name(const QString &name, const QString &ext)
{
    const auto plain = [](QChar c) {
        return c.unicode() < ascii_end && (c.isLetterOrNumber() || c == u'-' || c == u'_' || c == u'.');
    };
    const QString end = u'.' + ext;
    const bool typed = name.size() > end.size() && name.endsWith(end);
    return typed && !name.startsWith(u'.') && std::all_of(name.cbegin(), name.cend(), plain);
}

// A clear image of `size`, or of one pixel when `size` is empty, as Qt draws nothing from a null image.
inline QImage clear(const QSize &size)
{
    QImage art(size.isEmpty() ? one_pixel : size, QImage::Format_ARGB32_Premultiplied);
    art.fill(Qt::transparent);
    return art;
}

// Icon `id`, `<rrggbb>/<name>.<ext>`, from folder `root`, drawn at `size` (its own when invalid, one clear pixel when
// empty) with its ink filled in that tint and its alpha kept; a null image when the id is refused or the file does not
// read.
inline QImage tinted(const QString &root, const QString &ext, const QString &id, const QSize &size)
{
    const qsizetype cut = id.indexOf(u'/');
    const QString hex = u'#' + id.left(cut);
    const QColor tint = QColor::fromString(hex);
    const QString name = id.mid(cut + 1);
    if (cut <= 0 || !tint.isValid() || tint.name() != hex || !bare_name(name, ext))
        return {};
    if (size.isValid() && size.isEmpty())
        return clear(size);
    QImageReader reader(root + u'/' + name);
    if (size.isValid())
        reader.setScaledSize(size);
    QImage art = reader.read();
    if (art.isNull())
        return {};
    art.convertTo(QImage::Format_ARGB32_Premultiplied);
    {
        QPainter paint(&art);
        paint.setCompositionMode(QPainter::CompositionMode_SourceIn);
        paint.fillRect(art.rect(), tint);
    }
    return art;
}

#if defined(KX_UI_PROBES)
// Counts the icon textures made and those alive, for tests: only in debug builds, whose `kx-ui-build.h` this file
// includes above.
struct IconCount
{
    inline static std::atomic<std::uint64_t> made{0};
    inline static std::atomic<std::uint64_t> live{0};

    IconCount()
    {
        ++made;
        ++live;
    }
    IconCount(const IconCount &) = delete;
    IconCount &operator=(const IconCount &) = delete;
    ~IconCount()
    {
        --live;
    }
};
#endif

// An icon's own texture, outside Qt's shared atlas: an icon drawn between whole pixels samples the same however other
// images, such as a colour change's steps, filled the atlas.
class IconTexture : public QQuickTextureFactory
{
public:
    explicit IconTexture(QImage art) : art(std::move(art)) {}

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
#if defined(KX_UI_PROBES)
    const IconCount counted;
#endif
};

// Serves `image://<provider>/<rrggbb>/<name>.<ext>` from the icons in `root`, each as its own texture. A refused or
// missing icon is a clear image of the asked size, and `refused` hears its id.
class TintIcons : public QQuickImageProvider
{
public:
    TintIcons(QString root, QString ext, Refused refused)
        : QQuickImageProvider(QQuickImageProvider::Texture), root(std::move(root)), ext(std::move(ext)),
          refused(std::move(refused))
    {
    }

    QQuickTextureFactory *requestTexture(const QString &id, QSize *size, const QSize &requested) override
    {
        QImage art = serve(id, requested);
        if (size)
            *size = art.size();
        return new IconTexture(std::move(art));
    }

    // Icon `id` at `requested`, or a clear image after telling `refused`.
    QImage serve(const QString &id, const QSize &requested) const
    {
        QImage art = tinted(root, ext, id, requested);
        if (art.isNull()) {
            refused(id);
            art = clear(requested);
        }
        return art;
    }

private:
    const QString root;
    const QString ext;
    const Refused refused;
};

} // namespace kxui
