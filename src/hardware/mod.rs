use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};

pub struct CpuCoreInfo {
    pub name: String,
    pub usage_percent: f32,
    pub frequency: u64,
}

pub struct CpuInfo {
    pub name: String,
    pub manufacturer: String,
    pub frequency: u64,
    pub cores: usize,
    pub usage_percent: f32,
    pub cores_usage: Vec<CpuCoreInfo>,
}

pub struct RamInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
}

pub struct SwapInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
}

pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub ram: RamInfo,
    pub swap: SwapInfo,
    pub disks: Vec<DiskInfo>,
}

pub struct HardwareCollector {
    sys: System,
    disks: Disks,
}

impl HardwareCollector {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_all();
        let disks = Disks::new_with_refreshed_list();

        Self { sys, disks }
    }

    pub fn collect(&mut self) -> HardwareInfo {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.disks.refresh(true);

        HardwareInfo {
            cpu: extract_cpu(&self.sys),
            ram: extract_ram(&self.sys),
            swap: extract_swap(&self.sys),
            disks: extract_disks(&self.disks),
        }
    }
}

#[allow(dead_code)]
pub fn collect_info() -> HardwareInfo {
    let mut collector = HardwareCollector::new();
    collector.collect()
}

fn extract_cpu(sys: &System) -> CpuInfo {
    let cpus = sys.cpus();
    let (name, manufacturer, frequency) = match cpus.first() {
        Some(first) => (
            first.brand().to_string(),
            first.vendor_id().to_string(),
            first.frequency(),
        ),
        None => ("Unknown CPU".to_string(), "Unknown".to_string(), 0),
    };

    let cores_usage = cpus
        .iter()
        .enumerate()
        .map(|(idx, cpu)| {
            let core_name = if cpu.name().is_empty() {
                format!("CPU {}", idx)
            } else {
                cpu.name().to_string()
            };

            CpuCoreInfo {
                name: core_name,
                usage_percent: cpu.cpu_usage().clamp(0.0, 100.0),
                frequency: cpu.frequency(),
            }
        })
        .collect();

    CpuInfo {
        name,
        manufacturer,
        frequency,
        cores: cpus.len(),
        usage_percent: sys.global_cpu_usage().clamp(0.0, 100.0),
        cores_usage,
    }
}

fn extract_ram(sys: &System) -> RamInfo {
    RamInfo {
        total: sys.total_memory(),
        used: sys.used_memory(),
        available: sys.available_memory(),
    }
}

fn extract_swap(sys: &System) -> SwapInfo {
    SwapInfo {
        total: sys.total_swap(),
        used: sys.used_swap(),
        free: sys.free_swap(),
    }
}

fn extract_disks(disks: &Disks) -> Vec<DiskInfo> {
    disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            DiskInfo {
                name: disk.name().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().display().to_string(),
                file_system: disk.file_system().to_string_lossy().into_owned(),
                total,
                used,
                available,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::collect_info;

    #[test]
    fn collects_hardware_swap_and_disks() {
        let info = collect_info();
        assert!(info.cpu.cores > 0);
        assert_eq!(info.cpu.cores_usage.len(), info.cpu.cores);
        assert!((0.0..=100.0).contains(&info.cpu.usage_percent));
        for core in &info.cpu.cores_usage {
            assert!(!core.name.is_empty());
            assert!((0.0..=100.0).contains(&core.usage_percent));
        }
        assert!(info.ram.total > 0);
        assert!(info.swap.used <= info.swap.total);
        for disk in &info.disks {
            assert_eq!(disk.used, disk.total.saturating_sub(disk.available));
        }
    }
}
