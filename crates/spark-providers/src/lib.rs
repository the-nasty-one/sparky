#![allow(non_snake_case)]

pub mod cpu;
pub mod disk;
pub mod docker;
pub mod gpu;
pub mod memory;
pub mod models;
pub mod uptime;

use spark_types::SystemMetrics;

pub async fn collect_system_metrics() -> SystemMetrics {
    let (gpuResult, memoryResult, cpuResult, diskResult, uptimeResult) = tokio::join!(
        gpu::collect(),
        memory::collect(),
        cpu::collect(),
        disk::collect(),
        uptime::collect(),
    );

    SystemMetrics {
        gpu: gpuResult,
        memory: memoryResult,
        cpu: cpuResult,
        disk: diskResult,
        uptime: uptimeResult,
    }
}
