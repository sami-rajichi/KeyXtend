// Qt 6.11's RectangularShadow makes its default material in its private constructor through Q_Q, before q_ptr is set,
// so the material has no parent and nothing frees it. QML's soft shadow (Soft.qml) hands it to its shadow here, and
// every build of this file refuses any Qt but the pinned one (qtbuild.rs), so each Qt upgrade rechecks who frees it.
#pragma once

#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtQml/QQmlApplicationEngine>
#include <QtQml/QQmlContext>
#include <QtQml/QQmlEngine>
#include <QtQuick/QQuickItem>

#include <atomic>
#include <cstddef>

// The build kind sets `adopt` below the same way in every file; the Quick tests' plugin writes it empty.
#include "kx-ui-build.h"

namespace kxui {

#if defined(KX_UI_PROBES)
// How many materials the adopters gave their shadows so far; debug builds only.
inline std::atomic<std::size_t> adoptions{0};

inline std::size_t adopted_materials()
{
    return adoptions.load();
}
#endif

// The name QML reaches the adopter by, and the shadow's property that holds its material.
inline constexpr char SHADOWS[] = "shadows";
inline constexpr char MATERIAL[] = "material";

// Gives a soft shadow its own default material, so Qt deletes both together.
class Shadows : public QObject
{
    Q_OBJECT

public:
    using QObject::QObject;

    // Makes `shadow` own its material when nothing does; a material QML set has an owner already.
    Q_INVOKABLE void adopt(QQuickItem *shadow) const
    {
        QQuickItem *material = shadow ? shadow->property(MATERIAL).value<QQuickItem *>() : nullptr;
        if (!material || material->parent())
            return;
        material->setParent(shadow);
#if defined(KX_UI_PROBES)
        ++adoptions;
#endif
    }
};

// Puts one adopter, which `engine` owns, on its root context; call it before the engine loads QML.
inline void serve_shadows(QQmlEngine &engine)
{
    engine.rootContext()->setContextProperty(QString::fromLatin1(SHADOWS), new Shadows(&engine));
}

// The same for the app's engine, as the bridge names it.
inline void serve_shadows(QQmlApplicationEngine &engine)
{
    serve_shadows(static_cast<QQmlEngine &>(engine));
}

} // namespace kxui
