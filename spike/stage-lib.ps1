# Shared steps of the face stage scripts: dot-source this file first, then call Invoke-Stage.
# Cmdlets stop on errors; this local copy leaves the caller's own defaults untouched.
$PSDefaultParameterValues = $PSDefaultParameterValues.Clone()
$PSDefaultParameterValues['*:ErrorAction'] = 'Stop'

# Switches the D:\dev tools on for this process only.
$EnvScript = 'D:\dev\env.ps1'
# The spike folder (this file's folder) and the repository root.
$SpikeDir = $PSScriptRoot
$RootDir = Split-Path $SpikeDir -Parent
# Settings staged next to each exe.
$SettingsFile = Join-Path $SpikeDir 'spike.toml'
# Stage folders live in target\<this>\<face>.
$StageDir = 'spike-stage'
# The dev-install size cap, read from its one source in the root Cargo.toml.
$LimitFile = Join-Path $RootDir 'Cargo.toml'
$LimitPattern = '^install_max_mb\s*=\s*(\d+)'

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

# A fresh stage folder holding the face exe and spike.toml; returns its path.
function New-Stage([string] $Face) {
    $stage = Join-Path $RootDir "target\$StageDir\$Face"
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    New-Item -ItemType Directory -Path $stage | Out-Null
    Copy-Item (Join-Path $env:CARGO_TARGET_DIR "release\$Face.exe") $stage
    Copy-Item $SettingsFile $stage
    $stage
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
        & $Extra $stage
        Write-StageSize $stage
    } finally {
        Restore-Env $saved
    }
}
