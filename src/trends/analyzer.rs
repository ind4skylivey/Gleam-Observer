use super::types::{TrendDirection, TrendType, TrendSeverity, MetricTrend};
use crate::history::{CircularBuffer, DataPoint, MetricsHistory};
use crate::config::{TrendConfig, AlertsConfig};

pub struct TrendAnalyzer {
    config: TrendConfig,
    min_data_points: usize,
    analysis_window: usize,
}

impl TrendAnalyzer {
    pub fn new(config: TrendConfig) -> Self {
        Self {
            config,
            min_data_points: 3,  // Reduced from 5 to show trends faster
            analysis_window: 10,
        }
    }

    pub fn analyze_all(&self, history: &MetricsHistory, thresholds: &AlertsConfig) -> Vec<MetricTrend> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut trends = Vec::new();

        // Analyze CPU
        if let Some(cpu_trend) = self.analyze_metric(
            &history.cpu_usage,
            TrendType::Cpu,
            thresholds.cpu_threshold,
        ) {
            if cpu_trend.confidence >= self.config.min_confidence {
                trends.push(cpu_trend);
            }
        }

        // Analyze Memory
        if let Some(mem_trend) = self.analyze_metric(
            &history.memory_usage,
            TrendType::Memory,
            thresholds.memory_threshold,
        ) {
            if mem_trend.confidence >= self.config.min_confidence {
                trends.push(mem_trend);
            }
        }

        // Analyze SWAP
        if let Some(swap_trend) = self.analyze_metric(
            &history.swap_usage,
            TrendType::Swap,
            thresholds.swap_threshold,
        ) {
            if swap_trend.confidence >= self.config.min_confidence {
                trends.push(swap_trend);
            }
        }

        // Analyze GPU temperatures
        for (i, gpu_temp) in history.gpu_temp.iter().enumerate() {
            if let Some(trend) = self.analyze_metric(
                gpu_temp,
                TrendType::GpuTemp(i),
                thresholds.gpu_temp_threshold,
            ) {
                if trend.confidence >= self.config.min_confidence {
                    trends.push(trend);
                }
            }
        }

        // Analyze GPU utilization
        for (i, gpu_util) in history.gpu_util.iter().enumerate() {
            if let Some(trend) = self.analyze_metric(
                gpu_util,
                TrendType::GpuUtil(i),
                thresholds.gpu_util_threshold,
            ) {
                if trend.confidence >= self.config.min_confidence {
                    trends.push(trend);
                }
            }
        }

        // Analyze GPU memory
        for (i, gpu_mem) in history.gpu_mem.iter().enumerate() {
            if let Some(trend) = self.analyze_metric(
                gpu_mem,
                TrendType::GpuMemory(i),
                thresholds.gpu_mem_threshold,
            ) {
                if trend.confidence >= self.config.min_confidence {
                    trends.push(trend);
                }
            }
        }

        trends
    }

    fn analyze_metric(
        &self,
        buffer: &CircularBuffer<f32>,
        metric_type: TrendType,
        threshold: f32,
    ) -> Option<MetricTrend> {
        let data = buffer.get_all();

        if data.len() < self.min_data_points {
            return None;
        }

        // Take last N points for analysis
        let window_size = self.analysis_window.min(data.len());
        let recent_data: Vec<_> = data.iter()
            .rev()
            .take(window_size)
            .rev()
            .collect();

        // Linear regression: y = mx + b
        let (slope, intercept, r_squared) = self.linear_regression(&recent_data);

        // Calculate rate per minute
        let samples_per_minute = 60.0 / (self.config.sample_interval_secs as f32);
        let rate_per_minute = slope * samples_per_minute;

        // Determine direction
        let direction = if slope.abs() < 0.01 {
            TrendDirection::Stable
        } else if r_squared < 0.5 {
            TrendDirection::Volatile
        } else if slope > 0.0 {
            TrendDirection::Increasing
        } else {
            TrendDirection::Decreasing
        };

        // Skip if stable or volatile (unless configured to show)
        if !self.config.show_stable_trends {
            if matches!(direction, TrendDirection::Stable | TrendDirection::Volatile) {
                return None;
            }
        }

        // Current value
        let current = recent_data.last()?.value;

        // Predict 5 minutes ahead
        let minutes_ahead = 5.0;
        let samples_ahead = samples_per_minute * minutes_ahead;
        let predicted_5min = slope * (recent_data.len() as f32 + samples_ahead) + intercept;

        // Calculate time to threshold (if approaching)
        let time_to_threshold = if direction == TrendDirection::Increasing && current < threshold {
            if slope > 0.0 {
                let samples_to_threshold = (threshold - current) / slope;
                let seconds = samples_to_threshold * (self.config.sample_interval_secs as f32);
                if seconds > 0.0 && seconds < 7200.0 { // Max 2 hours
                    Some(seconds as u64)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Determine severity
        let severity = if let Some(time) = time_to_threshold {
            if time < 300 {
                TrendSeverity::Critical
            } else if time < 600 {
                TrendSeverity::Warning
            } else {
                TrendSeverity::Info
            }
        } else if rate_per_minute.abs() > 5.0 {
            TrendSeverity::Warning
        } else {
            TrendSeverity::Info
        };

        Some(MetricTrend {
            metric: metric_type,
            direction,
            rate_per_minute,
            confidence: r_squared,
            predicted_value_5min: predicted_5min,
            time_to_threshold,
            severity,
        })
    }

    fn linear_regression(&self, data: &[&DataPoint<f32>]) -> (f32, f32, f32) {
        let n = data.len() as f32;

        // Calculate means
        let mean_x = (n - 1.0) / 2.0;
        let mean_y: f32 = data.iter().map(|d| d.value).sum::<f32>() / n;

        // Calculate slope and intercept
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, point) in data.iter().enumerate() {
            let x = i as f32;
            let y = point.value;
            numerator += (x - mean_x) * (y - mean_y);
            denominator += (x - mean_x).powi(2);
        }

        let slope = if denominator != 0.0 {
            numerator / denominator
        } else {
            0.0
        };

        let intercept = mean_y - slope * mean_x;

        // Calculate R² (coefficient of determination)
        let mut ss_res = 0.0;
        let mut ss_tot = 0.0;

        for (i, point) in data.iter().enumerate() {
            let x = i as f32;
            let y = point.value;
            let y_pred = slope * x + intercept;
            ss_res += (y - y_pred).powi(2);
            ss_tot += (y - mean_y).powi(2);
        }

        let r_squared = if ss_tot != 0.0 {
            (1.0 - (ss_res / ss_tot)).max(0.0)
        } else {
            0.0
        };

        (slope, intercept, r_squared)
    }
}
