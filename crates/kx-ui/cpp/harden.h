// Qt's start in the uiAccess process; src/bridges/harden.rs calls these before Qt's app object exists.
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QStringList>
#include <QtCore/qtenvironmentvariables.h>

#include <string>

#include "rust/cxx.h"

namespace kxui {

// Removes variable `name` from the C runtime's environment, which Qt reads, and from the process's.
inline bool unset_env(rust::Str name)
{
    const std::string text(name.data(), name.size());
    return qunsetenv(text.c_str());
}

// Qt Quick neither reads nor writes its shader pipeline cache on disk.
inline void no_shader_cache()
{
    QCoreApplication::setAttribute(Qt::AA_DisableShaderDiskCache);
}

// Qt loads plugins only from `paths` and the app's own folder.
inline void set_library_paths(const QStringList &paths)
{
    QCoreApplication::setLibraryPaths(paths);
}

} // namespace kxui
