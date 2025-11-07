#[cfg(unix)]
use notify_rust::{Notification, Urgency};

use super::{Alert, AlertLevel};

pub struct Notifier {
    enabled: bool,
}

impl Notifier {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn send_alert(&self, alert: &Alert) {
        if !self.enabled {
            return;
        }

        #[cfg(unix)]
        {
            let (summary, urgency) = match alert.level {
                AlertLevel::Critical => ("⚠️ Critical Alert", Urgency::Critical),
                AlertLevel::Warning => ("⚠ Warning", Urgency::Normal),
                AlertLevel::Info => ("ℹ Info", Urgency::Low),
            };

            let icon = match alert.level {
                AlertLevel::Critical => "dialog-error",
                AlertLevel::Warning => "dialog-warning",
                AlertLevel::Info => "dialog-information",
            };

            let _ = Notification::new()
                .summary(&format!("{} - GleamObserver", summary))
                .body(&alert.message)
                .icon(icon)
                .urgency(urgency)
                .timeout(5000) // 5 seconds
                .show();
        }

        #[cfg(not(unix))]
        {
            // For non-Unix systems, just log
            log::warn!("Alert: {} - {}", 
                match alert.level {
                    AlertLevel::Critical => "CRITICAL",
                    AlertLevel::Warning => "WARNING",
                    AlertLevel::Info => "INFO",
                },
                alert.message
            );
        }
    }
}
