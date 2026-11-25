# Plumb 3D PowerShell Build Script
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host " Building Plumb 3D Suite (Release Mode)  " -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

cargo build --release --workspace

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ Build completed successfully." -ForegroundColor Green
    Write-Host "Binary available at: target\release\app.exe" -ForegroundColor Green
} else {
    Write-Host "`n❌ Build failed with exit code $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}
