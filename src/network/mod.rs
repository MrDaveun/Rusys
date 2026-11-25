use sysinfo::Networks;

pub struct InterfaceInfo {
    pub name: String,
    pub mac_address: String,
    // pub ip_addresses: Vec<String>,
    pub received: u64,
    pub transmitted: u64,
}

pub struct NetworkInfo {
    pub interfaces: Vec<InterfaceInfo>,
}

pub fn get_interfaces() -> Vec<InterfaceInfo> {
    let networks = Networks::new_with_refreshed_list();
    let mut interfaces = Vec::new();

    for (name, data) in networks.iter() {
        interfaces.push(InterfaceInfo {
            name: name.to_string(),
            mac_address: data.mac_address().to_string(),
            received: data.received(),
            transmitted: data.transmitted(),
        });
    }

    interfaces
}

pub fn get_network_info() -> NetworkInfo {
    NetworkInfo {
        interfaces: get_interfaces(),
    }
}
