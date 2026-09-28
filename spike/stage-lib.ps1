# Shared steps of the face stage scripts: dot-source this file first, then call Invoke-Stage.
# Cmdlets stop on errors; this local copy leaves the caller's own defaults untouched.
$PSDefaultParameterValues = $PSDefaultParameterValues.Clone()
$PSDefaultParameterValues['*:ErrorAction'] = 'Stop'

# Switches the D:\dev tools on for this process only.
$EnvScript = 'D:\dev\env.ps1'
# The spike folder (this file's folder) and the repository root.
$SpikeDir = $PSScriptRoot
$RootDir = Split-Path $SpikeDir -Parent
# Settings and themes staged next to each exe.
$SettingsFile = Join-Path $SpikeDir 'spike.toml'
$ThemesFile = Join-Path $SpikeDir 'themes.toml'
$ShapeFile = Join-Path $SpikeDir 'shape.toml'
$MotionFile = Join-Path $SpikeDir 'motion.toml'
# Fonts and icons stay out of git in D:\dev; the spike's own icons are in spike\assets\icons.
$AssetsSource = 'D:\dev\assets'
$LucideDir = 'icons\lucide-1.48.0'
$FontsDir = 'fonts'
$OwnIcons = Join-Path $SpikeDir 'assets\icons'
# Where spike.toml [assets] puts fonts and icons, read from that one source.
$FontsPattern = '^fonts\s*=\s*"([^"]+)"'
$IconsPattern = '^icons\s*=\s*"([^"]+)"'
# Stage folders live in target\<this>\<face>.
$StageDir = 'spike-stage'
# The dev-install size cap, read from its one source in the root Cargo.toml.
$LimitFile = Join-Path $RootDir 'Cargo.toml'
$LimitPattern = '^install_max_mb\s*=\s*(\d+)'
# The voice worker's exe name, read from its one source in spike.toml; its crate has the same stem.
$WorkerPattern = '^worker\s*=\s*"([^"]+)"'

# Runs a native tool; throws when it exits non-zero, since cargo logs progress to stderr.
function Invoke-Tool([string] $What, [scriptblock] $Run) {
    & $Run
    if ($LASTEXITCODE -ne 0) { throw "$What failed ($LASTEXITCODE)" }
}

# Every process env var, to put back when staging ends.
function Save-Env {
    [Environment]::GetEnvironmentVariables('Process')
}

# Puts the process env vars back as $Saved had them, dropping any added since.
function Restore-Env([Collections.IDictionary] $Saved) {
    foreach ($name in @([Environment]::GetEnvironmentVariables('Process').Keys)) {
        if (-not $Saved.Contains($name)) {
            [Environment]::SetEnvironmentVariable($name, $null, 'Process')
        }
    }
    foreach ($name in $Saved.Keys) {
        [Environment]::SetEnvironmentVariable($name, $Saved[$name], 'Process')
    }
}

# Builds the face in $Dir in release; cargo finds that face's own workspace from there.
function Build-Face([string] $Dir) {
    Push-Location $Dir
    try {
        Invoke-Tool 'cargo build --release' { cargo build --release }
    } finally {
        Pop-Location
    }
}

# A fresh stage folder holding the face exe and its settings files; returns its path.
function New-Stage([string] $Face) {
    $stage = Join-Path $RootDir "target\$StageDir\$Face"
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    New-Item -ItemType Directory -Path $stage | Out-Null
    Copy-Item (Join-Path $env:CARGO_TARGET_DIR "release\$Face.exe") $stage
    Copy-Item $SettingsFile $stage
    Copy-Item $ThemesFile $stage
    Copy-Item $ShapeFile $stage
    Copy-Item $MotionFile $stage
    $stage
}

# The value of $Pattern's group in spike.toml.
function Get-Setting([string] $Pattern) {
    $hit = Select-String -Path $SettingsFile -Pattern $Pattern
    if (-not $hit) { throw "$Pattern not found in $SettingsFile" }
    $hit[0].Matches[0].Groups[1].Value
}

# Copies the fonts and icons into $Stage where spike.toml [assets] says the faces look.
function Add-Assets([string] $Stage) {
    $fonts = Join-Path $Stage (Get-Setting $FontsPattern)
    $icons = Join-Path $Stage (Get-Setting $IconsPattern)
    New-Item -ItemType Directory -Path $fonts, $icons -Force | Out-Null
    Get-ChildItem -LiteralPath (Join-Path $AssetsSource $FontsDir) | Copy-Item -Destination $fonts -Recurse
    Get-ChildItem -LiteralPath (Join-Path $AssetsSource $LucideDir) | Copy-Item -Destination $icons
    Get-ChildItem -LiteralPath $OwnIcons | Copy-Item -Destination $icons
}

# Builds the voice worker in release and copies it into $Stage, next to the face.
function Add-Worker([string] $Stage) {
    $exe = Get-Setting $WorkerPattern
    Push-Location $SpikeDir
    try {
        Invoke-Tool 'cargo build worker' { cargo build --release -p ([IO.Path]::GetFileNameWithoutExtension($exe)) }
    } finally {
        Pop-Location
    }
    Copy-Item (Join-Path $env:CARGO_TARGET_DIR "release\$exe") $Stage
}

# The dev-install size cap in MB.
function Get-LimitMb {
    $hit = Select-String -Path $LimitFile -Pattern $LimitPattern
    if (-not $hit) { throw "install_max_mb not found in $LimitFile" }
    [int]$hit[0].Matches[0].Groups[1].Value
}

# Prints the file count, link count and size of $Stage; warns when dev-install would refuse it.
function Write-StageSize([string] $Stage) {
    $files = @(Get-ChildItem $Stage -Recurse -File)
    $links = @(Get-ChildItem $Stage -Recurse -Attributes ReparsePoint)
    $mb = [math]::Round(($files | Measure-Object Length -Sum).Sum / 1MB, 1)
    "staged: $Stage"
    "files: $($files.Count), links: $($links.Count), size: $mb MB"
    $limit = Get-LimitMb
    if ($mb -gt $limit) { Write-Warning "staged folder is over $limit MB" }
}

# Builds and stages one face; $Vars adds env vars and $Extra runs on the stage folder.
function Invoke-Stage {
    param(
        [Parameter(Mandatory)] [string] $Face,
        [Parameter(Mandatory)] [string] $TargetDir,
        [hashtable] $Vars = @{},
        [scriptblock] $Extra = {}
    )
    $saved = Save-Env
    try {
        . $EnvScript
        $env:CARGO_TARGET_DIR = Join-Path $RootDir "target\$TargetDir"
        foreach ($name in $Vars.Keys) {
            [Environment]::SetEnvironmentVariable($name, $Vars[$name], 'Process')
        }
        Build-Face (Join-Path $SpikeDir $Face)
        $stage = New-Stage $Face
        Add-Assets $stage
        Add-Worker $stage
        & $Extra $stage
        Write-StageSize $stage
    } finally {
        Restore-Env $saved
    }
}
