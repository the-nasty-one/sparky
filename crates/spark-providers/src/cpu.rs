use spark_types::CpuMetrics;
use tracing::warn;

pub async fn collect() -> CpuMetrics {
    match read_proc_loadavg().await {
        Ok(metrics) => metrics,
        Err(e) => {
            warn!("/proc/loadavg unavailable, returning mock CPU data: {e}");
            mock_cpu_metrics()
        }
    }
}

async fn read_proc_loadavg() -> Result<CpuMetrics, String> {
    let contents = tokio::fs::read_to_string("/proc/loadavg")
        .await
        .map_err(|e| format!("failed to read /proc/loadavg: {e}"))?;

    let fields: Vec<&str> = contents.split_whitespace().collect();
    if fields.len() < 3 {
        return Err(format!("unexpected /proc/loadavg format: {contents}"));
    }

    let load1m = fields[0].parse::<f32>().unwrap_or(0.0);
    let load5m = fields[1].parse::<f32>().unwrap_or(0.0);
    let load15m = fields[2].parse::<f32>().unwrap_or(0.0);

    Ok(CpuMetrics {
        load_1m: load1m,
        load_5m: load5m,
        load_15m: load15m,
    })
}

fn mock_cpu_metrics() -> CpuMetrics {
    CpuMetrics {
        load_1m: 2.45,
        load_5m: 1.89,
        load_15m: 1.32,
    }
}
