use crate::alerts::Alert;

pub fn send_alert(alert: &Alert) {
    #[cfg(unix)]
    {
        use notify_rust::{Notification, Urgency};
        
        let (summary, urgency) = match alert.level {
            crate::alerts::AlertLevel::Critical => ("Critical Alert", Urgency::Critical),
            crate::alerts::AlertLevel::Warning => ("Warning", Urgency::Normal),
            crate::alerts::AlertLevel::Info => ("Info", Urgency::Low),
        };

        let icon = match alert.level {
            crate::alerts::AlertLevel::Critical => "dialog-error",
            crate::alerts::AlertLevel::Warning => "dialog-warning",
            crate::alerts::AlertLevel::Info => "dialog-information",
        };

        let _ = Notification::new()
            .summary(&format!("{} - GleamObserver", summary))
            .body(&alert.message)
            .icon(icon)
            .urgency(urgency)
            .timeout(5000)
            .show();
    }

    #[cfg(not(unix))]
    {
        log::warn!("Alert: {} - {}", 
            match alert.level {
                crate::alerts::AlertLevel::Critical => "CRITICAL",
                crate::alerts::AlertLevel::Warning => "WARNING",
                crate::alerts::AlertLevel::Info => "INFO",
            },
            alert.message
        );
    }
}

pub fn send_status_update(cpu: f32, mem: f32) {
    #[cfg(unix)]
    {
        use notify_rust::Notification;
        
        let body = format!("CPU: {:.1}% | MEM: {:.1}%", cpu, mem);
        
        let _ = Notification::new()
            .summary("GleamObserver Status")
            .body(&body)
            .icon("gleamobserver")
            .timeout(3000)
            .show();
    }

    #[cfg(not(unix))]
    {
        log::info!("Status: CPU: {:.1}% | MEM: {:.1}%", cpu, mem);
    }
}
