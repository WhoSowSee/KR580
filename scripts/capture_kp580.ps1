# Auto-launch the original KP580 emulator, load a .580 image, run it,
# and screenshot the monitor window. Used to gather a reference image
# of the original colour palette for comparison with our reimplementation.
#
# Usage:
#   pwsh -NoProfile -File scripts/capture_kp580.ps1 -SnapshotPath <abs-path-to-580-file> -OutPng <abs-path-to-output-png>

param(
    [Parameter(Mandatory = $true)] [string] $SnapshotPath,
    [Parameter(Mandatory = $true)] [string] $OutPng,
    [string] $EmulatorExe = "D:\Эмулятор КР580\KP580.exe",
    [int]    $RunSeconds = 5
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

# WinAPI imports for raw window enumeration + screenshotting.
$winApi = @"
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Text;

public static class WinApi {
    [DllImport("user32.dll")]
    public static extern IntPtr FindWindow(string className, string windowName);

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    public static extern bool GetClientRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    public static extern bool ClientToScreen(IntPtr hWnd, ref POINT lpPoint);

    [DllImport("user32.dll")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int count);

    [DllImport("user32.dll")]
    public static extern int GetWindowTextLength(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool EnumChildWindows(IntPtr hWndParent, EnumWindowsProc enumProc, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);

    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct POINT {
        public int X;
        public int Y;
    }
}
"@
Add-Type -TypeDefinition $winApi -Language CSharp -ReferencedAssemblies 'System.Drawing','System.Windows.Forms'

function Get-ProcessWindows {
    param([int] $ProcessId)
    $list = New-Object System.Collections.Generic.List[IntPtr]
    $proc = {
        param($hWnd, $lParam)
        $procId = 0
        [void][WinApi]::GetWindowThreadProcessId($hWnd, [ref]$procId)
        if ($procId -eq $ProcessId -and [WinApi]::IsWindowVisible($hWnd)) {
            $list.Add($hWnd) | Out-Null
        }
        return $true
    }
    [void][WinApi]::EnumWindows($proc, [IntPtr]::Zero)
    return $list
}

function Get-WindowText {
    param([IntPtr] $Hwnd)
    $len = [WinApi]::GetWindowTextLength($Hwnd)
    if ($len -le 0) { return "" }
    $sb = New-Object System.Text.StringBuilder($len + 1)
    [void][WinApi]::GetWindowText($Hwnd, $sb, $sb.Capacity)
    return $sb.ToString()
}

function Take-WindowScreenshot {
    param([IntPtr] $Hwnd, [string] $OutPath)
    $rect = New-Object WinApi+RECT
    [void][WinApi]::GetWindowRect($Hwnd, [ref]$rect)
    $w = $rect.Right - $rect.Left
    $h = $rect.Bottom - $rect.Top
    if ($w -le 0 -or $h -le 0) { throw "Window has zero size" }
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($rect.Left, $rect.Top, 0, 0, (New-Object System.Drawing.Size $w, $h))
    $g.Dispose()
    $bmp.Save($OutPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

if (-not (Test-Path -LiteralPath $SnapshotPath)) {
    throw "Snapshot not found: $SnapshotPath"
}
if (-not (Test-Path -LiteralPath $EmulatorExe)) {
    throw "Emulator not found: $EmulatorExe"
}

# Kill leftover emulator process if any.
Get-Process -Name "KP580" -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "Killing stale KP580 PID $($_.Id)"
    Stop-Process -Id $_.Id -Force
    Start-Sleep -Milliseconds 500
}

$exeDir = Split-Path -Parent $EmulatorExe
Write-Host "Launching $EmulatorExe"
$proc = Start-Process -FilePath $EmulatorExe -WorkingDirectory $exeDir -PassThru
Start-Sleep -Seconds 3
$proc.Refresh()

if ($proc.HasExited) { throw "Emulator exited immediately." }

# Find the main window of the emulator.
$windows = Get-ProcessWindows -ProcessId $proc.Id
if ($windows.Count -eq 0) { throw "No emulator windows found." }
$mainHwnd = $windows[0]
foreach ($w in $windows) {
    $title = Get-WindowText -Hwnd $w
    Write-Host "  window 0x$($w.ToString('X')) title='$title'"
}

# Bring it forward.
[void][WinApi]::ShowWindow($mainHwnd, 9)  # SW_RESTORE
Start-Sleep -Milliseconds 400
[void][WinApi]::SetForegroundWindow($mainHwnd)
Start-Sleep -Milliseconds 400

# File menu → Open... (Alt+Ф then О).
Write-Host "Sending Alt+F (Файл) → О (Открыть)"
[System.Windows.Forms.SendKeys]::SendWait("%{F}")
Start-Sleep -Milliseconds 600
[System.Windows.Forms.SendKeys]::SendWait("o")
Start-Sleep -Seconds 1

# Type the absolute snapshot path into the Open dialog and Enter.
$pathToType = $SnapshotPath -replace '\+', '{+}' -replace '\^', '{^}' -replace '%', '{%}' -replace '~', '{~}' -replace '\(', '{(}' -replace '\)', '{)}'
[System.Windows.Forms.SendKeys]::SendWait($pathToType)
Start-Sleep -Milliseconds 400
[System.Windows.Forms.SendKeys]::SendWait("{ENTER}")
Start-Sleep -Seconds 2

# Bring main window forward again.
[void][WinApi]::SetForegroundWindow($mainHwnd)
Start-Sleep -Milliseconds 300

# MPS menu → Run program (Alt+М then В — there are several "В…" items;
# 'Выполнить программу' is the third one. Use down-arrow to be safe.)
Write-Host "Sending Alt+M (МП-система) → run program"
[System.Windows.Forms.SendKeys]::SendWait("%{M}")
Start-Sleep -Milliseconds 600
# Press DOWN three times to reach 'Выполнить программу', then ENTER.
[System.Windows.Forms.SendKeys]::SendWait("{DOWN}{DOWN}{DOWN}{ENTER}")
Start-Sleep -Seconds $RunSeconds

# Snapshot all windows of the process — main window + monitor child if open.
$allWindows = Get-ProcessWindows -ProcessId $proc.Id
$index = 0
foreach ($w in $allWindows) {
    $title = Get-WindowText -Hwnd $w
    $rect  = New-Object WinApi+RECT
    [void][WinApi]::GetWindowRect($w, [ref]$rect)
    $size  = "$($rect.Right - $rect.Left)x$($rect.Bottom - $rect.Top)"
    $tag = "win$index"
    $perPath = $OutPng -replace '\.png$', "_$tag.png"
    try {
        Take-WindowScreenshot -Hwnd $w -OutPath $perPath
        Write-Host "  saved $perPath ($size, title='$title')"
    } catch {
        Write-Warning "  failed to capture $w : $_"
    }
    $index++
}

# Always also save the main window under the requested OutPng.
Take-WindowScreenshot -Hwnd $mainHwnd -OutPath $OutPng
Write-Host "Saved main-window screenshot to $OutPng"

# Leave the emulator running so we can inspect it manually if needed.
Write-Host "Done. Emulator PID $($proc.Id) left running."
