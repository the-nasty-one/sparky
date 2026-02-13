use spark_types::{ContainerActionResult, ContainerStatus, ContainerSummary};
use std::collections::HashMap;
use tracing::warn;

/// Parse a Docker size string like "3.578MiB", "121.7GiB", "15.6kB", "126B" into bytes.
fn parse_docker_size(s: &str) -> u64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }

    // Find where the numeric part ends and the unit begins
    let unitStart = s
        .find(|c: char| c.is_alphabetic())
        .unwrap_or(s.len());
    let numStr = s[..unitStart].trim();
    let unit = s[unitStart..].trim();

    let num: f64 = match numStr.parse() {
        Ok(v) => v,
        Err(_) => return 0,
    };

    let multiplier: f64 = match unit {
        "B" => 1.0,
        "kB" | "KB" => 1_000.0,
        "MB" => 1_000_000.0,
        "GB" => 1_000_000_000.0,
        "TB" => 1_000_000_000_000.0,
        "KiB" => 1_024.0,
        "MiB" => 1_048_576.0,
        "GiB" => 1_073_741_824.0,
        "TiB" => 1_099_511_627_776.0,
        _ => 1.0,
    };

    (num * multiplier) as u64
}

fn parse_status(state: &str) -> ContainerStatus {
    match state.trim().to_lowercase().as_str() {
        "running" => ContainerStatus::Running,
        "exited" => ContainerStatus::Stopped,
        "restarting" => ContainerStatus::Restarting,
        "paused" => ContainerStatus::Paused,
        "dead" => ContainerStatus::Dead,
        _ => ContainerStatus::Unknown,
    }
}

