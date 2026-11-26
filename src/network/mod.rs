use std::net::IpAddr;
use std::time::Instant;

use sysinfo::Networks;

pub struct InterfaceInfo {
    pub name: String,
    pub mac_address: String,

    pub ip_addresses: Vec<String>,
    pub mtu: u64,

    pub received: u64,
    pub transmitted: u64,

    pub total_received: u64,
    pub total_transmitted: u64,

    pub download_speed: f64,
    pub upload_speed: f64,

    pub packets_received: u64,
    pub packets_transmitted: u64,

    pub errors_received: u64,
    pub errors_transmitted: u64,
}

pub struct NetworkInfo {
    pub interfaces: Vec<InterfaceInfo>,
}

pub struct NetworkCollector {
    networks: Networks,
    last_refresh: Instant,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            last_refresh: Instant::now(),
        }
    }

    pub fn collect(&mut self) -> NetworkInfo {
        self.networks.refresh(true);

        let elapsed = self.last_refresh.elapsed().as_secs_f64();
        self.last_refresh = Instant::now();

        let interfaces = self
            .networks
            .iter()
            .map(|(name, data)| {
                let received = data.received();
                let transmitted = data.transmitted();

                let ip_addresses = data
                    .ip_networks()
                    .iter()
                    .map(|ip| match ip.addr {
                        IpAddr::V4(_) => format!("IPv4: {}", ip),
                        IpAddr::V6(_) => format!("IPv6: {}", ip),
                    })
                    .collect();

                InterfaceInfo {
                    name: name.to_string(),
                    mac_address: data.mac_address().to_string(),

                    ip_addresses,
                    mtu: data.mtu(),

                    received,
                    transmitted,

                    total_received: data.total_received(),
                    total_transmitted: data.total_transmitted(),

                    download_speed: if elapsed > 0.0 {
                        received as f64 / elapsed
                    } else {
                        0.0
                    },

                    upload_speed: if elapsed > 0.0 {
                        transmitted as f64 / elapsed
                    } else {
                        0.0
                    },

                    packets_received: data.total_packets_received(),
                    packets_transmitted: data.total_packets_transmitted(),

                    errors_received: data.total_errors_on_received(),
                    errors_transmitted: data.total_errors_on_transmitted(),
                }
            })
            .collect();

        NetworkInfo { interfaces }
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkCollector;

    #[test]
    fn collects_network_interfaces() {
        let mut collector = NetworkCollector::new();
        let info = collector.collect();
        for iface in &info.interfaces {
            assert!(!iface.name.is_empty());
            assert!(iface.download_speed >= 0.0);
            assert!(iface.upload_speed >= 0.0);
        }
    }
}
