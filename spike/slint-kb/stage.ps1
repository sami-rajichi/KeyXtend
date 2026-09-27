# Builds slint-kb in release and stages it with spike.toml in target\spike-stage\slint-kb.
# Usage: .\stage.ps1   (from any folder; env changes last only while it runs)
. (Join-Path (Split-Path $PSScriptRoot -Parent) 'stage-lib.ps1')

Invoke-Stage -Face 'slint-kb' -TargetDir 'spike-slint'
