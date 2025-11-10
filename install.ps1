<#
.SYNOPSIS
    Installer script for rusty-clipboard on Windows.

.DESCRIPTION
    Builds the clipd and clipctl binaries, copies them to a per-user install
    directory, sets up environment variables, and configures PowerShell to
    start the daemon automatically and launch clipctl with F12.
#>
[CmdletBinding()]
param(
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

function Write-Section {
    param(
        [Parameter(Mandatory = $true)][string]$Message
    )
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Remove-BlockByMarkers {
    param(
        [string]$Text,
        [string]$StartMarker,
        [string]$EndMarker
    )

    if ([string]::IsNullOrEmpty($Text)) {
        return ''
    }

    $lines = $Text -split "`r?`n"
    $result = New-Object System.Collections.Generic.List[string]
    $insideBlock = $false

    foreach ($line in $lines) {
        if ($insideBlock) {
            if ($line.Trim() -eq $EndMarker.Trim()) {
                $insideBlock = $false
            }
            continue
        }

        if ($line.Trim() -eq $StartMarker.Trim()) {
            $insideBlock = $true
            continue
        }

        $result.Add($line)
    }

    return ($result -join "`r`n")
}

$markerStart = '# >>> rusty-clipboard integration (managed)'
$markerEnd = '# <<< rusty-clipboard integration (managed)'

function Remove-IntegrationBlock {
    param(
        [string]$Content
    )

    if ([string]::IsNullOrEmpty($Content)) {
        return ''
    }

    $cleaned = Remove-BlockByMarkers -Text $Content -StartMarker $markerStart -EndMarker $markerEnd
    $cleaned = Remove-BlockByMarkers -Text $cleaned -StartMarker '$markerStart' -EndMarker '$markerEnd'

    return $cleaned.TrimEnd()
}

$workspaceRoot = (Resolve-Path -Path $PSScriptRoot).ProviderPath
$releaseDir = Join-Path $workspaceRoot 'target\release'
$installDir = Join-Path $env:LOCALAPPDATA 'Programs\rusty-clipboard'
$clipdExe = Join-Path $releaseDir 'clipd.exe'
$clipctlExe = Join-Path $releaseDir 'clipctl.exe'

# Ensure cargo is available
Write-Section "Checking Rust toolchain"
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo was not found on PATH. Install the Rust toolchain before running this installer."
}

if (-not $SkipBuild) {
    Write-Section "Building release binaries (clipd, clipctl)"
    Push-Location $workspaceRoot
    try {
        cargo build --release --bin clipd --bin clipctl
    }
    finally {
        Pop-Location
    }
}
else {
    Write-Section "Skipping build (per --SkipBuild)"
}

if (-not (Test-Path $clipdExe) -or -not (Test-Path $clipctlExe)) {
    throw "Release binaries were not found in $releaseDir. Ensure the build succeeded."
}

Write-Section "Copying binaries to $installDir"
New-Item -Path $installDir -ItemType Directory -Force | Out-Null

$existingClipd = Get-Process -Name 'clipd' -ErrorAction SilentlyContinue | Where-Object {
    $_.Path -eq (Join-Path $installDir 'clipd.exe')
}
if ($existingClipd) {
    Write-Host "Stopping running clipd instance to update binaries..."
    $existingClipd | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
}

Copy-Item -Path $clipdExe -Destination (Join-Path $installDir 'clipd.exe') -Force
Copy-Item -Path $clipctlExe -Destination (Join-Path $installDir 'clipctl.exe') -Force

Write-Section "Setting environment variables"
[Environment]::SetEnvironmentVariable('RUSTY_CLIPBOARD_HOME', $installDir, 'User')
$env:RUSTY_CLIPBOARD_HOME = $installDir

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$pathSeparator = ';'
$pathParts = @()
if ([string]::IsNullOrEmpty($userPath)) {
    $pathParts = @()
}
else {
    $pathParts = $userPath.Split($pathSeparator, [System.StringSplitOptions]::RemoveEmptyEntries)
}

if (-not ($pathParts | Where-Object { $_.TrimEnd('\') -ieq $installDir.TrimEnd('\') })) {
    $pathParts += $installDir
    $newPath = ($pathParts -join $pathSeparator)
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    $env:Path = "$env:Path$pathSeparator$installDir"
    Write-Host "Added $installDir to user PATH."
}
else {
    Write-Host "$installDir already present in user PATH."
}

Write-Section "Updating PowerShell profile for keybinding and daemon autostart"
$profilePath = $PROFILE
$profileDir = Split-Path -Path $profilePath -Parent
if (-not (Test-Path $profileDir)) {
    New-Item -Path $profileDir -ItemType Directory -Force | Out-Null
}

$existingProfile = ''
if (Test-Path $profilePath) {
    $existingProfile = Get-Content -Path $profilePath -Raw
}

$existingProfile = Remove-IntegrationBlock -Content $existingProfile

$allHostsProfilePath = $PROFILE.CurrentUserAllHosts
if ($allHostsProfilePath -and ($allHostsProfilePath -ne $profilePath) -and (Test-Path $allHostsProfilePath)) {
    $allHostsContent = Get-Content -Path $allHostsProfilePath -Raw
    $cleanedAllHosts = Remove-IntegrationBlock -Content $allHostsContent
    if ($cleanedAllHosts -ne $allHostsContent) {
        if ([string]::IsNullOrWhiteSpace($cleanedAllHosts)) {
            Remove-Item -Path $allHostsProfilePath -Force
        }
        else {
            Set-Content -Path $allHostsProfilePath -Value $cleanedAllHosts -Encoding UTF8
        }
    }
}

$integrationBlock = @'
# >>> rusty-clipboard integration (managed)
Import-Module PSReadLine -ErrorAction SilentlyContinue

function Start-RustyClipboardDaemon {
    param(
        [string]$InstallDir = $env:RUSTY_CLIPBOARD_HOME
    )

    if (-not $InstallDir) { return }

    $clipdPath = Join-Path $InstallDir 'clipd.exe'
    if (-not (Test-Path $clipdPath)) { return }

    try {
        $running = Get-Process -Name 'clipd' -ErrorAction SilentlyContinue | Where-Object {
            $_.Path -eq $clipdPath
        }
    }
    catch {
        $running = $null
    }

    if (-not $running) {
        Start-Process -FilePath $clipdPath -WindowStyle Hidden | Out-Null
    }
}

Start-RustyClipboardDaemon

if (Get-Command Set-PSReadLineKeyHandler -ErrorAction SilentlyContinue) {
    Set-PSReadLineKeyHandler -Key F12 -BriefDescription 'Launch clipctl' -ScriptBlock {
        param($key, $arg)

        # Clear the current line
        [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
        [Microsoft.PowerShell.PSConsoleReadLine]::Insert("")

        $clipctlPath = Join-Path $env:RUSTY_CLIPBOARD_HOME 'clipctl.exe'
        if (Test-Path $clipctlPath) {
            try {
                # Use Start-Process with -NoNewWindow and -Wait to properly hand off terminal control
                $process = Start-Process -FilePath $clipctlPath `
                    -NoNewWindow `
                    -Wait `
                    -PassThru `
                    -ErrorAction Stop
                
                if ($process.ExitCode -ne 0) {
                    Write-Host "`nclipctl exited with code: $($process.ExitCode)" -ForegroundColor Yellow
                    Write-Host "Check if clipd daemon is running: Get-Process clipd" -ForegroundColor Cyan
                    Start-Sleep -Seconds 2
                }
            }
            catch {
                Write-Host "`nError launching clipctl: $_" -ForegroundColor Red
                Start-Sleep -Seconds 2
            }
            finally {
                [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt()
            }
        }
        else {
            Write-Host "clipctl.exe not found in $env:RUSTY_CLIPBOARD_HOME." -ForegroundColor Yellow
            [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt()
        }
    }
}
# <<< rusty-clipboard integration (managed)
'@

if (-not [string]::IsNullOrEmpty($existingProfile)) {
    $finalProfile = $existingProfile + "`r`n`r`n" + $integrationBlock
}
else {
    $finalProfile = $integrationBlock
}

Set-Content -Path $profilePath -Value $finalProfile -Encoding UTF8

Write-Section "Starting clipd daemon"
$installedClipd = Join-Path $installDir 'clipd.exe'
if (Test-Path $installedClipd) {
    $existingProcess = Get-Process -Name 'clipd' -ErrorAction SilentlyContinue | Where-Object {
        $_.Path -eq $installedClipd
    }
    if (-not $existingProcess) {
        $startedProcess = Start-Process -FilePath $installedClipd -WindowStyle Hidden -PassThru -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
        $runningNow = $null
        if ($startedProcess) {
            try {
                $runningNow = Get-Process -Id $startedProcess.Id -ErrorAction SilentlyContinue
            }
            catch {
                $runningNow = $null
            }
        }

        if (-not $runningNow) {
            $manualCommand = "& `"$installedClipd`""
            Write-Warning "clipd failed to stay running. Try launching manually: $manualCommand to view the error."
        }
        else {
            Write-Host "clipd started in the background."
        }
    }
    else {
        Write-Host "clipd is already running."
    }
}
else {
    Write-Warning "clipd.exe not found at $installedClipd. Skipping auto-start."
}

Write-Section "Installation complete"
Write-Host "rusty-clipboard binaries installed to: $installDir"
Write-Host "RUSTY_CLIPBOARD_HOME set and PATH updated (per-user)."
Write-Host "PowerShell profile updated to auto-start clipd and launch clipctl with F12."
Write-Host ""
Write-Host "Restart any open PowerShell sessions to pick up profile and PATH changes." -ForegroundColor Yellow

