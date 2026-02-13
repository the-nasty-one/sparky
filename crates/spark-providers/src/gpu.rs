use spark_types::{GpuMetrics, GpuProcess};
use tracing::warn;

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
    let utilizationPct = gpuFields[1]
        .trim()
        .parse::<f32>()
        .unwrap_or(0.0);
    let temperatureC = gpuFields[2]
        .trim()
        .parse::<u32>()
        .unwrap_or(0);
    let memoryUsedMib = gpuFields[3]
        .trim()
        .parse::<u64>()
        .unwrap_or(0);
    let memoryTotalMib = gpuFields[4]
        .trim()
        .parse::<u64>()
        .unwrap_or(0);
    let powerDrawW = gpuFields[5]
        .trim()
        .parse::<f32>()
        .unwrap_or(0.0);

    let processes = collect_gpu_processes().await.unwrap_or_default();

    Ok(GpuMetrics {
        name,
        utilization_pct: utilizationPct,
        temperature_c: temperatureC,
        memory_used_mib: memoryUsedMib,
        memory_total_mib: memoryTotalMib,
        power_draw_w: powerDrawW,
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
            let pid = fields[0].trim().parse::<u32>().unwrap_or(0);
            let name = fields[1].trim().to_string();
            let memoryMib = fields[2].trim().parse::<u64>().unwrap_or(0);

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
