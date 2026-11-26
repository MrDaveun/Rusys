<div align="center">
  <img src="assets/rusys-logo.webp" alt="Rusys Animated Logo" width="480" />

  <h1>Rusys</h1>

  <p>
    <strong>A lightweight, real-time Rust terminal system monitor built with
    <a href="https://crates.io/crates/ratatui">Ratatui</a>,
    <a href="https://crates.io/crates/crossterm">Crossterm</a>, and
    <a href="https://crates.io/crates/sysinfo">Sysinfo</a>.</strong>
  </p>

  <p>
    <img src="https://img.shields.io/badge/Rust-2024%20Edition-orange?style=for-the-badge&logo=rust" alt="Rust 2024 Edition" />
    <img src="https://img.shields.io/badge/Version-v0.3.0-00bcd4?style=for-the-badge" alt="Version v0.3.0" />
    <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="MIT License" />
    <img src="https://img.shields.io/badge/Platforms-Linux%20%7C%20Windows%20%7C%20macOS-blue?style=for-the-badge" alt="Cross-Platform" />
  </p>

  <p>
    <a href="#overview">Overview</a> •
    <a href="#collected-information--usage">Metrics</a> •
    <a href="#controls">Controls</a> •
    <a href="#project-structure">Structure</a> •
    <a href="#releases">Releases</a> •
    <a href="#building-from-source">Build</a>
  </p>
</div>

---

## Overview

**Rusys** inspects your local machine and renders live operating system, hardware, per-core CPU, memory/swap, storage, network, and process metrics inside an interactive terminal dashboard that refreshes every second.

- **Real-time & Local-only**: All telemetry is read directly from your OS without logging, disk writes, or network transmission.
- **Persistent Collectors**: Tracks continuous CPU deltas, per-core frequencies, Swap usage, process activity, and live network throughput (`Down/s` and `Up/s`).
- **Responsive TUI**: Automatically adapts layout between wide and compact terminal windows.

## Collected Information & Usage

All metrics are read locally in real time and used **exclusively for on-screen display** inside the terminal interface. Nothing is written to disk, logged, or transmitted over any network.

| Module | Collected Metrics | How It Is Used |
| :--- | :--- | :--- |
| **System** (`src/system/mod.rs`) | OS name, CPU architecture, OS family, hostname, OS version, kernel version, distribution ID, and system uptime | Shown in the top status bar (hostname and OS) and the **System** panel with human-readable uptime formatting |
| **Hardware** (`src/hardware/mod.rs`) | CPU brand/model, vendor ID, clock frequency (MHz), logical core count, global & per-core CPU usage (%) and frequency, total/used/available RAM, total/used/free Swap memory, and mounted disk/storage volumes | Shown in the **Hardware** panel (CPU, RAM, and Swap gauges), the scrollable **CPU Cores** panel, and the **Storage** table |
| **Processes** (`src/process/mod.rs`) | Running process PID, name, CPU usage (%), resident memory, virtual memory, and execution state | Shown in the scrollable **Processes** table with live sorting by `CPU%`, `MEM`, or `PID` and a detail strip for the selected process |
| **Network** (`src/network/mod.rs`) | Network interface names, MAC addresses, IPv4/IPv6 addresses, MTU, live download/upload speeds, cumulative & delta RX/TX bytes, packet counts, and error counters | Shown in the scrollable **Network** table (`Down/s`, `Up/s`, `RX Total`, `TX Total`) with a live detail strip for the selected interface |

## Controls

| Key | Action |
| :--- | :--- |
| **`q` / `Esc`** | Quit the application and restore the terminal state |
| **`r`** | Trigger an immediate manual refresh |
| **`s`** | Cycle process list sorting (`CPU%` → `MEM` → `PID`) |
| **`Tab`** | Switch active focus between **Processes**, **CPU Cores**, **Storage**, and **Network** |
| **`↑` / `↓`** or **`j` / `k`** | Navigate rows in the active panel |

## Project Structure

```text
Rusys/
├── .github/
│   └── workflows/
│       ├── ci.yml          # Cross-platform check, test, and build workflow
│       └── release.yml     # Multi-platform release executable packaging
├── assets/
│   └── rusys-logo.webp     # Animated Rusys logo for documentation
├── Cargo.toml
├── LICENSE
├── README.md
└── src/
    ├── main.rs             # Terminal initialization, 1s refresh loop, and key handling
    ├── ui/
    │   └── mod.rs          # Ratatui layout, widgets, formatters, and unit tests
    ├── system/
    │   └── mod.rs          # OS and host metadata collection and unit tests
    ├── hardware/
    │   └── mod.rs          # Persistent global/per-core CPU, RAM, Swap, and disk collection
    ├── process/
    │   └── mod.rs          # Persistent process monitor (PID, CPU%, memory, state, sorting)
    └── network/
        └── mod.rs          # Persistent network collector (IPs, MTU, speeds, totals, packets, errors)
```

## Releases

Pre-built release binaries are generated automatically by `.github/workflows/release.yml` for version tags (`v*`) and manual workflow dispatches across the following platforms and architectures:

| Platform | Architecture | Target Triple | Artifact |
| :--- | :--- | :--- | :--- |
| **Linux** | x86_64 (64-bit) | `x86_64-unknown-linux-gnu` | `rusys-linux-x86_64.tar.gz` |
| **Windows** | x86_64 (64-bit) | `x86_64-pc-windows-msvc` | `rusys-windows-x86_64.zip` |
| **macOS** | Apple Silicon (ARM64) | `aarch64-apple-darwin` | `rusys-macos-arm64.tar.gz` |
| **macOS** | Intel (x86_64) | `x86_64-apple-darwin` | `rusys-macos-x86_64.tar.gz` |

Download the archive for your platform from the [Releases](../../releases) page, extract the binary, and run `./rusys` (or `rusys.exe` on Windows).

## Building from Source

Requires Rust and Cargo:

```bash
# Run the interactive TUI
cargo run

# Verify compilation and run unit tests
cargo check --all-targets
cargo test

# Build an optimized release binary
cargo build --release
```

## License

This project is licensed under the [MIT License](LICENSE).
