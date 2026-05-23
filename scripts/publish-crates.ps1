# Publish Resuma workspace crates to crates.io in dependency order.
# Usage: .\scripts\publish-crates.ps1 [-DryRun]

param(
    [switch]$DryRun,
    [switch]$AllowDirty
)

$ErrorActionPreference = "Stop"
$crates = @(
    "resuma-rs2js",
    "resuma-core",
    "resuma-macros",
    "resuma-ssr",
    "resuma-router",
    "resuma-server",
    "resuma-flow",
    "resuma-cli",
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
