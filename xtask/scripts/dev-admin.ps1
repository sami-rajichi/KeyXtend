# Admin steps for KeyXtend test builds; asks Windows for admin rights when needed.
param(
    [Parameter(Mandatory)][ValidateSet('Trust', 'Untrust', 'Install', 'Uninstall')][string]$Action,
    [Parameter(Mandatory)][ValidateRange(1, 2147483647)][int]$CancelCode,
    [Parameter(Mandatory)][ValidateRange(1, 2147483647)][int]$FailCode,
    [string]$CertFile,
    [string]$Thumbprint,
    [string]$Subject,
    [string]$Source,
    [string]$Target,
    [string]$InstallDir,
    [string]$SecureEnv,
    [string]$ErrorFile,
    [switch]$Notify,
    [switch]$ValidateOnly
)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
# The script's own parameters, kept for the elevated re-run.
$Bound = $PSBoundParameters
# The machine-wide trusted root store that uiAccess signatures must chain to.
$RootName = 'Root'
$RootLocation = 'LocalMachine'
# A plain name: no separators, wildcards or reserved characters, and no trailing dot or space.
$PlainName = '^[^\\/:*?"<>|\[\]`]*[^\\/:*?"<>|\[\]`. ]$'
# Windows device names, with or without an extension; the digits include superscript 1-3.
$Device = '^(CON|PRN|AUX|NUL|COM[1-9\xB9\xB2\xB3]|LPT[1-9\xB9\xB2\xB3])(\..*)?$'
# A certificate thumbprint.
$Hex40 = '^[0-9A-Fa-f]{40}$'
# Separator of the thumbprint list; the Rust side names the same one as LIST_SEP.
$ListSep = ','
# Windows' ERROR_CANCELLED, raised when the owner answers No at the admin prompt.
$ErrorCancelled = 1223
# The shell verb that asks Windows for admin rights.
$AdminVerb = 'runas'

function Test-Admin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    ([Security.Principal.WindowsPrincipal]$id).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-Plain([string]$Name) {
    ($Name -match $PlainName) -and ($Name -notmatch $Device)
}

# Shows the result in a message box when a launcher started this script.
function Show-Result([string]$Text) {
    if (-not $Notify) { return }
    Add-Type -AssemblyName System.Windows.Forms
    [void][System.Windows.Forms.MessageBox]::Show($Text, 'KeyXtend')
}

# Reports a failure on stderr, in the error file and in the message box, then exits.
function Stop-WithError([string]$Text) {
    [Console]::Error.WriteLine($Text)
    if ($ErrorFile) { Set-Content -LiteralPath $ErrorFile -Value $Text -Encoding UTF8 }
    Show-Result "Failed: $Text"
    exit $FailCode
}

# The thumbprints in the comma-separated list, or nothing if any entry is malformed.
function Get-Thumbprints {
    $list = @($Thumbprint.Split($ListSep))
    if ($list | Where-Object { $_ -notmatch $Hex40 }) { return @() }
    $list
}

# Refuses any target that is not a plain folder directly inside <secure folder>\<InstallDir>.
function Assert-Target {
    if (-not (Test-Plain $InstallDir)) { throw 'InstallDir must be one plain folder name.' }
    $root = [Environment]::GetEnvironmentVariable($SecureEnv)
    if (-not $root -or -not [IO.Path]::IsPathRooted($root)) { throw "$SecureEnv does not name a folder." }
    $allowed = [IO.Path]::GetFullPath((Join-Path $root $InstallDir))
    $full = [IO.Path]::GetFullPath($Target)
    $leaf = Split-Path $full -Leaf
    if ((Split-Path $full -Parent) -ne $allowed -or -not (Test-Plain $leaf) -or $Target -ne $full) {
        throw "Refusing a target that is not a plain folder in $allowed."
    }
}

# Checks the parameters of the chosen action before anything changes.
function Assert-Arguments {
    switch ($Action) {
        'Trust' {
            if ($Thumbprint -notmatch $Hex40) { throw 'Trust needs a valid thumbprint.' }
            if (-not $CertFile -or -not (Test-Path -LiteralPath $CertFile -PathType Leaf)) { throw 'Trust needs the exported certificate file.' }
        }
        'Untrust' { if (-not $Subject -or -not (Get-Thumbprints)) { throw 'Untrust needs a subject and valid thumbprints.' } }
        'Install' {
            Assert-Target
            if (-not (Test-Path -LiteralPath $Source -PathType Container)) { throw "$Source is not a folder." }
            if (Get-ChildItem -LiteralPath $Source -Recurse -Force -Attributes ReparsePoint) { throw "$Source holds a link; the copy would follow it." }
        }
        'Uninstall' { Assert-Target }
    }
}

