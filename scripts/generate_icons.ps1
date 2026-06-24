# One-shot helper to regenerate the runtime icon set. Run from the
# repository root only when one of the source PNGs changes:
#
#   powershell -File scripts/generate_icons.ps1
#
# Sources (in `assets/icons/`):
#   - `icon.png`     — application icon master.
#   - `file-580.png` — `.580` file-type icon master.
#   - `installer-setup.png` — standalone setup icon master.
#   - `installer-uninstall.png` — installed uninstaller icon master.
#
# Outputs in `assets/icons/`, then mirrors the complete icon tree into
# `crates/ui/assets/icons/` so the published crates.io package is
# self-contained:
#   - `icon-{16,32,48,64,128,256}.png` — standalone cross-platform PNGs.
#   - `icon.ico`                       — multi-resolution Windows app icon.
#   - `file-580.ico`                   — multi-resolution `.580` file-type icon.
#   - `installer-setup.ico`            — multi-resolution setup `.exe` icon.
#   - `installer-uninstall.ico`        — multi-resolution uninstaller `.exe` icon.

Add-Type -AssemblyName System.Drawing

$root = Resolve-Path "$PSScriptRoot\.."
$outDir = Join-Path $root 'assets\icons'
$crateOutDir = Join-Path $root 'crates\ui\assets\icons'

if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

# Sizes shipped as standalone PNG (cross-platform window/desktop icons).
$pngSizes = 16, 32, 48, 64, 128, 256

# Sizes packed into the multi-resolution Windows .ico (covers every typical
# Explorer/taskbar/Start-menu DPI scaling combination on Windows 10/11).
$appIcoSizes  = 16, 20, 24, 32, 40, 48, 64, 96, 256
$fileIcoSizes = 16, 20, 24, 32, 40, 48, 64, 96, 128, 256
$installerIcoSizes = 16, 20, 24, 32, 40, 48, 64, 96, 128, 256

function ConvertTo-PngBytes {
    param([System.Drawing.Bitmap]$Bitmap)

    $stream = New-Object System.IO.MemoryStream
    try {
        $Bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
        return $stream.ToArray()
    } finally {
        $stream.Dispose()
    }
}

function Resize-Bitmap {
    param([System.Drawing.Image]$Source, [int]$Size)

    $bitmap = New-Object System.Drawing.Bitmap $Size, $Size
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.InterpolationMode  = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $graphics.SmoothingMode      = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
        $graphics.PixelOffsetMode    = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
        $graphics.DrawImage($Source, 0, 0, $Size, $Size)
    } finally {
        $graphics.Dispose()
    }
    return $bitmap
}

function Write-IcoFile {
    # Multi-resolution ICO. The format is well documented:
    #   https://learn.microsoft.com/windows/win32/menurc/icon-resource
    # We embed PNG payloads (Vista+ supports this for any size and saves a lot
    # of bytes vs. raw BMP, especially for 256x256). Frames are written
    # largest-first so default Windows previewers (Photos, Paint, IconViewer)
    # display the largest layer when the file is opened directly.
    param(
        [hashtable]$PngBytesBySize,
        [int[]]$Sizes,
        [string]$IcoPath
    )

    $sorted = $Sizes | Sort-Object -Unique -Descending
    $headerSize = 6 + 16 * $sorted.Count

    $memory = New-Object System.IO.MemoryStream
    $writer = New-Object System.IO.BinaryWriter $memory
    try {
        # ICONDIR
        $writer.Write([uint16]0)                # reserved, must be 0
        $writer.Write([uint16]1)                # 1 = ICO (2 = CUR)
        $writer.Write([uint16]$sorted.Count)

        # ICONDIRENTRY[]
        $offset = $headerSize
        foreach ($size in $sorted) {
            $bytes = $PngBytesBySize[$size]
            $dim = if ($size -ge 256) { [byte]0 } else { [byte]$size }
            $writer.Write($dim)                 # width  (0 means 256)
            $writer.Write($dim)                 # height (0 means 256)
            $writer.Write([byte]0)              # color count (0 for 32 bpp)
            $writer.Write([byte]0)              # reserved
            $writer.Write([uint16]1)            # color planes
            $writer.Write([uint16]32)           # bits per pixel
            $writer.Write([uint32]$bytes.Length)
            $writer.Write([uint32]$offset)
            $offset += $bytes.Length
        }

        # Image payloads (PNG bytes, in the same order as ICONDIRENTRY).
        # Use the underlying stream's `Write(byte[], int, int)` to avoid
        # PowerShell ambiguity between `BinaryWriter.Write(byte[])` and
        # `Write(char[])`, which previously produced a 0-byte payload.
        foreach ($size in $sorted) {
            $bytes = $PngBytesBySize[$size]
            $memory.Write($bytes, 0, $bytes.Length)
        }

        $writer.Flush()
        [System.IO.File]::WriteAllBytes($IcoPath, $memory.ToArray())
    } finally {
        $writer.Dispose()
        $memory.Dispose()
    }
    Write-Host "Wrote $IcoPath"
}

