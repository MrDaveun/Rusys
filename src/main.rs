mod hardware;
mod network;
mod system;

fn main() {
    let sys_info = system::collect_info();
    let hw_info = hardware::collect_info();
    let net_info = network::get_network_info();

    println!("=== System ===");
    println!("OS: {}", sys_info.os);
    println!("Arch: {}", sys_info.arch);
    println!("Family: {}", sys_info.family);
    println!("Hostname: {}", sys_info.hostname.expect("Hostname unavailable"));
    println!(
        "OS Version: {}",
        sys_info.os_version.as_deref().unwrap_or("unknown")
    );
    println!(
        "Kernel Version: {}",
        sys_info.kernel_version.as_deref().unwrap_or("unknown")
    );
    println!("Distribution: {}", sys_info.distribution);
    println!("Uptime: {}s", sys_info.uptime);

    println!("\n=== Hardware ===");
    println!("CPU: {}", hw_info.cpu.name);
    println!("Manufacturer: {}", hw_info.cpu.manufacturer);
    println!("Frequency: {} MHz", hw_info.cpu.frequency);
    println!("Cores: {}", hw_info.cpu.cores);
    println!("RAM Total: {} bytes", hw_info.ram.total);
    println!("RAM Used: {} bytes", hw_info.ram.used);
    println!("RAM Available: {} bytes", hw_info.ram.available);

    println!("\n=== Network ===");
    for iface in net_info.interfaces {
        println!(
            "{} | MAC: {} | RX: {} B | TX: {} B",
            iface.name, iface.mac_address, iface.received, iface.transmitted
        );
    }
}
