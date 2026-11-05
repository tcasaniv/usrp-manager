# usrp-manager

**Interactive CLI tool for managing Ettus Research USRP SDR devices.**

> Unbrick, flash, download FPGA images, install and switch UHD driver versions — guided step-by-step.

---

## ✨ Features

- 🔍 **System Diagnostic** — detects UHD version, djtgcfg, USRP on network, version mismatches
- ⚡ **Unbrick via JTAG** — recovers a bricked USRP X300/X310 using a Digilent JTAG cable (`djtgcfg`)
- 🌐 **Flash FPGA via Network** — permanently writes FPGA image to USRP flash over Ethernet
- 📥 **Manage FPGA Images** — browse GitHub releases, download & cache images locally
- 🛠 **Manage UHD Drivers** — install multiple UHD versions side-by-side, switch active version in PATH
- 📚 **Guide / Education** — built-in explanations for novices (what is USRP, antenna safety, bricking, versions)
- 🌍 **i18n** — English (default) + Spanish; switch from Settings menu
- 💻 **Windows, Linux, macOS** — single binary, no prior tools assumed

---

## 🚀 Download & Run

Download the latest binary for your OS from [Releases](https://github.com/tcasaniv/usrp-manager/releases).

```
# Windows
usrp-manager.exe

# Linux / macOS
chmod +x usrp-manager
./usrp-manager
```

No installation required. Just run and follow the menus.

---

## 🔧 Supported Hardware

| Model | JTAG Unbrick | Network Flash | Notes |
|-------|-------------|---------------|-------|
| USRP X300 | ✅ | ✅ | XC7K325T |
| USRP X310 | ✅ | ✅ | XC7K410T |
| USRP N210 | ❌ (text guide) | Ethernet bootloader | Different process |
| USRP B2xx | ❌ | Via USB/UHD | No JTAG needed |

---

## 📋 Requirements for JTAG Unbricking

- Digilent JTAG-HS2/HS3 cable (or compatible FTDI-based JTAG)
- Digilent Adept 2 Runtime + Utilities installed → https://digilent.com/reference/software/adept/start
- FPGA `.bit` image file for your USRP model/version

---

## 🏗 Build from Source

```bash
git clone https://github.com/tcasaniv/usrp-manager
cd usrp-manager
cargo build --release
./target/release/usrp-manager
```

Requires: Rust 1.70+

---

## 📁 Multi-Version UHD (Windows)

Installed versions go in:
```
C:\Program Files\UHD_versions\
  UHD-4.8\
  UHD-4.10\
  UHD-4.11\
```

Use **Manage UHD Drivers → Switch active version** to update `PATH`.

## 📁 Multi-Version UHD (Linux)

Installed versions go in:
```
/opt/uhd/
  uhd-4.8/
  uhd-4.10/
  uhd-4.11/
```

Registered via `update-alternatives` and `~/.bashrc` env vars.

---

## 📄 License

MIT © [tcasaniv](https://github.com/tcasaniv)
