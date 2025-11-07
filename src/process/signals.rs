use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use anyhow::{Result, anyhow};

/// Smart kill: Try SIGTERM first, wait, then escalate to SIGKILL if needed
pub async fn smart_kill(pid: u32) -> Result<String> {
    if !process_exists(pid) {
        return Err(anyhow!("Process {} does not exist", pid));
    }

    // Step 1: Send SIGTERM (graceful termination)
    log::info!("Sending SIGTERM to process {}", pid);
    send_signal_to_process(pid, Signal::SIGTERM)?;

    // Step 2: Wait 3 seconds for graceful shutdown
    sleep(Duration::from_secs(3)).await;

    // Step 3: Check if process still exists
    if process_exists(pid) {
        log::warn!("Process {} didn't respond to SIGTERM, escalating to SIGKILL", pid);
        send_signal_to_process(pid, Signal::SIGKILL)?;
        
        // Give it a moment
        sleep(Duration::from_millis(500)).await;
        
        if process_exists(pid) {
            return Ok(format!("Sent SIGKILL to process {} (escalated from SIGTERM)", pid));
        } else {
            return Ok(format!("Process {} terminated (escalated to SIGKILL)", pid));
        }
    } else {
        return Ok(format!("Process {} terminated gracefully (SIGTERM)", pid));
    }
}

/// Force kill: Immediate SIGKILL without waiting
pub fn force_kill(pid: u32) -> Result<String> {
    if !process_exists(pid) {
        return Err(anyhow!("Process {} does not exist", pid));
    }

    log::info!("Sending SIGKILL to process {}", pid);
    send_signal_to_process(pid, Signal::SIGKILL)?;
    
    Ok(format!("Sent SIGKILL to process {}", pid))
}

/// Terminate: Send SIGTERM only
pub fn terminate(pid: u32) -> Result<String> {
    if !process_exists(pid) {
        return Err(anyhow!("Process {} does not exist", pid));
    }

    log::info!("Sending SIGTERM to process {}", pid);
    send_signal_to_process(pid, Signal::SIGTERM)?;
    
    Ok(format!("Sent SIGTERM to process {}", pid))
}

/// Send arbitrary signal to a process
pub fn send_signal_to_process(pid: u32, sig: Signal) -> Result<()> {
    let nix_pid = Pid::from_raw(pid as i32);
    signal::kill(nix_pid, sig)
        .map_err(|e| anyhow!("Failed to send signal {:?} to process {}: {}", sig, pid, e))
}

/// Check if process exists by checking /proc/[pid]
fn process_exists(pid: u32) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}

/// Send signal with custom signal number
pub fn send_custom_signal(pid: u32, signal_num: i32) -> Result<String> {
    if !process_exists(pid) {
        return Err(anyhow!("Process {} does not exist", pid));
    }

    let sig = match signal_num {
        1 => Signal::SIGHUP,
        2 => Signal::SIGINT,
        9 => Signal::SIGKILL,
        15 => Signal::SIGTERM,
        19 => Signal::SIGSTOP,
        18 => Signal::SIGCONT,
        10 => Signal::SIGUSR1,
        12 => Signal::SIGUSR2,
        _ => return Err(anyhow!("Unsupported signal number: {}", signal_num)),
    };

    send_signal_to_process(pid, sig)?;
    Ok(format!("Sent signal {} to process {}", signal_num, pid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_exists() {
        // PID 1 (init/systemd) should always exist
        assert!(process_exists(1));
        
        // PID 99999 probably doesn't exist
        assert!(!process_exists(99999));
    }

    #[test]
    fn test_self_process_exists() {
        let self_pid = std::process::id();
        assert!(process_exists(self_pid));
    }
}
