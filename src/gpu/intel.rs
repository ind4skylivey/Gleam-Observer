use crate::error::{Error, Result};
use crate::gpu::backend::GPUBackend;

pub struct IntelBackend;

impl IntelBackend {
    pub fn detect_all() -> Result<Vec<Self>> {
        Err(Error::Gpu("Intel GPU monitoring not yet implemented".to_string()))
    }
}

impl GPUBackend for IntelBackend {
    fn name(&self) -> String {
        "Intel GPU".to_string()
    }

    fn vendor(&self) -> String {
        "Intel".to_string()
    }

    fn temperature(&self) -> Option<f32> {
        None
    }

    fn utilization(&self) -> Option<f32> {
        None
    }

    fn memory_used(&self) -> Option<u64> {
        None
    }

    fn memory_total(&self) -> Option<u64> {
        None
    }

    fn power_draw(&self) -> Option<f32> {
        None
    }

    fn power_limit(&self) -> Option<f32> {
        None
    }

    fn clock_speed(&self) -> Option<u32> {
        None
    }

    fn memory_clock(&self) -> Option<u32> {
        None
    }

    fn fan_speed(&self) -> Option<u32> {
        None
    }

    fn processes(&self) -> Vec<crate::gpu::backend::GPUProcess> {
        Vec::new()
    }
}
