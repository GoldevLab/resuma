# Publish Resuma to crates.io (3 crates: macros → runtime → flow UI).
# Usage: .\scripts\publish-crates.ps1 [-DryRun] [-AllowDirty]

param(
    [switch]$DryRun,
    [switch]$AllowDirty
)

$ErrorActionPreference = "Stop"
$crates = @(
    "resuma-macros",
    "resuma",
    "resuma-flow"
)

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$extra = @()
if ($AllowDirty) { $extra += "--allow-dirty" }

for ($i = 0; $i -lt $crates.Count; $i++) {
    $crate = $crates[$i]
    Write-Host ""
    Write-Host "=== $crate ===" -ForegroundColor Cyan
    if ($DryRun) {
        cargo publish -p $crate --dry-run @extra
    } else {
        cargo publish -p $crate @extra
    }
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed on $crate"
    }
    # Wait for crates.io to index before publishing the next dependent crate.
    if (-not $DryRun -and $i -lt ($crates.Count - 1)) {
        Write-Host "Waiting 90s for crates.io index..." -ForegroundColor Yellow
        Start-Sleep -Seconds 90
    }
}

Write-Host ""
Write-Host "Done." -ForegroundColor Green
