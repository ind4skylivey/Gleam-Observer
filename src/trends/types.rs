use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Increasing,   // Values going up
    Decreasing,   // Values going down
    Stable,       // No significant change
    Volatile,     // Erratic/unpredictable
}

#[derive(Debug, Clone)]
pub enum TrendType {
    Cpu,
    Memory,
    Swap,
    GpuTemp(usize),
    GpuUtil(usize),
    GpuMemory(usize),
}

impl fmt::Display for TrendType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrendType::Cpu => write!(f, "CPU"),
            TrendType::Memory => write!(f, "Memory"),
            TrendType::Swap => write!(f, "SWAP"),
            TrendType::GpuTemp(id) => write!(f, "GPU {} Temp", id),
            TrendType::GpuUtil(id) => write!(f, "GPU {} Usage", id),
            TrendType::GpuMemory(id) => write!(f, "GPU {} Memory", id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TrendSeverity {
    Info,      // Interesting but not concerning
    Warning,   // Worth attention
    Critical,  // Urgent action needed
}

#[derive(Debug, Clone)]
pub struct MetricTrend {
    pub metric: TrendType,
    pub direction: TrendDirection,
    pub rate_per_minute: f32,        // Change per minute
    pub confidence: f32,              // 0.0-1.0 (statistical significance)
    pub predicted_value_5min: f32,    // Forecast 5 minutes ahead
    pub time_to_threshold: Option<u64>, // Seconds until critical threshold
    pub severity: TrendSeverity,
}

// Alias for backward compatibility
pub type Trend = MetricTrend;
