<div align="center">

<img src="wechat-rs.ico" width="128" />

# WeChat Tweak RS

**A cross-platform WeChat toolkit written in pure Rust.**

Multi-instance · Anti-revoke · Auto-update bypass

[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue)](#-platform-support)
[![Language](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-green)](LICENSE)

</div>

---

## ✨ Features

| Feature | macOS | Windows |
|---------|:-----:|:-------:|
| 🔓 Multi-instance | ✅ Binary Patch | ✅ Mutex Kill |
| 🛡️ Anti-revoke (message recall block) | ✅ | — |
| 🚫 Disable auto-update | ✅ | — |
| 🔏 Auto re-sign (ad-hoc codesign) | ✅ | — |

## 🏗️ How It Works

### macOS — Binary Patching

Directly patches the WeChat Mach-O binary to modify specific functions:

- **Multi-instance**: Patches the single-instance check to `return true` (`mov w0, #1; ret`)
- **Anti-revoke**: Patches the message revoke handler to `return false` (`mov w0, #0; ret`)
- **Disable updates**: Neutralizes all auto-update related functions

Supports both FAT (universal) and thin Mach-O binaries. Automatically re-signs with ad-hoc signature after patching.

Patch configs are version-specific and embedded in the binary. Currently supports WeChat versions: `31927`, `31960`, `32281`, `32288`, `34371`.

### Windows — Runtime Mutex

Finds the running WeChat process, locates the `_WeChat_App_Instance_Identity_Mutex_Name` mutex handle via `NtQuerySystemInformation`, and closes it. Then launches a new WeChat instance.

## 📦 Install

### From Source

```bash
# Install Rust: https://rustup.rs
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/jiusanzhou/multi-wechat-rs.git
cd multi-wechat-rs
cargo build --release
```

### From Cargo

```bash
cargo install multi-wechat-rs
```

### Pre-built Binaries

Download from [Releases](https://github.com/jiusanzhou/multi-wechat-rs/releases).

## 🚀 Usage

### macOS

```bash
# Patch WeChat (multi-instance + anti-revoke + disable updates)
multi-wechat-rs patch

# Specify custom app path
multi-wechat-rs --app /Applications/WeChat.app patch

# Use external config (for new WeChat versions)
multi-wechat-rs --config ./config.json patch

# List current and supported versions
multi-wechat-rs versions
```

### Windows

Double-click to run. It will automatically close the mutex lock and launch a new WeChat instance.

## 🔧 Config Format

Patch configs can be embedded or loaded externally. Format:

```json
[
  {
    "version": "34371",
    "targets": [
      {
        "identifier": "multiInstance",
        "entries": [
          { "arch": "arm64", "addr": "1001b82c4", "asm": "20008052C0035FD6" }
        ]
      },
      {
        "identifier": "revoke",
        "entries": [
          { "arch": "arm64", "addr": "103e7cd2c", "asm": "00008052C0035FD6" }
        ]
      }
    ]
  }
]
```

To add support for a new WeChat version, reverse-engineer the target function addresses and add a new entry.

## 🏛️ Architecture

```
src/
├── main.rs              # Entry point (platform-dispatched)
├── macos/
│   ├── mod.rs           # Module root + error types
│   ├── config.rs        # Patch config loading (embedded + external)
│   ├── config.json      # Embedded WeChatTweak configs
│   ├── macho.rs         # Mach-O parser + binary patcher
│   └── patcher.rs       # Orchestrator: version → config → patch → codesign
├── process.rs           # [Windows] Process enumeration
├── system.rs            # [Windows] System handle query (NtQuerySystemInformation)
├── utils.rs             # [Windows] Registry, privileges, process creation
└── winapi.rs            # [Windows] Win32 API bindings
```

## 📝 Credits

- macOS binary patching approach inspired by [WeChatTweak](https://github.com/sunnyyoung/WeChatTweak)
- Windows DLL injection toolkit: [injrs](https://github.com/jiusanzhou/injrs)

## ❤️ Support

<img width="200" src="https://payone.wencai.app/s/zoe.png" alt="Support via payone.wencai.app">

*Support via [payone.wencai.app](https://payone.wencai.app)*

## 📄 License

[Apache-2.0](LICENSE)