pub async fn collect() -> Vec<ContainerSummary> {
    let containers = match collect_container_list().await {
        Ok(c) => c,
        Err(e) => {
            warn!("docker ps failed: {e}");
            return Vec::new();
        }
    };

    if containers.is_empty() {
        return Vec::new();
    }

    // Collect stats for running containers
    let hasRunning = containers.iter().any(|c| c.status == ContainerStatus::Running);
    let statsMap = if hasRunning {
        collect_stats().await.unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Collect inspect data for all containers
    let ids: Vec<String> = containers.iter().map(|c| c.id.clone()).collect();
    let inspectMap = collect_inspect(&ids).await;

    // Merge everything
    containers
        .into_iter()
        .map(|mut c| {
            if let Some(stats) = statsMap.get(&c.name) {
                c.cpu_pct = stats.cpu_pct;
                c.memory_usage_bytes = stats.memory_usage_bytes;
                c.memory_limit_bytes = stats.memory_limit_bytes;
                c.net_rx_bytes = stats.net_rx_bytes;
                c.net_tx_bytes = stats.net_tx_bytes;
            }
            if let Some(inspect) = inspectMap.get(&c.id) {
                c.runtime = inspect.runtime.clone();
                c.restart_policy = inspect.restart_policy.clone();
                c.mounts = inspect.mounts.clone();
            }
            c
        })
        .collect()
}

struct StatsData {
    cpu_pct: f64,
    memory_usage_bytes: u64,
    memory_limit_bytes: u64,
    net_rx_bytes: u64,
    net_tx_bytes: u64,
}

struct InspectData {
    runtime: String,
    restart_policy: String,
    mounts: Vec<String>,
}

async fn collect_container_list() -> Result<Vec<ContainerSummary>, String> {
    let output = tokio::process::Command::new("docker")
        .args([
            "ps",
            "-a",
            "--format",
            "{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.State}}\t{{.Status}}\t{{.Ports}}\t{{.CreatedAt}}",
        ])
        .output()
        .await
        .map_err(|e| format!("failed to run docker ps: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("docker ps failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 7 {
            warn!("unexpected docker ps line format: {line}");
            continue;
        }

        let id = fields[0].trim().to_string();
        let name = fields[1].trim().to_string();
        let image = fields[2].trim().to_string();
        let state = fields[3].trim();
        let statusText = fields[4].trim().to_string();
        let portsRaw = fields[5].trim();
        let created = fields[6].trim().to_string();

        let ports = if portsRaw.is_empty() {
            Vec::new()
        } else {
            portsRaw.split(", ").map(|s| s.to_string()).collect()
        };

        containers.push(ContainerSummary {
            id,
            name,
            image,
            status: parse_status(state),
            state_text: statusText,
            ports,
            created,
            ..Default::default()
        });
    }

    Ok(containers)
}

async fn collect_stats() -> Result<HashMap<String, StatsData>, String> {
    let output = tokio::process::Command::new("docker")
        .args([
            "stats",
            "--no-stream",
            "--format",
            "{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}",
        ])
        .output()
        .await
        .map_err(|e| format!("failed to run docker stats: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("docker stats failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut map = HashMap::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 4 {
            continue;
        }

        let name = fields[0].trim().to_string();

        // CPU%: strip trailing "%"
        let cpuStr = fields[1].trim().trim_end_matches('%');
        let cpuPct: f64 = cpuStr.parse().unwrap_or(0.0);

        // MemUsage: "3.578MiB / 121.7GiB"
        let (memUsage, memLimit) = if let Some((used, limit)) = fields[2].split_once('/') {
            (parse_docker_size(used), parse_docker_size(limit))
        } else {
            (0, 0)
        };

        // NetIO: "15.6kB / 126B"
        let (netRx, netTx) = if let Some((rx, tx)) = fields[3].split_once('/') {
            (parse_docker_size(rx), parse_docker_size(tx))
        } else {
            (0, 0)
        };

        map.insert(
            name,
            StatsData {
                cpu_pct: cpuPct,
                memory_usage_bytes: memUsage,
                memory_limit_bytes: memLimit,
                net_rx_bytes: netRx,
                net_tx_bytes: netTx,
            },
        );
    }

    Ok(map)
}

async fn collect_inspect(ids: &[String]) -> HashMap<String, InspectData> {
    let mut map = HashMap::new();

    for id in ids {
        match inspect_one(id).await {
            Ok(data) => {
                map.insert(id.clone(), data);
            }
            Err(e) => {
                warn!("docker inspect failed for {id}: {e}");
            }
        }
    }

    map
}

async fn inspect_one(id: &str) -> Result<InspectData, String> {
    let output = tokio::process::Command::new("docker")
        .args([
            "inspect",
            id,
            "--format",
            "{{.HostConfig.Runtime}}\t{{.HostConfig.RestartPolicy.Name}}\t{{json .Mounts}}",
        ])
        .output()
        .await
        .map_err(|e| format!("failed to run docker inspect: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("docker inspect failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();
    let fields: Vec<&str> = line.splitn(3, '\t').collect();

    let runtime = fields.first().map(|s| s.trim().to_string()).unwrap_or_default();
    let restartPolicy = fields.get(1).map(|s| s.trim().to_string()).unwrap_or_default();
    let mountsJson = fields.get(2).map(|s| s.trim()).unwrap_or("[]");

    let mounts = parse_mounts_json(mountsJson);

    Ok(InspectData {
        runtime,
        restart_policy: restartPolicy,
        mounts,
    })
}

fn parse_mounts_json(raw: &str) -> Vec<String> {
    // Parse as JSON array of objects with "Source" and "Destination" fields
    let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(raw);
    match parsed {
        Ok(arr) => arr
            .iter()
            .filter_map(|m| {
                let src = m.get("Source")?.as_str()?;
                let dst = m.get("Destination")?.as_str()?;
                Some(format!("{src}:{dst}"))
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub async fn execute_action(container_id: &str, action: &str) -> ContainerActionResult {
    let cmd = match action {
        "start" | "stop" | "restart" => action,
        _ => {
            return ContainerActionResult {
                success: false,
                message: format!("unknown action: {action}"),
            };
        }
    };

    let output = match tokio::process::Command::new("docker")
        .args([cmd, container_id])
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            return ContainerActionResult {
                success: false,
                message: format!("failed to run docker {cmd}: {e}"),
            };
        }
    };

    if output.status.success() {
        ContainerActionResult {
            success: true,
            message: format!("docker {cmd} {container_id} succeeded"),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ContainerActionResult {
            success: false,
            message: format!("docker {cmd} failed: {stderr}"),
        }
    }
}
