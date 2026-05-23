# Changelog

All notable changes to this project will be documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.1] - 2026-05-23

### Fixed

- `repository` and `homepage` metadata now point to `https://github.com/GolfredoPerezFernandez/resuma`
- All published crates include the workspace `README.md` on crates.io

## [0.1.0] - 2025-05-23

### Added

- Resumable SSR framework: signals, `view!`, `#[component]`, `#[island]`
- Server actions (`#[server]`) with CSRF, rate limits, and security headers
- Resuma Flow: `#[load]`, `#[submit]`, `#[middleware]`, file-based pages
- CLI: `resuma new`, `resuma dev`, `resuma build`, `resuma routes --generate`
- Examples: counter, todo (backend security reference), flow-demo, flow-pages, website
- Documentation site and markdown guides under `docs/`

[0.1.1]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.1.1
[0.1.0]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.1.0
