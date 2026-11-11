# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.4] - 2026-11-11

### Added
- **Intelligent RAM-Aware Concurrency Control**: Fixed catastrophic WSL/Linux crashes during UHD source compilation caused by OOM when running `make -j <all_cores>`. Workers are now automatically capped via `min(cores, ram_gb / 2)`, preventing kernel panic and WSL VM crashes.
- **Persistent Storage Compilation**: Moved build root from `/tmp` (which is RAM-backed `tmpfs` on Linux/WSL) to `/var/tmp` / user cache on disk, avoiding RAM exhaustion and permissions collisions.
- **Clear Installation Guidance**: Highlighted package manager (`apt`) installation as the recommended fast method (~10 seconds) versus source compilation (~30-60 min).

---

## [0.2.3] - 2026-11-11

### Fixed
- **Source Build Working Directory**: Fixed `cmake`, `make`, and `sudo make install` execution directory in Linux and macOS by introducing `run_visible_in_dir`. Previously, `cmake ..` executed in the user's current directory instead of `uhd-src/host/build`, resulting in CMake looking for `CMakeLists.txt` in `/home`.

---

## [0.2.2] - 2026-11-10

### Added
- **Linux Build Prerequisites Auto-Detection & Guided Installation**: Before compiling UHD from source on Linux/Ubuntu, `usrp-manager` now detects missing build dependencies (`cmake`, `make`, `g++`, `libboost-all-dev`, `libusb-1.0-0-dev`, `python3-mako`, etc.) and offers to install them automatically with one click instead of failing with an `os error 2`.

### Fixed
- **Probe Menu Icon & Localization**: Replaced broken fallback glyphs (`??`) with proper satellite emoji `📡` and integrated complete Spanish (`es.yml`) and English (`en.yml`) translations for all probe strings (`probe.title`, `probe.intro`, `probe.no_uhd`, `probe.scanning`, etc.).
- **Linux Binary Execution Documentation**: Clarified standard Linux binary execution (`./usrp-manager-linux-x86_64`) vs shell script invocation (`bash ./...`).

---

## [0.2.1] - 2026-11-07

### Added
- **Zero-Assumption Guided Flashing Wizard**: Step-by-step interactive assistant for permanent network flashing (`uhd_image_loader`). Features clear explanations of volatile RAM vs permanent flash, device discovery, model selection (N210, X300/X310, N300/N310, E310), FPGA image variant explanations (`HG` 1G/10G vs `XG` dual 10G), explicit command preview (`● <cmd>`), and confirmation.
- **Physical Guidance for JTAG Recovery**: Contextual unbrick assistant explaining safe boot modes per hardware model, including the internal **S2 recovery button** for USRP N200/N210, JTAG pinouts, and front-panel micro-USB JTAG connections on USRP X300/X310.
- **Automatic Image Extraction Flattening**: Top-level directory redundancy in GitHub zip packages is now automatically stripped upon extraction, placing all `.bit`/`.bin` files directly in `fpga-images/uhd-images_<version>/`.
- **Educational Onboarding**: Plain-language explanations in Spanish and English for what FPGAs do, why images differ, and how antenna RF reflections can damage hardware without proper loads.

### Fixed
- **Menu Index Navigation**: Fixed off-by-one and mismatch index issues in `images_menu` (where *Back* previously showed local images and *View Images* went back) and `settings_menu` (restoring proper alignment for directory selectors and back option).
- **Subdirectory Duplication**: Resolved nested folders during FPGA image downloads (`uhd-images_x.y.z/uhd-images_x.y.z`).
- **Comprehensive Localization**: Cleaned and validated UTF-8 locales in English and Spanish ensuring no unlocalized keys or broken encoding appear across probe, diagnostic, or education views.

---

## [0.2.0] - 2026-11-06

### Added
- **Multi-version UHD Detection & Quick Switcher**: System Diagnostic now scans `config.uhd_versions_dir` (e.g. `C:\Program Files\UHD_versions` or `/opt/uhd/`), displays all detected versions with active marks, and prompts the user to switch the active UHD version directly on the spot.
- **Probe USRP Menu ("📡 Probar USRP / Test USRP")**: Dedicated interactive menu replacing manual runs of `uhd_find_devices` and `uhd_usrp_probe`. Automatically detects connected devices, allows manual IP/args input (`addr=192.168.10.2`), prints the exact command executed, and summarizes daughterboards, frequencies, channels, and serial numbers with optional full raw output display.
- **Host Network NIC Assistant**: In Diagnostic, if no USRP is detected over Ethernet, actionable network hints and exact commands (`netsh` on Windows, `ip addr` on Linux) are presented to guide setting the host PC to `192.168.10.1`.

### Fixed
- **FPGA Images Multi-line SHA-256 Verification**: Fixed checksum validation failure on GitHub release assets where `.sha256` files contain entries for `.tar.gz`, `.tar.xz`, and `.zip`. The parser now searches for the exact asset filename.
- **Full Command Visibility**: All operations print the exact command and arguments using `● <cmd>` in bright yellow before execution, enabling users to copy and paste them elsewhere.

---

## [0.1.0] - 2026-11-05

### Added
- **Interactive CLI menus** using `dialoguer` for all major workflows.
- **JTAG unbrick & Network flash flows**.
- **Multi-version UHD management** on Windows, Linux and macOS.
- **Bilingual i18n support** (English and Spanish).
- **GitHub Actions matrix CI/CD** with standalone binary releases for 5 targets.

[Unreleased]: https://github.com/tcasaniv/usrp-manager/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/tcasaniv/usrp-manager/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/tcasaniv/usrp-manager/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/tcasaniv/usrp-manager/releases/tag/v0.1.0
