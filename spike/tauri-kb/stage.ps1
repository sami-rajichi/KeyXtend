# Builds tauri-kb in release (uiAccess manifest) and stages it with spike.toml for dev-install.
# Usage: .\stage.ps1   (from any folder; env changes last only while it runs)
. (Join-Path (Split-Path $PSScriptRoot -Parent) 'stage-lib.ps1')

Invoke-Stage -Face 'tauri-kb' -TargetDir 'spike-tauri'
