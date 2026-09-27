# Builds qt-kb in release and stages it with its Qt files in target\spike-stage\qt-kb.
# Usage: .\stage.ps1   (from any folder; env changes last only while it runs)
. (Join-Path (Split-Path $PSScriptRoot -Parent) 'stage-lib.ps1')

# Qt kit that builds and deploys the face.
$qt = 'D:\dev\Qt\6.11.2\msvc2022_64'
# QML folder windeployqt scans for imports.
$qml = Join-Path $PSScriptRoot 'qml'
# Plugin types the keyboard never loads: no network, no QML debugging.
$skipPlugins = 'tls,networkinformation,qmltooling'

Invoke-Stage -Face 'qt-kb' -TargetDir 'spike-qt' -Vars @{ QMAKE = "$qt\bin\qmake.exe" } -Extra {
    param($stage)
    $deploy = @('--release', '--no-translations', '--no-compiler-runtime', '--no-opengl-sw',
        '--skip-plugin-types', $skipPlugins, '--qmldir', $qml, (Join-Path $stage 'qt-kb.exe'))
    Invoke-Tool 'windeployqt' { & "$qt\bin\windeployqt.exe" @deploy }
    # windeployqt leaves empty module folders behind; drop them, deepest first.
    Get-ChildItem $stage -Recurse -Directory | Sort-Object { $_.FullName.Length } -Descending |
        Where-Object { -not (Get-ChildItem $_.FullName -Force) } | Remove-Item
}
