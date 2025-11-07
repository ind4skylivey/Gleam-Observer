#[cfg(all(unix, feature = "systray"))]
use signal_hook::{consts::SIGTERM, iterator::Signals};
use crate::error::Result;

#[cfg(all(unix, feature = "systray"))]
pub fn setup_signal_handlers() -> Result<()> {
    let mut signals = Signals::new(&[SIGTERM, signal_hook::consts::SIGINT])
        .map_err(|e| crate::error::Error::Daemon(format!("Failed to setup signal handlers: {}", e)))?;

    std::thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                signal_hook::consts::SIGTERM | signal_hook::consts::SIGINT => {
                    log::info!("Received termination signal, shutting down gracefully");
                    super::set_stop_flag();
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    });

    Ok(())
}

#[cfg(not(all(unix, feature = "systray")))]
pub fn setup_signal_handlers() -> Result<()> {
    Ok(())
}
