use sysinfo::System;

pub struct SystemInfo {
    pub os: &'static str,
    pub arch: &'static str,
    pub family: &'static str,
    pub hostname: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub distribution: String,
    pub uptime: u64,
}

pub fn collect_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        family: std::env::consts::FAMILY,
        hostname: System::host_name(),
        os_version: System::os_version(),
        kernel_version: System::kernel_version(),
        distribution: System::distribution_id(),
        uptime: System::uptime(),
    }
}
