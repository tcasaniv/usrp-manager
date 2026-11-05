# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-05

### Added

- **Interactive CLI menus** using `dialoguer` for all major workflows
- **JTAG unbrick** flow: auto-detect `djtgcfg`, enumerate devices, program bitstream
- **Network flash** flow: ping device, probe with `uhd_find_devices`, flash over Ethernet
- **FPGA image management**: browse GitHub Releases, download with progress bar, SHA-256 verification, local cache
- **UHD driver management** (multi-version):
  - Windows: NSIS installer download, registry PATH switching via `winreg`
  - Linux: apt/dnf/pacman package managers, CMake build from source, `update-alternatives` integration
  - macOS: Homebrew install, CMake build from source, `~/.zprofile` PATH management
- **System diagnostics**: OS detection, admin/root check, active UHD detection, installed versions listing
- **Educational content**: 8 topics covering USRP hardware, UHD architecture, FPGA/JTAG concepts, RF fundamentals, lab troubleshooting
- **Internationalization (i18n)**: English (default) and Spanish via `rust-i18n v3`; language selectable in Settings
- **Settings menu**: language, images cache directory, UHD versions directory — persisted as JSON in platform config dir
- **Command visibility**: every external command shown with `● cmd args` (yellow) + live output + exit code
- **GitHub Actions CI**: Build & Clippy matrix on ubuntu-latest / windows-latest / macos-latest
- **GitHub Actions Release**: cross-compiled binaries for 5 targets on `v*` tag push:
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`

### Dependencies

- `dialoguer 0.11` with fuzzy-select feature
- `rust-i18n 3`
- `reqwest 0.12` (blocking + json)
- `indicatif 0.17` (progress bars)
- `console 0.15` (colored output)
- `zip 2`, `sha2 0.10`, `hex 0.4` (image verification)
- `which 6`, `dirs 5`, `anyhow 1`, `thiserror 2`
- `winreg 0.52` (Windows only)
- `libc 0.2` (Linux/macOS only)

[Unreleased]: https://github.com/tcasaniv/usrp-manager/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tcasaniv/usrp-manager/releases/tag/v0.1.0
