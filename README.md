# usrp-manager

> **Interactive CLI Manager for Ettus Research USRP Software Defined Radios (SDR)**

`usrp-manager` provides a guided, interactive terminal experience for telecom labs, university students, and SDR researchers. It removes the friction of remembering syntax and flags for utilities like `uhd_find_devices`, `uhd_usrp_probe`, `uhd_image_loader`, and JTAG recovery tools.

---

## 🌟 Key Capabilities

1. **Full CLI Transparency**: Every single executed command is highlighted in bright yellow (`● <cmd> <args>`) with live output, so you can copy and paste commands directly into scripts or other terminals.
2. **System Diagnostic with Multi-Version Switcher**:
   - Detects active and installed UHD driver versions (`UHD-4.5` through `UHD-4.11`).
   - Switch active UHD versions on the spot without leaving the diagnostic view.
   - Detects local host IP subnet configuration and provides copy-paste commands to set static IP (`192.168.10.1`) when needed.
3. **Hardware Inspection ("📡 Probar USRP / Test USRP")**:
   - Replaces manual invocation of `uhd_usrp_probe`.
   - Discovers connected devices over Ethernet/USB, parses daughterboards (RF frontends), supported frequency bands, and serial numbers into clean colored tables.
4. **Permanent FPGA Network Flashing Wizard**:
   - Explains what permanent flashing means and guides users through device detection, model selection (N210, X310, N310, etc.), and image variant selection (`HG`, `XG`).
   - Builds the exact `uhd_image_loader` command for explicit confirmation.
5. **JTAG Unbrick Assistant (Zero-Assumption Guidance)**:
   - Complete physical setup instructions per device (e.g. holding button **S2** on USRP N210 for safe boot; front micro-USB JTAG on X300/X310).
   - Detects Digilent JTAG cables via `djtgcfg`, initialises JTAG chains, loads volatile `.bit` images, and seamlessly transitions to permanent network flashing once recovered.
6. **FPGA Image Downloader & Cache Manager**:
   - Downloads official release images from GitHub with progress bars and SHA-256 verification.
   - Automatically unpacks and organizes bitstream files ready for immediate flashing.
7. **Educational Guides**:
   - 8 interactive educational topics covering SDR concepts, FPGA architectures, RF antenna safety, brick recovery, and compatibility.
   - Bilingual support: English and Spanish.

---

## 🚀 Installation & Running

Download pre-compiled standalone binaries from [GitHub Releases](https://github.com/tcasaniv/usrp-manager/releases) for:
- Windows (`x86_64`)
- Linux (`x86_64` & `aarch64`)
- macOS (`x86_64` & Apple Silicon `aarch64`)

Or compile from source:

```bash
git clone https://github.com/tcasaniv/usrp-manager.git
cd usrp-manager
cargo run --release
```

---

## ⚙️ Requirements

- **UHD**: Recommended for hardware discovery and network flashing (`uhd_find_devices`, `uhd_usrp_probe`, `uhd_image_loader`).
- **Digilent Adept Utilities** (`djtgcfg`): Required only for hardware unbricking via USB JTAG cables.
