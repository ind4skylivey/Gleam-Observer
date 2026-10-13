use crate::config::AlertsConfig;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub enum AlertType {
    CpuUsage,
    MemoryUsage,
    SwapUsage,
    GpuTemperature { gpu_id: usize },
    GpuUtilization { gpu_id: usize },
    GpuMemory { gpu_id: usize },
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub alert_type: AlertType,
    pub level: AlertLevel,
    pub value: f32,
    pub threshold: f32,
    pub message: String,
    pub timestamp: u64,
}

impl Alert {
    fn new(
        alert_type: AlertType,
        level: AlertLevel,
        value: f32,
        threshold: f32,
        message: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            alert_type,
            level,
            value,
            threshold,
            message,
            timestamp,
        }
    }
}

pub struct AlertDetector {
    config: AlertsConfig,
    active_alerts: Vec<Alert>,
    last_notification_time: HashMap<String, u64>,
}

impl AlertDetector {
    pub fn new(config: AlertsConfig) -> Self {
        Self {
            config,
            active_alerts: Vec::new(),
            last_notification_time: HashMap::new(),
        }
    }

    fn get_alert_key(alert_type: &AlertType) -> String {
        match alert_type {
            AlertType::CpuUsage => "cpu".to_string(),
            AlertType::MemoryUsage => "memory".to_string(),
            AlertType::SwapUsage => "swap".to_string(),
            AlertType::GpuTemperature { gpu_id } => format!("gpu_{}_temp", gpu_id),
            AlertType::GpuUtilization { gpu_id } => format!("gpu_{}_util", gpu_id),
            AlertType::GpuMemory { gpu_id } => format!("gpu_{}_mem", gpu_id),
        }
    }

    pub fn should_notify(&mut self, alert_type: &AlertType) -> bool {
        let key = Self::get_alert_key(alert_type);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Some(&last_time) = self.last_notification_time.get(&key) {
            if now - last_time < self.config.notification_cooldown_secs {
                return false; // Still in cooldown
            }
        }

        self.last_notification_time.insert(key, now);
        true
    }

    pub fn check_alerts(
        &mut self,
        cpu_usage: f32,
        mem_usage: f32,
        swap_usage: f32,
        swap_total: u64,
        gpu_infos: &[crate::gpu::GPUInfo],
    ) -> Vec<Alert> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut new_alerts = Vec::new();

        // Check CPU
        if cpu_usage > self.config.cpu_threshold {
            let level = if cpu_usage > 95.0 {
                AlertLevel::Critical
            } else if cpu_usage > self.config.cpu_threshold {
                AlertLevel::Warning
            } else {
                AlertLevel::Info
            };

            let alert = Alert::new(
                AlertType::CpuUsage,
                level,
                cpu_usage,
                self.config.cpu_threshold,
                format!(
                    "CPU usage at {:.1}% (threshold: {:.1}%)",
                    cpu_usage, self.config.cpu_threshold
                ),
            );
            new_alerts.push(alert);
        }

        // Check Memory
        if mem_usage > self.config.memory_threshold {
            let level = if mem_usage > 95.0 {
                AlertLevel::Critical
            } else {
                AlertLevel::Warning
            };

            let alert = Alert::new(
                AlertType::MemoryUsage,
                level,
                mem_usage,
                self.config.memory_threshold,
                format!(
                    "Memory usage at {:.1}% (threshold: {:.1}%)",
                    mem_usage, self.config.memory_threshold
                ),
            );
            new_alerts.push(alert);
        }

        // Check Swap
        if swap_usage > self.config.swap_threshold && swap_total > 0 {
            let alert = Alert::new(
                AlertType::SwapUsage,
                AlertLevel::Warning,
                swap_usage,
                self.config.swap_threshold,
                format!(
                    "SWAP usage at {:.1}% (threshold: {:.1}%)",
                    swap_usage, self.config.swap_threshold
                ),
            );
            new_alerts.push(alert);
        }

        // Check GPU
        for (i, gpu_info) in gpu_infos.iter().enumerate() {
            // GPU Temperature
            if let Some(temp) = gpu_info.temperature {
                if temp > self.config.gpu_temp_threshold {
                    let level = if temp > 85.0 {
                        AlertLevel::Critical
                    } else {
                        AlertLevel::Warning
                    };

                    let alert = Alert::new(
                        AlertType::GpuTemperature { gpu_id: i },
                        level,
                        temp,
                        self.config.gpu_temp_threshold,
                        format!(
                            "GPU {} temperature at {:.1}°C (threshold: {:.1}°C)",
                            i, temp, self.config.gpu_temp_threshold
                        ),
                    );
                    new_alerts.push(alert);
                }
            }

            // GPU Utilization
            if let Some(util) = gpu_info.utilization {
                if util > self.config.gpu_util_threshold {
                    let alert = Alert::new(
                        AlertType::GpuUtilization { gpu_id: i },
                        AlertLevel::Info,
                        util,
                        self.config.gpu_util_threshold,
                        format!(
                            "GPU {} utilization at {:.1}% (threshold: {:.1}%)",
                            i, util, self.config.gpu_util_threshold
                        ),
                    );
                    new_alerts.push(alert);
                }
            }

            // GPU Memory
            if let Some(mem_percent) = gpu_info.memory_usage_percent() {
                if mem_percent > self.config.gpu_mem_threshold {
                    let alert = Alert::new(
                        AlertType::GpuMemory { gpu_id: i },
                        AlertLevel::Warning,
                        mem_percent,
                        self.config.gpu_mem_threshold,
                        format!(
                            "GPU {} memory at {:.1}% (threshold: {:.1}%)",
                            i, mem_percent, self.config.gpu_mem_threshold
                        ),
                    );
                    new_alerts.push(alert);
                }
            }
        }

