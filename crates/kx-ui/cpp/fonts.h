// The theme's bundled fonts in Qt's font list (ADR-0021); src/bridges/fonts.rs calls these on the Qt GUI thread.
#pragma once

#include <QtCore/QString>
#include <QtGui/QFontDatabase>

#include <cstdint>

#include "rust/cxx.h"

namespace kxui {

// Adds the font file at `path`, a compiled-in resource, to Qt's font list; its id, or -1 when it did not load.
inline std::int32_t font_load(rust::Str path)
{
    return QFontDatabase::addApplicationFont(QString::fromUtf8(path.data(), qsizetype(path.size())));
}

// Drops the font file added as `id`; false when Qt held none.
inline bool font_unload(std::int32_t id)
{
    return QFontDatabase::removeApplicationFont(id);
}

} // namespace kxui
