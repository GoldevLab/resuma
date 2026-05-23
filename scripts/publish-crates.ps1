# Publish Resuma to crates.io (2 crates: macros first, then runtime).
# Usage: .\scripts\publish-crates.ps1 [-DryRun] [-AllowDirty]

param(
    [switch]$DryRun,
    [switch]$AllowDirty
)

$ErrorActionPreference = "Stop"
$crates = @(
    "resuma-macros",
    "resuma"
)

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$extra = @()
if ($AllowDirty) { $extra += "--allow-dirty" }

foreach ($crate in $crates) {
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
}

Write-Host ""
Write-Host "Done." -ForegroundColor Green
