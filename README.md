# Rusys

A **Rust** tool for collecting system, hardware, and network information using the [`sysinfo`](https://crates.io/crates/sysinfo) crate.

## Project Structure

```text
Rusys/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs          # Application entry point
    ├── system/
    │   └── mod.rs       # OS info collection (OS, architecture, family, hostname, kernel, distro, uptime)
    ├── hardware/
    │   └── mod.rs       # CPU info (brand, vendor, frequency, cores) and RAM info collection
    └── network/
        └── mod.rs       # Network info collection (interfaces, MAC address, received/transmitted bytes)
```

## Current Implementation

- **System (`src/system/mod.rs`)**: `SystemInfo` and `collect_info()` collecting operating system, architecture, family, hostname, OS version, kernel version, distribution, and uptime.
- **Hardware (`src/hardware/mod.rs`)**: `CpuInfo`, `RamInfo`, `HardwareInfo`, `get_cpu()`, `get_ram()`, and `collect_info()`.
- **Network (`src/network/mod.rs`)**: `InterfaceInfo`, `NetworkInfo`, `get_interfaces()`, and `get_network_info()`.
- **Main (`src/main.rs`)**: Entry point collecting and displaying all system, hardware, and network information.

## Running

With Rust and Cargo installed:

```bash
cargo run
```

To check compilation only:

```bash
cargo check
```

## Workflow

This repository is being structured using **Google AI Studio** with the **Gemini 4 Hydrogen** model to organize modules and prepare commits prior to building with Cargo.
