# Regenerates the `.580` file-type icon set from
# `assets/icons/file-580.png`. Run when the source PNG changes:
#
#   powershell -File scripts/generate_file_icon.ps1
#
# Outputs are checked into the repository so the build does not
# decode/resize the master at compile time.

Add-Type -AssemblyName System.Drawing

$root = Resolve-Path "$PSScriptRoot\.."
$outDir = Join-Path $root 'assets\icons'
$source = Join-Path $outDir 'file-580.png'
$icoPath = Join-Path $outDir 'file-580.ico'

if (-not (Test-Path $source)) {
    throw "Missing source icon: $source"
}

$icoSizes = 16, 20, 24, 32, 40, 48, 64, 96, 128, 256

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
    $pngBytesBySize = @{}
    foreach ($size in ($icoSizes | Sort-Object -Unique)) {
        $bitmap = Resize-Bitmap $original $size
        try {
            $pngBytesBySize[$size] = ConvertTo-PngBytes $bitmap
        } finally {
            $bitmap.Dispose()
        }
    }

    $icoSorted = $icoSizes | Sort-Object -Unique -Descending
    $headerSize = 6 + 16 * $icoSorted.Count

    $memory = New-Object System.IO.MemoryStream
    $writer = New-Object System.IO.BinaryWriter $memory
    try {
        $writer.Write([uint16]0)
        $writer.Write([uint16]1)
        $writer.Write([uint16]$icoSorted.Count)

        $offset = $headerSize
        foreach ($size in $icoSorted) {
            $bytes = $pngBytesBySize[$size]
            $dim = if ($size -ge 256) { [byte]0 } else { [byte]$size }
            $writer.Write($dim)
            $writer.Write($dim)
            $writer.Write([byte]0)
            $writer.Write([byte]0)
            $writer.Write([uint16]1)
            $writer.Write([uint16]32)
            $writer.Write([uint32]$bytes.Length)
            $writer.Write([uint32]$offset)
            $offset += $bytes.Length
        }

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
