# Rusys

A **Rust** TUI (Terminal User Interface) tool for monitoring system, hardware, and network information using [`ratatui`](https://crates.io/crates/ratatui) and [`sysinfo`](https://crates.io/crates/sysinfo).

## Project Structure

```text
Rusys/
├── .github/
│   └── workflows/
│       ├── ci.yml       # Cross-platform compile, test, and build checks
│       └── release.yml  # Release executable builds for Linux, Windows, and macOS
├── Cargo.toml
├── LICENSE
├── README.md
└── src/
    ├── main.rs          # Ratatui TUI entry point and event loop
    ├── ui/
    │   └── mod.rs       # Terminal dashboard layout (System, Hardware, Network)
    ├── system/
    │   └── mod.rs       # OS info collection (OS, architecture, family, hostname, kernel, distro, uptime)
    ├── hardware/
    │   └── mod.rs       # CPU info (brand, vendor, frequency, cores) and RAM info collection
    └── network/
        └── mod.rs       # Network info collection (interfaces, MAC address, total RX/TX bytes)
```

## Current Implementation

- **System (`src/system/mod.rs`)**: `SystemInfo` and `collect_info()` collecting operating system, architecture, family, hostname, OS version, kernel version, distribution, and uptime.
- **Hardware (`src/hardware/mod.rs`)**: `CpuInfo`, `RamInfo`, `HardwareInfo`, `get_cpu()`, `get_ram()`, and `collect_info()`.
- **Network (`src/network/mod.rs`)**: `InterfaceInfo`, `NetworkInfo`, `get_interfaces()`, and `get_network_info()`.
- **UI (`src/ui/mod.rs`)**: Ratatui dashboard displaying System info, CPU details, a live RAM usage gauge, and a navigable Network interfaces table.
- **Main (`src/main.rs`)**: Event loop with 1-second auto-refresh and keyboard controls (`q`/`Esc` to quit, `r` to refresh, `↑`/`↓` or `j`/`k` to select network interface).

## Running

With Rust and Cargo installed:

```bash
cargo run
```

To check compilation and run tests:

```bash
cargo check
cargo test
```

## GitHub Actions

- **CI (`ci.yml`)**: Runs `cargo check`, `cargo test`, and `cargo build` on Linux, Windows, and macOS on every push and pull request.
- **Release Executables (`release.yml`)**: Builds release binaries for Linux (`x86_64`), Windows (`x86_64`), and macOS (`arm64` and `x86_64`) via manual workflow dispatch or `v*` tags (automatically publishing GitHub Release assets on tags).

## Workflow

This repository is being structured using **Google AI Studio** with the **Gemini 4 Hydrogen** model to organize modules and prepare commits prior to building with Cargo.

## License

This project is licensed under the [MIT License](LICENSE).