        self.active_alerts = new_alerts.clone();
        new_alerts
    }

    pub fn active_alerts(&self) -> &[Alert] {
        &self.active_alerts
    }

    pub fn has_alerts(&self) -> bool {
        !self.active_alerts.is_empty()
    }

    pub fn critical_count(&self) -> usize {
        self.active_alerts
            .iter()
            .filter(|a| a.level == AlertLevel::Critical)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.active_alerts
            .iter()
            .filter(|a| a.level == AlertLevel::Warning)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AlertsConfig;

    fn test_config() -> AlertsConfig {
        AlertsConfig {
            enabled: true,
            notifications_enabled: true,
            notification_cooldown_secs: 60,
            cpu_threshold: 85.0,
            memory_threshold: 90.0,
            swap_threshold: 80.0,
            gpu_temp_threshold: 75.0,
            gpu_util_threshold: 95.0,
            gpu_mem_threshold: 90.0,
        }
    }

    fn test_gpu_info(
        temp: Option<f32>,
        util: Option<f32>,
        mem_pct: Option<f32>,
    ) -> crate::gpu::GPUInfo {
        let (memory_used, memory_total) = match mem_pct {
            Some(p) => (Some((p * 2048.0 / 100.0) as u64), Some(2048)),
            None => (Some(1024), Some(2048)),
        };
        crate::gpu::GPUInfo {
            id: 0,
            name: "Test GPU".to_string(),
            vendor: "Test".to_string(),
            temperature: temp,
            utilization: util,
            memory_used,
            memory_total,
            power_draw: None,
            power_limit: None,
            power_efficiency: None,
            clock_speed: None,
            memory_clock: None,
            fan_speed: None,
            processes: Vec::new(),
        }
    }

    #[test]
    fn test_no_alerts_below_thresholds() {
        let mut detector = AlertDetector::new(test_config());
        let alerts = detector.check_alerts(50.0, 60.0, 40.0, 8_000_000_000, &[]);
        assert!(alerts.is_empty());
        assert!(!detector.has_alerts());
    }

    #[test]
    fn test_cpu_alert_levels() {
        let mut detector = AlertDetector::new(test_config());

        let warnings = detector.check_alerts(90.0, 50.0, 40.0, 8_000_000_000, &[]);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].level, AlertLevel::Warning);
        assert!(matches!(warnings[0].alert_type, AlertType::CpuUsage));

        let criticals = detector.check_alerts(96.0, 50.0, 40.0, 8_000_000_000, &[]);
        assert_eq!(criticals.len(), 1);
        assert_eq!(criticals[0].level, AlertLevel::Critical);
        assert_eq!(detector.critical_count(), 1);
    }

    #[test]
    fn test_memory_and_swap_alerts() {
        let mut detector = AlertDetector::new(test_config());

        let alerts = detector.check_alerts(50.0, 96.0, 85.0, 8_000_000_000, &[]);
        assert_eq!(alerts.len(), 2);
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, AlertType::MemoryUsage)
                && a.level == AlertLevel::Critical));
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, AlertType::SwapUsage)));
        assert_eq!(detector.warning_count(), 1);
    }

    #[test]
    fn test_swap_ignored_when_disabled() {
        let mut detector = AlertDetector::new(test_config());
        let alerts = detector.check_alerts(50.0, 60.0, 99.0, 0, &[]);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_gpu_alerts() {
        let mut detector = AlertDetector::new(test_config());
        let hot_gpu = test_gpu_info(Some(80.0), Some(96.0), Some(95.0));

        let alerts = detector.check_alerts(50.0, 50.0, 40.0, 8_000_000_000, &[hot_gpu]);
        assert_eq!(alerts.len(), 3);
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, AlertType::GpuTemperature { .. })));
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, AlertType::GpuUtilization { .. })));
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, AlertType::GpuMemory { .. })));
    }

    #[test]
    fn test_disabled_detector_returns_nothing() {
        let mut config = test_config();
        config.enabled = false;
        let mut detector = AlertDetector::new(config);
        let alerts = detector.check_alerts(99.0, 99.0, 99.0, 8_000_000_000, &[]);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_notification_cooldown() {
        let mut detector = AlertDetector::new(test_config());
        let alert_type = AlertType::CpuUsage;
        assert!(detector.should_notify(&alert_type));
        assert!(!detector.should_notify(&alert_type));

        let other = AlertType::MemoryUsage;
        assert!(detector.should_notify(&other));
    }
}
