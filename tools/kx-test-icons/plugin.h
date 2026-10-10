// The Qt Quick tests' QML plugin: each test's engine gets kx-ui's icon provider (tint.h) over the icons laid out
// beside the plugin, so the tests draw icons as the app does.
#pragma once

#include <QtCore/QString>
#include <QtQml/QQmlEngine>
#include <QtQml/QQmlExtensionPlugin>
#include <QtQml/qqml.h>

#include "kx-test-icons.h"
#include "tint.h"

class TestIcons : public QQmlExtensionPlugin
{
    Q_OBJECT
    Q_PLUGIN_METADATA(IID QQmlExtensionInterface_iid)

public:
    // The tests' folder names no module, so the plugin registers the one Qt names after it, with no types.
    void registerTypes(const char *uri) override
    {
        qmlRegisterModule(uri, 1, 0);
    }

    // Serves the icons in the folder beside the plugin; a refused icon warns, which fails its test.
    void initializeEngine(QQmlEngine *engine, const char *) override
    {
        const QString root = baseUrl().toLocalFile() + u'/' + QStringLiteral(KX_ICONS_AT);
        const auto refused = [](const QString &id) { qWarning("icon refused or unread: %s", qPrintable(id)); };
        engine->addImageProvider(QStringLiteral(KX_ICON_PROVIDER),
                                 new kxui::TintIcons(root, QStringLiteral(KX_ICON_EXT), refused));
    }
};
