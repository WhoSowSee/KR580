# One-shot helper to regenerate the runtime icon set from the source
# `icon.png`. Run from the repository root only when the source changes:
#
#   powershell -File scripts/generate_icons.ps1
#
# The generated files in `assets/icons/` are checked into the repository so
# the application binary does not need to decode or resize the master image
# at build time or at run time.

Add-Type -AssemblyName System.Drawing

$root = Resolve-Path "$PSScriptRoot\.."
$outDir = Join-Path $root 'assets\icons'
$source = Join-Path $outDir 'icon.png'
$icoPath = Join-Path $outDir 'icon.ico'

if (-not (Test-Path $source)) {
    throw "Missing source icon: $source"
}

if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

# Sizes shipped as standalone PNG (cross-platform window/desktop icons).
$pngSizes = 16, 32, 48, 64, 128, 256

# Sizes packed into the multi-resolution Windows .ico (covers every typical
# Explorer/taskbar/Start-menu DPI scaling combination on Windows 10/11).
$icoSizes = 16, 20, 24, 32, 40, 48, 64, 96, 256

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

$original = [System.Drawing.Image]::FromFile($source)
try {
    # Render every required size exactly once. Re-encoding from the master
    # avoids quality loss vs. resampling already-shrunk PNGs.
    $allSizes = ($pngSizes + $icoSizes) | Sort-Object -Unique
    $pngBytesBySize = @{}
    foreach ($size in $allSizes) {
        $bitmap = Resize-Bitmap $original $size
        try {
            $pngBytesBySize[$size] = ConvertTo-PngBytes $bitmap
        } finally {
            $bitmap.Dispose()
        }
    }

    # Standalone PNGs.
    foreach ($size in $pngSizes) {
        $target = Join-Path $outDir ("icon-{0}.png" -f $size)
        [System.IO.File]::WriteAllBytes($target, $pngBytesBySize[$size])
        Write-Host "Wrote $target"
    }

    # Multi-resolution ICO. The format is well documented:
    #   https://learn.microsoft.com/windows/win32/menurc/icon-resource
    # We embed PNG payloads (Vista+ supports this for any size and saves a lot
    # of bytes vs. raw BMP, especially for 256x256). Frames are written
    # largest-first so default Windows previewers (Photos, Paint, IconViewer)
    # display the 256x256 layer when the file is opened directly.
    $icoSorted = $icoSizes | Sort-Object -Unique -Descending
    $headerSize = 6 + 16 * $icoSorted.Count

    $memory = New-Object System.IO.MemoryStream
    $writer = New-Object System.IO.BinaryWriter $memory
    try {
        # ICONDIR
        $writer.Write([uint16]0)                # reserved, must be 0
        $writer.Write([uint16]1)                # 1 = ICO (2 = CUR)
        $writer.Write([uint16]$icoSorted.Count)

        # ICONDIRENTRY[]
        $offset = $headerSize
        foreach ($size in $icoSorted) {
            $bytes = $pngBytesBySize[$size]
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
        foreach ($size in $icoSorted) {
            $bytes = $pngBytesBySize[$size]
            $memory.Write($bytes, 0, $bytes.Length)
        }

        $writer.Flush()
        [System.IO.File]::WriteAllBytes($icoPath, $memory.ToArray())
    } finally {
        $writer.Dispose()
        $memory.Dispose()
    }
    Write-Host "Wrote $icoPath"
} finally {
    $original.Dispose()
}
