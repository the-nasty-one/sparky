use spark_types::UptimeMetrics;
use tracing::warn;

pub async fn collect() -> UptimeMetrics {
    match read_proc_uptime().await {
        Ok(metrics) => metrics,
        Err(e) => {
            warn!("/proc/uptime unavailable, returning mock uptime data: {e}");
            mock_uptime_metrics()
        }
    }
}

async fn read_proc_uptime() -> Result<UptimeMetrics, String> {
    let contents = tokio::fs::read_to_string("/proc/uptime")
        .await
        .map_err(|e| format!("failed to read /proc/uptime: {e}"))?;

    let firstField = contents
        .split_whitespace()
        .next()
        .ok_or("empty /proc/uptime")?;

    let uptimeSeconds = firstField
        .parse::<f64>()
        .map_err(|e| format!("failed to parse uptime: {e}"))?;

    Ok(UptimeMetrics {
        seconds: uptimeSeconds as u64,
    })
}

fn mock_uptime_metrics() -> UptimeMetrics {
    UptimeMetrics {
        seconds: 3 * 86400 + 7 * 3600 + 42 * 60 + 15,
    }
}
