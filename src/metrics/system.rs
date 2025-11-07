use sysinfo::{System, Process, Pid, ProcessesToUpdate};
use std::collections::HashMap;

pub struct SystemMetrics {
    system: System,
}

impl SystemMetrics {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn refresh_cpu(&mut self) {
        self.system.refresh_cpu_all();
    }

    pub fn refresh_memory(&mut self) {
        self.system.refresh_memory();
    }

    pub fn refresh_processes(&mut self) {
        self.system.refresh_processes(ProcessesToUpdate::All);
    }

    pub fn cpu_usage(&self) -> Vec<f32> {
        self.system.cpus().iter()
            .map(|cpu| cpu.cpu_usage())
            .collect()
    }

    pub fn cpu_count(&self) -> usize {
        self.system.cpus().len()
    }

    pub fn cpu_info(&self) -> Vec<CpuInfo> {
        self.system.cpus().iter()
            .enumerate()
            .map(|(i, cpu)| CpuInfo {
                id: i,
                name: cpu.name().to_string(),
                usage: cpu.cpu_usage(),
                frequency: cpu.frequency(),
            })
            .collect()
    }

    pub fn global_cpu_usage(&self) -> f32 {
        self.system.global_cpu_usage()
    }

    pub fn memory_used(&self) -> u64 {
        self.system.used_memory()
    }

    pub fn memory_total(&self) -> u64 {
        self.system.total_memory()
    }

    pub fn memory_usage_percent(&self) -> f32 {
        let total = self.memory_total() as f32;
        if total == 0.0 {
            return 0.0;
        }
        (self.memory_used() as f32 / total) * 100.0
    }

    pub fn swap_used(&self) -> u64 {
        self.system.used_swap()
    }

    pub fn swap_total(&self) -> u64 {
        self.system.total_swap()
    }

    pub fn swap_usage_percent(&self) -> f32 {
        let total = self.swap_total() as f32;
        if total == 0.0 {
            return 0.0;
        }
        (self.swap_used() as f32 / total) * 100.0
    }

    pub fn processes(&self) -> &HashMap<Pid, Process> {
        self.system.processes()
    }

    pub fn process_count(&self) -> usize {
        self.system.processes().len()
    }

    pub fn top_processes_by_cpu(&self, limit: usize) -> Vec<ProcessInfo> {
        let mut processes: Vec<_> = self.system.processes()
            .iter()
            .map(|(pid, process)| {
                let cmd_vec: Vec<String> = process.cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect();
                let cmd = cmd_vec.join(" ");
                let cmd_display = if cmd.is_empty() {
                    process.name().to_string_lossy().to_string()
                } else {
                    cmd
                };
                
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    cmd: cmd_display,
                    cpu_usage: process.cpu_usage(),
                    memory_kb: process.memory(),
                    user: process.user_id()
                        .map(|uid| uid.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                }
            })
            .collect();

        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        processes.truncate(limit);
        processes
    }

    pub fn top_processes_by_memory(&self, limit: usize) -> Vec<ProcessInfo> {
        let mut processes: Vec<_> = self.system.processes()
            .iter()
            .map(|(pid, process)| {
                let cmd_vec: Vec<String> = process.cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect();
                let cmd = cmd_vec.join(" ");
                let cmd_display = if cmd.is_empty() {
                    process.name().to_string_lossy().to_string()
                } else {
                    cmd
                };
                
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    cmd: cmd_display,
                    cpu_usage: process.cpu_usage(),
                    memory_kb: process.memory(),
                    user: process.user_id()
                        .map(|uid| uid.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                }
            })
            .collect();

        processes.sort_by(|a, b| b.memory_kb.cmp(&a.memory_kb));
        processes.truncate(limit);
        processes
    }
    
    /// Get all processes as a flat list
    pub fn all_processes(&self) -> Vec<ProcessInfo> {
        self.system.processes()
            .iter()
            .map(|(pid, process)| {
                let cmd_vec: Vec<String> = process.cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect();
                let cmd = cmd_vec.join(" ");
                let cmd_display = if cmd.is_empty() {
                    process.name().to_string_lossy().to_string()
                } else {
                    cmd
                };
                
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    cmd: cmd_display,
                    cpu_usage: process.cpu_usage(),
                    memory_kb: process.memory(),
                    user: process.user_id()
                        .map(|uid| uid.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                }
            })
            .collect()
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub id: usize,
    pub name: String,
    pub usage: f32,
    pub frequency: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: String,  // Full command line
    pub cpu_usage: f32,
    pub memory_kb: u64,  // Memory in KB for consistency
    pub user: String,
}

impl ProcessInfo {
    pub fn memory(&self) -> u64 {
        self.memory_kb
    }
}
