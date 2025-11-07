use crate::history::buffer::MetricsHistory;
use crate::error::{Error, Result};
use std::fs::File;
use std::io::Write;
use serde_json::json;

pub fn export_to_csv(history: &MetricsHistory, path: &str) -> Result<()> {
    let mut file = File::create(path)?;
    
    writeln!(file, "timestamp,cpu_usage,memory_usage,swap_usage")?;
    
    let cpu_data = history.cpu_usage.get_all();
    let mem_data = history.memory_usage.get_all();
    let swap_data = history.swap_usage.get_all();
    
    let max_len = cpu_data.len().max(mem_data.len()).max(swap_data.len());
    
    for i in 0..max_len {
        let timestamp = cpu_data.get(i).map(|d| d.timestamp).unwrap_or(0);
        let cpu = cpu_data.get(i).map(|d| d.value).unwrap_or(0.0);
        let mem = mem_data.get(i).map(|d| d.value).unwrap_or(0.0);
        let swap = swap_data.get(i).map(|d| d.value).unwrap_or(0.0);
        
        writeln!(file, "{},{:.2},{:.2},{:.2}", timestamp, cpu, mem, swap)?;
    }
    
    Ok(())
}

pub fn export_to_json(history: &MetricsHistory, path: &str) -> Result<()> {
    let cpu_data: Vec<_> = history.cpu_usage.get_all().iter()
        .map(|dp| json!({"timestamp": dp.timestamp, "value": dp.value}))
        .collect();
    
    let mem_data: Vec<_> = history.memory_usage.get_all().iter()
        .map(|dp| json!({"timestamp": dp.timestamp, "value": dp.value}))
        .collect();
    
    let swap_data: Vec<_> = history.swap_usage.get_all().iter()
        .map(|dp| json!({"timestamp": dp.timestamp, "value": dp.value}))
        .collect();
    
    let mut gpu_data = Vec::new();
    for (i, gpu_temp_buffer) in history.gpu_temp.iter().enumerate() {
        let temp_data: Vec<_> = gpu_temp_buffer.get_all().iter()
            .map(|dp| json!({"timestamp": dp.timestamp, "value": dp.value}))
            .collect();
        
        let util_data: Vec<_> = history.gpu_util.get(i)
            .map(|b| b.get_all().iter()
                .map(|dp| json!({"timestamp": dp.timestamp, "value": dp.value}))
                .collect())
            .unwrap_or_default();
        
        let mem_data: Vec<_> = history.gpu_mem.get(i)
            .map(|b| b.get_all().iter()
                .map(|dp| json!({"timestamp": dp.timestamp, "value": dp.value}))
                .collect())
            .unwrap_or_default();
        
        gpu_data.push(json!({
            "gpu_id": i,
            "temperature": temp_data,
            "utilization": util_data,
            "memory": mem_data,
        }));
    }
    
    let output = json!({
        "cpu": cpu_data,
        "memory": mem_data,
        "swap": swap_data,
        "gpus": gpu_data,
    });
    
    let json_str = serde_json::to_string_pretty(&output)
        .map_err(|e| Error::Unknown(format!("Failed to serialize JSON: {}", e)))?;
    
    let mut file = File::create(path)?;
    file.write_all(json_str.as_bytes())?;
    
    Ok(())
}
