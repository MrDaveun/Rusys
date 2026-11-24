use sysinfo::{CpuRefreshKind, System};

pub struct CpuInfo {
    pub name: String,
    pub manufacturer: String,
    pub frequency: u64,
    pub cores: usize,
}

pub struct RamInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
}

pub fn get_cpu() -> CpuInfo {
    let mut sys = System::new();
    sys.refresh_cpu_list(CpuRefreshKind::everything());
    let cpu = sys.cpus();
    let first = cpu.first().expect("CPU unavailable");

    let name = first.brand().to_string();
    let manufacturer = first.vendor_id().to_string();
    let frequency = first.frequency();
    let cores = cpu.len();

    CpuInfo {
        name,
        manufacturer,
        frequency,
        cores,
    }
}

pub fn get_ram() -> RamInfo {
    let mut sys = System::new();
    sys.refresh_memory();

    RamInfo {
        total: sys.total_memory(),
        used: sys.used_memory(),
        available: sys.available_memory(),
    }
}

pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub ram: RamInfo,
}

pub fn collect_info() -> HardwareInfo {
    HardwareInfo {
        cpu: get_cpu(),
        ram: get_ram(),
    }
}