# Quotes one argument for the elevated command line, doubling trailing backslashes.
function Format-Arg([string]$Value) {
    '"' + ($Value -replace '(\\+)$', '$1$1') + '"'
}

# True if the exception chain holds Windows' "cancelled by the user" error.
function Test-Cancelled($Err) {
    while ($Err) {
        if ($Err -is [ComponentModel.Win32Exception] -and $Err.NativeErrorCode -eq $ErrorCancelled) { return $true }
        $Err = $Err.InnerException
    }
    $false
}

# Re-runs this script elevated with the same parameters, passes on its error, and exits with its code.
# It uses .NET, not Start-Process, because Start-Process drops the Windows error code of a cancel.
function Invoke-Elevated {
    $list = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Format-Arg $PSCommandPath))
    foreach ($p in $Bound.GetEnumerator()) {
        if ($p.Value -is [switch]) { if ($p.Value) { $list += "-$($p.Key)" } }
        else { $list += "-$($p.Key)"; $list += (Format-Arg "$($p.Value)") }
    }
    if ($ErrorFile) { Remove-Item -LiteralPath $ErrorFile -ErrorAction SilentlyContinue }
    $shell = (Get-Process -Id $PID).Path
    $start = New-Object Diagnostics.ProcessStartInfo -ArgumentList $shell, ($list -join ' ')
    $start.UseShellExecute = $true
    $start.Verb = $AdminVerb
    try { $proc = [Diagnostics.Process]::Start($start) }
    catch {
        if (Test-Cancelled $_.Exception) { exit $CancelCode }
        Stop-WithError $_.Exception.Message
    }
    $proc.WaitForExit()
    if ($proc.ExitCode -ne 0 -and $ErrorFile -and (Test-Path -LiteralPath $ErrorFile)) {
        [Console]::Error.WriteLine((Get-Content -LiteralPath $ErrorFile -Raw))
    }
    exit $proc.ExitCode
}

# Adds exactly the certificate object that was checked to the machine root store.
function Add-Trust {
    $cert = New-Object Security.Cryptography.X509Certificates.X509Certificate2 $CertFile
    if ($cert.Thumbprint -ne $Thumbprint) { throw 'The certificate file does not match the expected thumbprint.' }
    $store = New-Object Security.Cryptography.X509Certificates.X509Store $RootName, $RootLocation
    $store.Open('ReadWrite')
    try { $store.Add($cert) } finally { $store.Close() }
    Show-Result 'The KeyXtend test certificate is now trusted.'
}

# Removes only root certificates that match both a listed thumbprint and the subject.
function Remove-Trust {
    $list = Get-Thumbprints
    $store = New-Object Security.Cryptography.X509Certificates.X509Store $RootName, $RootLocation
    $store.Open('ReadWrite')
    try {
        @($store.Certificates) | Where-Object { $_.Subject -eq $Subject -and $list -contains $_.Thumbprint } |
            ForEach-Object { $store.Remove($_) }
    }
    finally { $store.Close() }
    Show-Result 'The KeyXtend test certificate is no longer trusted.'
}

function Install-Build {
    if (Test-Path -LiteralPath $Target) { Remove-Item -LiteralPath $Target -Recurse -Force }
    New-Item -ItemType Directory -Path $Target -Force | Out-Null
    Get-ChildItem -LiteralPath $Source -Force | Copy-Item -Destination $Target -Recurse -Force
}

function Remove-Build {
    if (Test-Path -LiteralPath $Target) { Remove-Item -LiteralPath $Target -Recurse -Force }
}

try { Assert-Arguments } catch { Stop-WithError $_.Exception.Message }
if ($ValidateOnly) { exit 0 }
if (-not (Test-Admin)) { Invoke-Elevated }
try {
    switch ($Action) {
        'Trust' { Add-Trust }
        'Untrust' { Remove-Trust }
        'Install' { Install-Build }
        'Uninstall' { Remove-Build }
    }
}
catch { Stop-WithError $_.Exception.Message }
exit 0
