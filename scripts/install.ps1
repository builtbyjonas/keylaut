# Keylaut installer for Windows PowerShell
# https://github.com/builtbyjonas/keylaut

[CmdletBinding()]
param (
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\Keylaut",
    [switch]$NoStartup
)

$ErrorActionPreference = "Stop"

$Repo = "builtbyjonas/keylaut"

# Detect Architecture
$Arch = if ([System.Environment]::Is64BitOperatingSystem) {
    if ($env:PROCESSOR_ARCHITECTURE -match "ARM64") { "aarch64" } else { "x86_64" }
} else {
    Write-Error "Keylaut requires a 64-bit operating system."
    exit 1
}

$ArtifactName = "keylaut-windows-$Arch.zip"

if ($Version -eq "latest") {
    $BaseUrl = "https://github.com/$Repo/releases/latest/download"
} else {
    $BaseUrl = "https://github.com/$Repo/releases/download/$Version"
}

$DownloadUrl = "$BaseUrl/$ArtifactName"
$ChecksumsUrl = "$BaseUrl/SHA256SUMS"

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("keylaut-install-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

try {
    Write-Host "==> Downloading $ArtifactName..." -ForegroundColor Cyan
    $ZipPath = Join-Path $TempDir $ArtifactName
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing

    Write-Host "==> Downloading SHA256SUMS..." -ForegroundColor Cyan
    $SumsPath = Join-Path $TempDir "SHA256SUMS"
    Invoke-WebRequest -Uri $ChecksumsUrl -OutFile $SumsPath -UseBasicParsing

    Write-Host "==> Verifying SHA-256 checksum..." -ForegroundColor Cyan
    $ExpectedLine = Get-Content $SumsPath | Where-Object { $_ -match $ArtifactName }
    if ($ExpectedLine) {
        $ExpectedHash = ($ExpectedLine -split "\s+")[0].Trim()
        $ActualHash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()

        if ($ActualHash -ne $ExpectedHash.ToLower()) {
            Write-Host "`nKeylaut installation failed.`n" -ForegroundColor Red
            Write-Host "The downloaded file did not match the expected checksum." -ForegroundColor Red
            Write-Host "Expected: $ExpectedHash"
            Write-Host "Actual:   $ActualHash"
            Write-Host "Nothing was installed."
            exit 1
        }
        Write-Host "==> Checksum verified successfully." -ForegroundColor Green
    }

    # Extract archive
    $ExtractDir = Join-Path $TempDir "extracted"
    Expand-Archive -Path $ZipPath -DestinationPath $ExtractDir -Force

    # Install executable
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    $ExePath = Join-Path $InstallDir "keylaut.exe"
    Copy-Item -Path (Join-Path $ExtractDir "keylaut.exe") -Destination $ExePath -Force

    Write-Host "==> Keylaut installed to $ExePath" -ForegroundColor Green

    # Add to user PATH if not present
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
        Write-Host "==> Added $InstallDir to user PATH." -ForegroundColor Cyan
    }

    # Configure autostart
    if (-not $NoStartup) {
        Write-Host "==> Configuring automatic login startup..." -ForegroundColor Cyan
        & $ExePath autostart enable
    }

    Write-Host "`nKeylaut is successfully installed!" -ForegroundColor Green
    & $ExePath --version
}
finally {
    if (Test-Path $TempDir) {
        Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
