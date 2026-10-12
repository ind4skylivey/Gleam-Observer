#[cfg(all(target_os = "macos", feature = "apple-gpu"))]
pub mod apple_impl {
    use crate::gpu::backend::{GPUBackend, GPUProcess};

    pub struct AppleBackend;

    impl AppleBackend {
        pub fn detect_all() -> Result<Vec<AppleBackend>, String> {
            Ok(Vec::new())
        }
    }

    impl GPUBackend for AppleBackend {
        fn name(&self) -> String {
            "Apple Silicon GPU".to_string()
        }

        fn vendor(&self) -> String {
            "Apple".to_string()
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

        fn processes(&self) -> Vec<GPUProcess> {
            Vec::new()
        }
    }
}
