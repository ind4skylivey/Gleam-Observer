use sysinfo::Networks;

pub struct NetworkMetrics {
    networks: Networks,
}

impl NetworkMetrics {
    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
        }
    }

    pub fn refresh(&mut self) {
        self.networks.refresh();
    }

    pub fn list(&self) -> Vec<NetworkInfo> {
        self.networks.iter()
            .map(|(name, network)| NetworkInfo {
                name: name.to_string(),
                received: network.received(),
                transmitted: network.transmitted(),
                packets_received: network.packets_received(),
                packets_transmitted: network.packets_transmitted(),
                errors_on_received: network.errors_on_received(),
                errors_on_transmitted: network.errors_on_transmitted(),
            })
            .collect()
    }
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
    pub errors_on_received: u64,
    pub errors_on_transmitted: u64,
}
