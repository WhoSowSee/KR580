[CmdletBinding()]
param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "release",
    [string]$Target = "",
    [string]$DistDir = ""
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$ManifestPath = Join-Path $RepoRoot "Cargo.toml"
$ProfileArgs = @()
$TargetArgs = @()

if ($Profile -eq "release") {
    $ProfileArgs += "--release"
}

if (-not [string]::IsNullOrWhiteSpace($Target)) {
    $TargetArgs += "--target"
    $TargetArgs += $Target
}

if ([string]::IsNullOrWhiteSpace($DistDir)) {
    $DistDir = Join-Path $RepoRoot "dist"
}

Remove-Item Env:\KR580_WINDOWS_ICON_KIND -ErrorAction SilentlyContinue

& cargo build @ProfileArgs @TargetArgs -p k580-ui --bin k580 --bin kr --manifest-path $ManifestPath
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

$TargetRoot = if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
    Join-Path $RepoRoot "target"
} else {
    $env:CARGO_TARGET_DIR
}

$PayloadDir = if ([string]::IsNullOrWhiteSpace($Target)) {
    Join-Path $TargetRoot $Profile
} else {
    Join-Path (Join-Path $TargetRoot $Target) $Profile
}

$env:KR580_WINDOWS_ICON_KIND = "uninstaller"
try {
    & cargo build @ProfileArgs @TargetArgs -p k580-ui --bin k580-uninstaller --manifest-path $ManifestPath
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
finally {
    Remove-Item Env:\KR580_WINDOWS_ICON_KIND -ErrorAction SilentlyContinue
}

$env:KR580_INSTALLER_PAYLOAD_DIR = $PayloadDir
$env:KR580_WINDOWS_ICON_KIND = "setup"
try {
    & cargo build @ProfileArgs @TargetArgs -p k580-ui --bin k580-installer --manifest-path $ManifestPath
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
finally {
    Remove-Item Env:\KR580_INSTALLER_PAYLOAD_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:\KR580_WINDOWS_ICON_KIND -ErrorAction SilentlyContinue
}

New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

$CargoToml = Get-Content -LiteralPath $ManifestPath -Raw
if ($CargoToml -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
    throw "workspace version not found in $ManifestPath"
}
$Version = $Matches[1]
$Platform = if ([string]::IsNullOrWhiteSpace($Target)) {
    $Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
    "windows-$Arch"
} else {
    $Target
}
$Source = Join-Path $PayloadDir "k580-installer.exe"
$TargetPath = Join-Path $DistDir "KR580-Setup-$Version-$Platform.exe"

function Copy-InstallerArtifact {
    param(
        [Parameter(Mandatory)]
        [string]$SourcePath,
        [Parameter(Mandatory)]
        [string]$TargetPath
    )

    try {
        Copy-Item -LiteralPath $SourcePath -Destination $TargetPath -Force
        return $TargetPath
    }
    catch {
        if (-not ($_.Exception -is [System.IO.IOException])) {
            throw
        }
    }

    $TargetDir = Split-Path -Parent $TargetPath
    $BaseName = [System.IO.Path]::GetFileNameWithoutExtension($TargetPath)
    $Extension = [System.IO.Path]::GetExtension($TargetPath)
    for ($Index = 1; $Index -le 99; $Index++) {
        $Candidate = Join-Path $TargetDir "$BaseName-$Index$Extension"
        try {
            Copy-Item -LiteralPath $SourcePath -Destination $Candidate -Force
            return $Candidate
        }
        catch {
            if (-not ($_.Exception -is [System.IO.IOException])) {
                throw
            }
        }
    }

    throw "installer target is locked and no free numbered target is available"
}

$Written = Copy-InstallerArtifact -SourcePath $Source -TargetPath $TargetPath
Write-Host "Built installer: $Written"
