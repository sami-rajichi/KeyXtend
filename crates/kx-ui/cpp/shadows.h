// Qt 6.11's RectangularShadow makes its default material with no owner and never frees it, so each shadow dropped kept
// one. QML's soft shadow (Soft.qml) hands that material to its shadow here, which then frees it.
#pragma once

#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtCore/QtVersionChecks>
#include <QtQml/QQmlApplicationEngine>
#include <QtQml/QQmlContext>
#include <QtQml/QQmlEngine>
#include <QtQuick/QQuickItem>

// A Qt that frees the material itself would free it twice, so each Qt upgrade rechecks RectangularShadow first.
static_assert(QT_VERSION < QT_VERSION_CHECK(6, 12, 0), "recheck who frees RectangularShadow's default material");

namespace kxui {

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
        if (material && !material->parent())
            material->setParent(shadow);
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
