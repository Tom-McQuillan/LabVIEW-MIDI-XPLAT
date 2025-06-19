Write-Host "Building TomMIDIllan DLL for LabVIEW..." -ForegroundColor Green
Write-Host ""

# Clean previous builds
Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
cargo clean

# Install targets if not already installed
Write-Host "Installing build targets..." -ForegroundColor Yellow
rustup target add i686-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc

# Build 64-bit version
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building 64-bit version..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
cargo build --release --target x86_64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: 64-bit build failed!" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Build 32-bit version
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building 32-bit version..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
cargo build --release --target i686-pc-windows-msvc
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: 32-bit build failed!" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Create output directory
if (!(Test-Path "dist")) {
    New-Item -ItemType Directory -Path "dist"
}

# Copy and rename DLLs
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Copying and renaming DLLs..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

try {
    Copy-Item "target\x86_64-pc-windows-msvc\release\TomMIDIllan.dll" "dist\TomMIDIllan_64.dll"
    Copy-Item "target\i686-pc-windows-msvc\release\TomMIDIllan.dll" "dist\TomMIDIllan_32.dll"
} catch {
    Write-Host "ERROR: Failed to copy DLLs!" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "Build Complete! 🎹" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Created:" -ForegroundColor White
Write-Host "  dist\TomMIDIllan_32.dll (32-bit for LabVIEW 32-bit)" -ForegroundColor White
Write-Host "  dist\TomMIDIllan_64.dll (64-bit for LabVIEW 64-bit)" -ForegroundColor White
Write-Host ""
Write-Host "File sizes:" -ForegroundColor White
Get-ChildItem "dist\TomMIDIllan_*.dll" | Format-Table Name, Length -AutoSize
Write-Host ""
Write-Host "Ready for LabVIEW integration! 🚀" -ForegroundColor Green
Read-Host "Press Enter to continue"