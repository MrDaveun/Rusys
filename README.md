# Rusys

A lightweight **Rust** terminal user interface (TUI) system monitor built with [`ratatui`](https://crates.io/crates/ratatui), [`crossterm`](https://crates.io/crates/crossterm), and [`sysinfo`](https://crates.io/crates/sysinfo).

## Overview

Rusys inspects your local machine and renders live operating system, hardware, and network metrics inside an interactive terminal dashboard that refreshes every second.

## Collected Information & Usage

All metrics are read locally in real time and used **exclusively for on-screen display** inside the terminal interface. Nothing is written to disk, logged, or transmitted over any network.

| Module | Collected Metrics | How It Is Used |
| :--- | :--- | :--- |
| **System** (`src/system/mod.rs`) | OS name, CPU architecture, OS family, hostname, OS version, kernel version, distribution ID, and system uptime | Shown in the top status bar (hostname and OS) and the **System** panel with human-readable uptime formatting |
| **Hardware** (`src/hardware/mod.rs`) | CPU brand/model, vendor ID, clock frequency (MHz), logical core count, and total / used / available RAM | Shown in the **Hardware** panel with CPU specs and a live **RAM Usage** progress gauge in KB/MB/GB |
| **Network** (`src/network/mod.rs`) | Network interface names, MAC addresses, and cumulative received (`RX`) / transmitted (`TX`) byte counters | Shown in the navigable **Network** table with formatted byte totals |

## Controls

- **`q` / `Esc`**: Quit the application and restore the terminal state
- **`r`**: Trigger an immediate manual refresh
- **`↑` / `↓`** or **`j` / `k`**: Navigate rows in the network interfaces table

## Project Structure

```text
Rusys/
├── .github/
│   └── workflows/
│       ├── ci.yml       # Cross-platform check, test, and build workflow
│       └── release.yml  # Multi-platform release executable packaging
├── Cargo.toml
├── LICENSE
├── README.md
└── src/
    ├── main.rs          # Terminal initialization, 1s refresh loop, and key handling
    ├── ui/
    │   └── mod.rs       # Ratatui layout, widgets, formatters, and unit tests
    ├── system/
    │   └── mod.rs       # OS and host metadata collection and unit tests
    ├── hardware/
    │   └── mod.rs       # CPU and memory collection
    └── network/
        └── mod.rs       # Network interface and traffic counter collection
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
