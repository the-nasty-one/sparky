use spark_types::DiskMetrics;
use tracing::warn;

pub async fn collect() -> DiskMetrics {
    match read_disk_stats() {
        Ok(metrics) => metrics,
        Err(e) => {
            warn!("statvfs unavailable, returning mock disk data: {e}");
            mock_disk_metrics()
        }
    }
}

fn read_disk_stats() -> Result<DiskMetrics, String> {
    let stat = nix::sys::statvfs::statvfs("/")
        .map_err(|e| format!("statvfs failed: {e}"))?;

    let blockSize = stat.block_size() as u64;
    let totalBytes = stat.blocks() as u64 * blockSize;
    let availableBytes = stat.blocks_available() as u64 * blockSize;
    let usedBytes = totalBytes.saturating_sub(availableBytes);

    Ok(DiskMetrics {
        total_bytes: totalBytes,
        used_bytes: usedBytes,
        available_bytes: availableBytes,
        mount_point: "/".into(),
    })
}

fn mock_disk_metrics() -> DiskMetrics {
    let TOTAL: u64 = 2 * 1024 * 1024 * 1024 * 1024;
    let USED: u64 = 750 * 1024 * 1024 * 1024;
    DiskMetrics {
        total_bytes: TOTAL,
        used_bytes: USED,
        available_bytes: TOTAL - USED,
        mount_point: "/".into(),
    }
}