function Build-IconSet {
    param(
        [string]$SourcePng,
        [string]$IcoPath,
        [int[]]$IcoSizes,
        [int[]]$ExtraPngSizes = @()
    )

    if (-not (Test-Path $SourcePng)) {
        throw "Missing source icon: $SourcePng"
    }

    $allSizes = ($IcoSizes + $ExtraPngSizes) | Sort-Object -Unique
    $original = [System.Drawing.Image]::FromFile($SourcePng)
    try {
        # Render every required size exactly once. Re-encoding from the master
        # avoids quality loss vs. resampling already-shrunk PNGs.
        $pngBytesBySize = @{}
        foreach ($size in $allSizes) {
            $bitmap = Resize-Bitmap $original $size
            try {
                $pngBytesBySize[$size] = ConvertTo-PngBytes $bitmap
            } finally {
                $bitmap.Dispose()
            }
        }

        # Standalone PNGs (only emitted when ExtraPngSizes is non-empty).
        foreach ($size in $ExtraPngSizes) {
            $sourceLeaf = [System.IO.Path]::GetFileNameWithoutExtension($SourcePng)
            $target = Join-Path (Split-Path -Parent $SourcePng) ("{0}-{1}.png" -f $sourceLeaf, $size)
            [System.IO.File]::WriteAllBytes($target, $pngBytesBySize[$size])
            Write-Host "Wrote $target"
        }

        Write-IcoFile -PngBytesBySize $pngBytesBySize -Sizes $IcoSizes -IcoPath $IcoPath
    } finally {
        $original.Dispose()
    }
}

# Application icon: emits standalone PNGs + icon.ico.
Build-IconSet `
    -SourcePng     (Join-Path $outDir 'icon.png') `
    -IcoPath       (Join-Path $outDir 'icon.ico') `
    -IcoSizes      $appIcoSizes `
    -ExtraPngSizes $pngSizes

# `.580` file-type icon: only the multi-resolution ICO is needed (it goes
# into the PE resource section as resource id 2 via `crates/ui/build.rs`).
Build-IconSet `
    -SourcePng (Join-Path $outDir 'file-580.png') `
    -IcoPath   (Join-Path $outDir 'file-580.ico') `
    -IcoSizes  $fileIcoSizes

Build-IconSet `
    -SourcePng (Join-Path $outDir 'installer-setup.png') `
    -IcoPath   (Join-Path $outDir 'installer-setup.ico') `
    -IcoSizes  $installerIcoSizes

Build-IconSet `
    -SourcePng (Join-Path $outDir 'installer-uninstall.png') `
    -IcoPath   (Join-Path $outDir 'installer-uninstall.ico') `
    -IcoSizes  $installerIcoSizes

if (Test-Path $crateOutDir) {
    Remove-Item -Recurse -Force $crateOutDir
}
New-Item -ItemType Directory -Force -Path $crateOutDir | Out-Null
Copy-Item -Recurse -Force -Path (Join-Path $outDir '*') -Destination $crateOutDir
Write-Host "Synced $crateOutDir"
