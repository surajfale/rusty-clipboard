# Development launcher for rusty-clipboard
# Starts clipd daemon in background and clipctl TUI in foreground

Write-Host "Starting rusty-clipboard development environment..." -ForegroundColor Cyan
Write-Host ""

# Start clipd daemon in background
Write-Host "Starting clipd daemon..." -ForegroundColor Yellow
$clipdJob = Start-Job -ScriptBlock {
    Set-Location $using:PWD
    cargo run --bin clipd 2>&1
}

# Wait a moment for clipd to initialize
Start-Sleep -Seconds 2

# Check if clipd started successfully
$clipdState = Get-Job -Id $clipdJob.Id | Select-Object -ExpandProperty State
if ($clipdState -ne "Running") {
    Write-Host "Warning: clipd may not have started correctly. Check job output:" -ForegroundColor Yellow
    Receive-Job -Id $clipdJob.Id
}

Write-Host "Starting clipctl TUI..." -ForegroundColor Yellow
Write-Host ""

# Start clipctl in foreground (this will block until user exits)
try {
    cargo run --bin clipctl
}
finally {
    # Clean up: stop clipd when clipctl exits
    Write-Host ""
    Write-Host "Stopping clipd daemon..." -ForegroundColor Yellow
    Stop-Job -Id $clipdJob.Id -ErrorAction SilentlyContinue
    Remove-Job -Id $clipdJob.Id -Force -ErrorAction SilentlyContinue
    Write-Host "Development environment stopped." -ForegroundColor Cyan
}

