pub mod backend;

#[cfg(feature = "nvidia")]
pub mod nvidia;

#[cfg(feature = "amd")]
pub mod amd;

#[cfg(feature = "intel")]
pub mod intel;

#[cfg(all(target_os = "macos", feature = "apple-gpu"))]
pub mod apple;

pub use backend::{GPUBackend, GPUManager, GPUInfo};
