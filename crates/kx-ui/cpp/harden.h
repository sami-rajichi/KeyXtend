// Qt's start in the uiAccess process; src/bridges/harden.rs calls these before Qt's app object exists.
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QStringList>
#include <QtCore/qtenvironmentvariables.h>

#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <string>

#include "rust/cxx.h"

#if !defined(_WIN32)
extern char **environ;
#endif

namespace kxui {

// The C runtime's narrow environment, which Qt's qgetenv reads on Windows.
inline char **narrow_environment()
{
#if defined(_WIN32)
#if defined(_MSC_VER)
#pragma warning(push)
#pragma warning(disable : 4996)
#endif
    return _environ;
#if defined(_MSC_VER)
#pragma warning(pop)
#endif
#else
    return environ;
#endif
}

// Every name in the narrow environment, each ended by a NUL; no value leaves it. It reads without the runtime's lock,
// so no other thread may write the environment meanwhile.
inline rust::Vec<std::uint8_t> narrow_names()
{
    rust::Vec<std::uint8_t> names;
    for (char **entry = narrow_environment(); entry && *entry; ++entry) {
        const char *text = *entry;
        // A name may start with '=', as "=C:" does; its value starts after the next one.
        const char *value = std::strchr(text + (text[0] == '=' ? 1 : 0), '=');
        const std::size_t size = value ? std::size_t(value - text) : std::strlen(text);
        for (std::size_t i = 0; i < size; ++i)
            names.push_back(static_cast<std::uint8_t>(text[i]));
        names.push_back(0);
    }
    return names;
}

// Removes variable `name`, in the narrow environment's bytes, from the C runtime's copies and the process's.
inline bool unset_env(rust::Slice<const std::uint8_t> name)
{
    const std::string text(reinterpret_cast<const char *>(name.data()), name.size());
    return qunsetenv(text.c_str());
}

// True when Qt reads variable `name` as set; on Windows it reads the C runtime's narrow copy.
inline bool qt_sees(rust::Str name)
{
    const std::string text(name.data(), name.size());
    return qEnvironmentVariableIsSet(text.c_str());
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
