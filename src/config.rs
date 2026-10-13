use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub refresh: RefreshConfig,
    pub alerts: AlertsConfig,
    pub display: DisplayConfig,
    pub trends: TrendConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshConfig {
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_history_samples")]
    pub history_samples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_notifications_enabled")]
    pub notifications_enabled: bool,
    #[serde(default = "default_notification_cooldown")]
    pub notification_cooldown_secs: u64,
    #[serde(default = "default_cpu_threshold")]
    pub cpu_threshold: f32,
    #[serde(default = "default_memory_threshold")]
    pub memory_threshold: f32,
    #[serde(default = "default_swap_threshold")]
    pub swap_threshold: f32,
    #[serde(default = "default_gpu_temp_threshold")]
    pub gpu_temp_threshold: f32,
    #[serde(default = "default_gpu_util_threshold")]
    pub gpu_util_threshold: f32,
    #[serde(default = "default_gpu_mem_threshold")]
    pub gpu_mem_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_show_processes")]
    pub show_processes: bool,
    #[serde(default = "default_process_count")]
    pub process_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendConfig {
    #[serde(default = "default_trend_enabled")]
    pub enabled: bool,
    #[serde(default = "default_sample_interval")]
    pub sample_interval_secs: u64,
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,
    #[serde(default = "default_show_stable")]
    pub show_stable_trends: bool,
}

fn default_interval_ms() -> u64 {
    1000
}
fn default_history_samples() -> usize {
    60
}
fn default_enabled() -> bool {
    true
}
fn default_notifications_enabled() -> bool {
    true
}
fn default_notification_cooldown() -> u64 {
    60
} // 1 minute cooldown
fn default_cpu_threshold() -> f32 {
    85.0
}
fn default_memory_threshold() -> f32 {
    90.0
}
fn default_swap_threshold() -> f32 {
    80.0
}
fn default_gpu_temp_threshold() -> f32 {
    75.0
}
fn default_gpu_util_threshold() -> f32 {
    95.0
}
fn default_gpu_mem_threshold() -> f32 {
    90.0
}
fn default_theme() -> String {
    "dark".to_string()
}
fn default_show_processes() -> bool {
    true
}
fn default_process_count() -> usize {
    10
}
fn default_trend_enabled() -> bool {
    true
}
fn default_sample_interval() -> u64 {
    1
}
fn default_min_confidence() -> f32 {
    0.7
}
fn default_show_stable() -> bool {
    true
} // Show all trends including stable ones for testing

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh: RefreshConfig {
                interval_ms: default_interval_ms(),
                history_samples: default_history_samples(),
            },
            alerts: AlertsConfig {
                enabled: default_enabled(),
                notifications_enabled: default_notifications_enabled(),
                notification_cooldown_secs: default_notification_cooldown(),
                cpu_threshold: default_cpu_threshold(),
                memory_threshold: default_memory_threshold(),
                swap_threshold: default_swap_threshold(),
                gpu_temp_threshold: default_gpu_temp_threshold(),
                gpu_util_threshold: default_gpu_util_threshold(),
                gpu_mem_threshold: default_gpu_mem_threshold(),
            },
            display: DisplayConfig {
                theme: default_theme(),
                show_processes: default_show_processes(),
                process_count: default_process_count(),
            },
            trends: TrendConfig {
                enabled: default_trend_enabled(),
                sample_interval_secs: default_sample_interval(),
                min_confidence: default_min_confidence(),
                show_stable_trends: default_show_stable(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| Error::Config("HOME environment variable not set".to_string()))?;

        Ok(PathBuf::from(home).join(".config/gleam-observer/config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = Config::default();
        assert_eq!(config.refresh.interval_ms, 1000);
        assert_eq!(config.refresh.history_samples, 60);
        assert!(config.alerts.enabled);
        assert_eq!(config.alerts.cpu_threshold, 85.0);
        assert_eq!(config.alerts.memory_threshold, 90.0);
        assert_eq!(config.display.theme, "dark");
        assert_eq!(config.display.process_count, 10);
        assert!(config.trends.enabled);
    }

    #[test]
    fn test_partial_toml_fills_defaults() {
        // Sections must be present (serde defaults apply per-field, not
        // per-section), but missing fields inside them fall back to defaults.
        let content = r#"
[refresh]
interval_ms = 500

[alerts]
cpu_threshold = 70.0

[display]

[trends]
"#;
        let config: Config = toml::from_str(content).expect("partial config should parse");
        assert_eq!(config.refresh.interval_ms, 500);
        assert_eq!(config.refresh.history_samples, 60);
        assert_eq!(config.alerts.cpu_threshold, 70.0);
        assert_eq!(config.alerts.memory_threshold, 90.0);
        assert_eq!(config.display.theme, "dark");
        assert_eq!(config.display.process_count, 10);
        assert!(config.trends.enabled);
    }

    #[test]
    fn test_invalid_toml_is_rejected() {
        let result: std::result::Result<Config, toml::de::Error> =
            toml::from_str("not [ valid toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_serialization() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).expect("serialize");
        let parsed: Config = toml::from_str(&serialized).expect("reparse");
        assert_eq!(parsed.refresh.interval_ms, config.refresh.interval_ms);
        assert_eq!(parsed.alerts.cpu_threshold, config.alerts.cpu_threshold);
        assert_eq!(parsed.display.theme, config.display.theme);
        assert_eq!(parsed.trends.min_confidence, config.trends.min_confidence);
    }
}
