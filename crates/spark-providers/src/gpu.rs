use spark_types::{GpuMetrics, GpuProcess};
use tracing::warn;

/// Try to parse a numeric value from an nvidia-smi field.
/// Strips brackets, whitespace, and unit suffixes (e.g. "MiB", "W").
/// Returns None for N/A variants like "[N/A]", "N/A", "N/A MiB", etc.
fn parse_nvsmi_field<T: std::str::FromStr>(raw: &str) -> Option<T> {
    let s = raw.trim().trim_matches(|c| c == '[' || c == ']').trim();
    if s.eq_ignore_ascii_case("n/a") || s.is_empty() {
        return None;
    }
    // Strip trailing unit suffixes like "MiB", "W", "%" so we can parse the number
    let numeric = s.split_whitespace().next().unwrap_or(s);
    numeric.parse::<T>().ok()
}

/// Read MemTotal from /proc/meminfo and return it in MiB.
async fn read_proc_meminfo_total_mib() -> Option<u64> {
    let contents = tokio::fs::read_to_string("/proc/meminfo").await.ok()?;
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            // Value is typically in kB, e.g. "MemTotal:       131841024 kB"
            let kb: u64 = rest
                .trim()
                .split_whitespace()
                .next()?
                .parse()
                .ok()?;
            return Some(kb / 1024);
        }
    }
    None
}

pub async fn collect() -> GpuMetrics {
    match collect_from_nvidia_smi().await {
        Ok(metrics) => metrics,
        Err(e) => {
            warn!("nvidia-smi unavailable, returning mock GPU data: {e}");
            mock_gpu_metrics()
        }
    }
}

async fn collect_from_nvidia_smi() -> Result<GpuMetrics, String> {
    let gpuOutput = tokio::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,temperature.gpu,memory.used,memory.total,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .await
        .map_err(|e| format!("failed to run nvidia-smi: {e}"))?;

    if !gpuOutput.status.success() {
        return Err(format!(
            "nvidia-smi exited with status {}",
            gpuOutput.status
        ));
    }

    let gpuCsv = String::from_utf8_lossy(&gpuOutput.stdout);
    let gpuLine = gpuCsv.lines().next().ok_or("empty nvidia-smi output")?;
    let gpuFields: Vec<&str> = gpuLine.split(", ").collect();

    if gpuFields.len() < 6 {
        return Err(format!(
            "unexpected nvidia-smi output format: {}",
            gpuLine
        ));
    }

    let name = gpuFields[0].trim().to_string();
    let utilizationPct = parse_nvsmi_field::<f32>(gpuFields[1]).unwrap_or_else(|| {
        warn!("could not parse GPU utilization '{}'", gpuFields[1].trim());
        0.0
    });
    let temperatureC = parse_nvsmi_field::<u32>(gpuFields[2]).unwrap_or_else(|| {
        warn!("could not parse GPU temperature '{}'", gpuFields[2].trim());
        0
    });

    // On unified-memory systems (e.g. DGX Spark GB10), nvidia-smi returns [N/A]
    // for memory fields. Fall back to /proc/meminfo for total memory.
    let memoryUsedMib = parse_nvsmi_field::<u64>(gpuFields[3]).unwrap_or(0);
    let mut unifiedMemory = false;
    let memoryTotalMib = match parse_nvsmi_field::<u64>(gpuFields[4]) {
        Some(v) => v,
        None => {
            warn!(
                "nvidia-smi memory.total is N/A ('{}'), falling back to /proc/meminfo",
                gpuFields[4].trim()
            );
            unifiedMemory = true;
            read_proc_meminfo_total_mib().await.unwrap_or(0)
        }
    };

    let powerDrawW = parse_nvsmi_field::<f32>(gpuFields[5]).unwrap_or_else(|| {
        warn!("could not parse GPU power draw '{}'", gpuFields[5].trim());
        0.0
    });

    let processes = collect_gpu_processes().await.unwrap_or_default();

    Ok(GpuMetrics {
        name,
        utilization_pct: utilizationPct,
        temperature_c: temperatureC,
        memory_used_mib: memoryUsedMib,
        memory_total_mib: memoryTotalMib,
        power_draw_w: powerDrawW,
        unified_memory: unifiedMemory,
        processes,
    })
}

async fn collect_gpu_processes() -> Result<Vec<GpuProcess>, String> {
    let processOutput = tokio::process::Command::new("nvidia-smi")
        .args([
            "--query-compute-apps=pid,process_name,used_gpu_memory",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .await
        .map_err(|e| format!("failed to query GPU processes: {e}"))?;

    if !processOutput.status.success() {
        return Ok(Vec::new());
    }

    let processCsv = String::from_utf8_lossy(&processOutput.stdout);
    let mut processes = Vec::new();

    for line in processCsv.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split(", ").collect();
        if fields.len() >= 3 {
            let pid = fields[0].trim().parse::<u32>()
                .inspect_err(|e| warn!("failed to parse GPU process PID '{}': {e}", fields[0].trim()))
                .unwrap_or(0);
            let name = fields[1].trim().to_string();
            let memoryMib = fields[2].trim().parse::<u64>()
                .inspect_err(|e| warn!("failed to parse GPU process memory '{}': {e}", fields[2].trim()))
                .unwrap_or(0);

            processes.push(GpuProcess {
                pid,
                name,
                memory_mib: memoryMib,
            });
        }
    }

    Ok(processes)
}

fn mock_gpu_metrics() -> GpuMetrics {
    GpuMetrics {
        name: "NVIDIA GH200 (mock)".into(),
        utilization_pct: 42.0,
        temperature_c: 55,
        memory_used_mib: 15360,
        memory_total_mib: 98304,
        power_draw_w: 185.0,
        unified_memory: false,
        processes: vec![
            GpuProcess {
                pid: 1234,
                name: "python3".into(),
                memory_mib: 8192,
            },
            GpuProcess {
                pid: 5678,
                name: "comfyui".into(),
                memory_mib: 4096,
            },
            GpuProcess {
                pid: 9012,
                name: "ollama".into(),
                memory_mib: 3072,
            },
        ],
    }
}
