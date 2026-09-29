# Keylaut uninstaller for Windows PowerShell
# https://github.com/builtbyjonas/keylaut

[CmdletBinding()]
param (
    [string]$InstallDir = "$env:LOCALAPPDATA\Keylaut"
)

$ErrorActionPreference = "SilentlyContinue"

Write-Host "==> Uninstalling Keylaut..." -ForegroundColor Cyan

# Disable autostart
$ExePath = Join-Path $InstallDir "keylaut.exe"
if (Test-Path $ExePath) {
    & $ExePath autostart disable
}

# Remove Startup shortcut if present
$StartupScript = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Startup\Keylaut.cmd"
if (Test-Path $StartupScript) {
    Remove-Item -Path $StartupScript -Force
}

# Remove installation directory
if (Test-Path $InstallDir) {
    Remove-Item -Path $InstallDir -Recurse -Force
    Write-Host "==> Removed $InstallDir" -ForegroundColor Green
}

# Remove from User PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -like "*$InstallDir*") {
    $NewPath = ($UserPath -split ";" | Where-Object { $_ -ne $InstallDir -and $_ -ne "" }) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    Write-Host "==> Removed $InstallDir from user PATH." -ForegroundColor Cyan
}

Write-Host "Keylaut has been successfully uninstalled." -ForegroundColor Green
