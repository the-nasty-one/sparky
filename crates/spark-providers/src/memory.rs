use spark_types::MemoryMetrics;
use tracing::warn;

pub async fn collect() -> MemoryMetrics {
    match read_proc_meminfo().await {
        Ok(metrics) => metrics,
        Err(e) => {
            warn!("/proc/meminfo unavailable, returning mock memory data: {e}");
            mock_memory_metrics()
        }
    }
}

async fn read_proc_meminfo() -> Result<MemoryMetrics, String> {
    let contents = tokio::fs::read_to_string("/proc/meminfo")
        .await
        .map_err(|e| format!("failed to read /proc/meminfo: {e}"))?;

    let mut memTotalKb: u64 = 0;
    let mut memAvailableKb: u64 = 0;
    let mut swapTotalKb: u64 = 0;
    let mut swapFreeKb: u64 = 0;

    for line in contents.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let valueKb = parts[1].parse::<u64>().unwrap_or(0);

        match parts[0] {
            "MemTotal:" => memTotalKb = valueKb,
            "MemAvailable:" => memAvailableKb = valueKb,
            "SwapTotal:" => swapTotalKb = valueKb,
            "SwapFree:" => swapFreeKb = valueKb,
            _ => {}
        }
    }

    let KB_TO_BYTES: u64 = 1024;
    let totalBytes = memTotalKb * KB_TO_BYTES;
    let availableBytes = memAvailableKb * KB_TO_BYTES;
    let usedBytes = totalBytes.saturating_sub(availableBytes);
    let swapTotalBytes = swapTotalKb * KB_TO_BYTES;
    let swapUsedBytes = swapTotalBytes.saturating_sub(swapFreeKb * KB_TO_BYTES);

    Ok(MemoryMetrics {
        total_bytes: totalBytes,
        used_bytes: usedBytes,
        available_bytes: availableBytes,
        swap_total_bytes: swapTotalBytes,
        swap_used_bytes: swapUsedBytes,
    })
}

fn mock_memory_metrics() -> MemoryMetrics {
    let TOTAL: u64 = 128 * 1024 * 1024 * 1024;
    let USED: u64 = 48 * 1024 * 1024 * 1024;
    MemoryMetrics {
        total_bytes: TOTAL,
        used_bytes: USED,
        available_bytes: TOTAL - USED,
        swap_total_bytes: 8 * 1024 * 1024 * 1024,
        swap_used_bytes: 512 * 1024 * 1024,
    }
}
