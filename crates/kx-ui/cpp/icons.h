// The bridge's side of the icon provider (tint.h): src/bridges/tint.rs registers it on the app's engine, and debug
// builds read what it serves, for tests.
#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QString>
#include <QtGui/QImage>
#include <QtGui/QImageReader>
#include <QtQml/QQmlApplicationEngine>

#include <cstddef>
#include <cstdint>
#include <memory>

#include "kx-ui-build.h"
#include "rust/cxx.h"
#include "tint.h"

namespace kxui {

// What hears a refused icon's id, in Rust.
using Heard = rust::Fn<void(rust::Str)>;

// `text` as a Qt string.
inline QString icon_text(rust::Str text)
{
    return QString::fromUtf8(text.data(), qsizetype(text.size()));
}

// A new provider over the icons of type `ext` in folder `root`, which tells `heard` each refused id.
inline TintIcons *tint_icons(rust::Str root, rust::Str ext, Heard heard)
{
    return new TintIcons(icon_text(root), icon_text(ext), [heard](const QString &id) {
        const QByteArray utf8 = id.toUtf8();
        heard(rust::Str(utf8.constData(), std::size_t(utf8.size())));
    });
}

// Registers that provider on `engine` as `name`; the engine owns it.
inline void serve_icons(QQmlApplicationEngine &engine, rust::Str name, rust::Str root, rust::Str ext, Heard heard)
{
    engine.addImageProvider(icon_text(name), tint_icons(root, ext, heard));
}

#if defined(KX_UI_PROBES)
// `art`'s premultiplied ARGB words, row by row.
inline rust::Vec<std::uint32_t> icon_words(const QImage &art)
{
    const QImage words = art.convertToFormat(QImage::Format_ARGB32_Premultiplied);
    rust::Vec<std::uint32_t> out;
    for (int y = 0; y < words.height(); ++y) {
        const auto *row = reinterpret_cast<const QRgb *>(words.constScanLine(y));
        for (int x = 0; x < words.width(); ++x)
            out.push_back(row[x]);
    }
    return out;
}

// What the provider serves for `id` at `side` pixels square; debug builds only.
inline rust::Vec<std::uint32_t> icon_pixels(rust::Str root, rust::Str ext, rust::Str id, std::int32_t side,
                                            Heard heard)
{
    const std::unique_ptr<TintIcons> icons(tint_icons(root, ext, heard));
    return icon_words(icons->requestImage(icon_text(id), nullptr, QSize(side, side)));
}

// File `name` in `root` read at `side` pixels square, untinted; debug builds only.
inline rust::Vec<std::uint32_t> icon_mask(rust::Str root, rust::Str name, std::int32_t side)
{
    QImageReader reader(icon_text(root) + u'/' + icon_text(name));
    reader.setScaledSize(QSize(side, side));
    return icon_words(reader.read());
}
#endif

} // namespace kxui
