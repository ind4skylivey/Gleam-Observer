use sysinfo::{Disks};

pub struct DiskMetrics {
    disks: Disks,
}

impl DiskMetrics {
    pub fn new() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list(),
        }
    }

    pub fn refresh(&mut self) {
        self.disks.refresh();
    }

    pub fn list(&self) -> Vec<DiskInfo> {
        self.disks.iter()
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                file_system: disk.file_system().to_string_lossy().to_string(),
            })
            .collect()
    }
}

impl Default for DiskMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub file_system: String,
}

impl DiskInfo {
    pub fn used_space(&self) -> u64 {
        self.total_space.saturating_sub(self.available_space)
    }

    pub fn usage_percent(&self) -> f32 {
        if self.total_space == 0 {
            return 0.0;
        }
        (self.used_space() as f32 / self.total_space as f32) * 100.0
    }
}
