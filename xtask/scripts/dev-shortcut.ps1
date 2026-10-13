# User-level Start-menu shortcut steps for `cargo xtask dev-install`.
param(
    [Parameter(Mandatory)][ValidateSet('Add', 'Find', 'Remove')][string]$Action,
    [Parameter(Mandatory)][string]$Link,
    [string]$Exe
)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
# The Windows Script Host shell, which reads and writes .lnk files.
$Shell = New-Object -ComObject WScript.Shell

# The program the shortcut at $Link starts, or nothing when there is none.
function Get-Target {
    if (Test-Path -LiteralPath $Link -PathType Leaf) { $Shell.CreateShortcut($Link).TargetPath }
}

# $Path with long folder names, as a shortcut reports its program; Windows PowerShell expands short (8.3) names.
function Get-Long([string]$Path) {
    if ($Path -and (Test-Path -LiteralPath $Path)) { (Get-Item -LiteralPath $Path).FullName } else { $Path }
}

switch ($Action) {
    'Add' {
        if (-not (Test-Path -LiteralPath $Exe -PathType Leaf)) { throw "$Exe is not a file." }
        if (Test-Path -LiteralPath $Link) { Remove-Item -LiteralPath $Link -Force }
        $lnk = $Shell.CreateShortcut($Link)
        $lnk.TargetPath = $Exe
        $lnk.WorkingDirectory = [IO.Path]::GetDirectoryName($Exe)
        $lnk.Save()
    }
    'Find' { Get-Target }
    'Remove' { if ($Exe -and (Get-Long (Get-Target)) -eq (Get-Long $Exe)) { Remove-Item -LiteralPath $Link -Force } }
}
