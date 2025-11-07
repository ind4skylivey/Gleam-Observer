mod notifications;
mod signals;

pub use notifications::{send_alert, send_status_update};
pub use signals::setup_signal_handlers;

use crate::config::Config;
use crate::error::Result;
use crate::metrics::SystemMetrics;
use crate::gpu::GPUManager;
use crate::alerts::{AlertDetector, Notifier};
use crate::history::MetricsHistory;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicBool, Ordering};

static STOP_FLAG: AtomicBool = AtomicBool::new(false);

pub struct DaemonContext {
    pub config: Config,
    pub metrics: SystemMetrics,
    pub gpu: Option<GPUManager>,
    pub alert_detector: AlertDetector,
    pub notifier: Notifier,
    pub history: MetricsHistory,
    pub alerts_paused: bool,
    pub cpu_percent: f32,
    pub mem_percent: f32,
}

impl DaemonContext {
    pub fn new(config: Config) -> Result<Self> {
        let gpu = {
            let manager = GPUManager::new();
            if manager.has_gpus() {
                log::info!("GPU monitoring enabled: {} GPU(s) detected", manager.gpu_count());
                Some(manager)
            } else {
                log::warn!("GPU monitoring enabled but no GPUs detected");
                None
            }
        };

        let alert_detector = AlertDetector::new(config.alerts.clone());
        let gpu_count = if let Some(ref gpu_manager) = gpu {
            gpu_manager.gpu_count()
        } else {
            0
        };
        let notifier = Notifier::new(config.alerts.notifications_enabled);
        let history = MetricsHistory::new(config.refresh.history_samples, gpu_count);
        
        Ok(Self {
            config,
            metrics: SystemMetrics::new(),
            gpu,
            alert_detector,
            notifier,
            history,
            alerts_paused: false,
            cpu_percent: 0.0,
            mem_percent: 0.0,
        })
    }

    fn monitoring_loop(&mut self) -> Result<()> {
        let refresh_interval = Duration::from_millis(self.config.refresh.interval_ms);
        
        log::info!("Starting daemon monitoring loop (interval: {}ms)", self.config.refresh.interval_ms);
        
        loop {
            let loop_start = std::time::Instant::now();
            
            if should_stop() {
                log::info!("Daemon stop signal received");
                break;
            }
            
            self.update_metrics();
            self.process_alerts();
            self.sleep_until_next_cycle(loop_start, refresh_interval);
        }
        
        Ok(())
    }
    
    fn update_metrics(&mut self) {
        self.metrics.refresh();
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.cpu_percent = self.metrics.global_cpu_usage();
        self.mem_percent = self.metrics.memory_usage_percent();
        let swap_usage = self.metrics.swap_usage_percent();
        
        let gpu_info = self.get_gpu_info();
        
        self.history.update(timestamp, self.cpu_percent, self.mem_percent, swap_usage, &gpu_info);
    }
    
    fn get_gpu_info(&self) -> Vec<crate::gpu::GPUInfo> {
        self.gpu.as_ref()
            .map(|manager| manager.get_info())
            .unwrap_or_default()
    }
    
    fn process_alerts(&mut self) {
        if !self.should_process_alerts() {
            return;
        }
        
        let swap_usage = self.metrics.swap_usage_percent();
        let swap_total = self.metrics.swap_total();
        let gpu_info = self.get_gpu_info();
        
        let alerts = self.alert_detector.check_alerts(
            self.cpu_percent,
            self.mem_percent,
            swap_usage,
            swap_total,
            &gpu_info,
        );
        
        for alert in alerts {
            log::warn!("Alert triggered: {:?} - {}", alert.level, alert.message);
            self.notifier.send_alert(&alert);
        }
    }
    
    fn should_process_alerts(&self) -> bool {
        !self.alerts_paused && self.config.alerts.enabled
    }
    
    fn sleep_until_next_cycle(&self, loop_start: std::time::Instant, refresh_interval: Duration) {
        let elapsed = loop_start.elapsed();
        if elapsed < refresh_interval {
            std::thread::sleep(refresh_interval - elapsed);
        }
    }

    pub fn start(config: Config) -> Result<()> {
        log::info!("Initializing GleamObserver daemon");
        
        signals::setup_signal_handlers()?;
        
        let mut ctx = Self::new(config)?;
        
        ctx.monitoring_loop()
    }
}

pub fn should_stop() -> bool {
    STOP_FLAG.load(Ordering::Relaxed)
}

pub fn set_stop_flag() {
    STOP_FLAG.store(true, Ordering::Relaxed);
}

#[cfg(unix)]
pub fn daemonize() -> Result<()> {
    use nix::unistd::{fork, ForkResult, setsid};
    use std::process::exit;
    
    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            exit(0);
        }
        Ok(ForkResult::Child) => {
            setsid().map_err(|e| crate::error::Error::Daemon(format!("setsid failed: {}", e)))?;
            
            match unsafe { fork() } {
                Ok(ForkResult::Parent { .. }) => {
                    exit(0);
                }
                Ok(ForkResult::Child) => {
                    std::env::set_current_dir("/")
                        .map_err(|e| crate::error::Error::Daemon(format!("chdir failed: {}", e)))?;
                    
                    Ok(())
                }
                Err(e) => Err(crate::error::Error::Daemon(format!("fork failed: {}", e))),
            }
        }
        Err(e) => Err(crate::error::Error::Daemon(format!("fork failed: {}", e))),
    }
}

#[cfg(not(unix))]
pub fn daemonize() -> Result<()> {
    Err(crate::error::Error::Daemon("Daemonization not supported on this platform".to_string()))
}
