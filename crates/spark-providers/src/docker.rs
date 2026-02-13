use spark_types::{ContainerActionResult, ContainerStatus, ContainerSummary};
use std::collections::HashMap;
use tracing::warn;

/// Parse a size string like "3.578MiB", "121.7GiB", "15.6kB", "126B" into bytes.
fn parse_size_to_bytes(s: &str) -> u64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }

    // Find where the numeric part ends and the unit begins
    let numericEnd = s
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(s.len());
    let numStr = &s[..numericEnd];
    let unit = s[numericEnd..].trim();

    let value: f64 = match numStr.parse() {
        Ok(v) => v,
        Err(_) => return 0,
    };

    let multiplier: f64 = match unit {
        "B" => 1.0,
        "kB" | "KB" => 1000.0,
        "MB" => 1_000_000.0,
        "GB" => 1_000_000_000.0,
        "TB" => 1_000_000_000_000.0,
        "KiB" => 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        "TiB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };

    (value * multiplier) as u64
}

fn parse_state(state: &str) -> ContainerStatus {
    match state.to_lowercase().as_str() {
        "running" => ContainerStatus::Running,
        "exited" => ContainerStatus::Stopped,
        "restarting" => ContainerStatus::Restarting,
        "paused" => ContainerStatus::Paused,
        "dead" => ContainerStatus::Dead,
        _ => ContainerStatus::Unknown,
    }
}

struct ContainerStats {
    cpu_pct: f64,
    memory_usage_bytes: u64,
    memory_limit_bytes: u64,
    net_rx_bytes: u64,
    net_tx_bytes: u64,
}

struct ContainerInspect {
    runtime: String,
    restart_policy: String,
    mounts: Vec<String>,
}

async fn collect_ps() -> Result<Vec<ContainerSummary>, String> {
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

        let ports = if fields[5].is_empty() {
            Vec::new()
        } else {
            fields[5].split(", ").map(|s| s.to_string()).collect()
        };

        containers.push(ContainerSummary {
            id: fields[0].to_string(),
            name: fields[1].to_string(),
            image: fields[2].to_string(),
            status: parse_state(fields[3]),
            state_text: fields[4].to_string(),
            ports,
            created: fields[6].to_string(),
            ..Default::default()
        });
    }

    Ok(containers)
}

async fn collect_stats(names: &[String]) -> HashMap<String, ContainerStats> {
    let mut map = HashMap::new();

    if names.is_empty() {
        return map;
    }

    let output = match tokio::process::Command::new("docker")
        .args([
            "stats",
            "--no-stream",
            "--format",
            "{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}",
        ])
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            warn!("failed to run docker stats: {e}");
            return map;
        }
    };

    if !output.status.success() {
        warn!(
            "docker stats failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return map;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 4 {
            continue;
        }

        let name = fields[0].to_string();

        // CPU%: strip trailing "%"
        let cpuPct: f64 = fields[1]
            .trim()
            .trim_end_matches('%')
            .parse()
            .unwrap_or(0.0);

        // MemUsage: "3.578MiB / 121.7GiB"
        let memParts: Vec<&str> = fields[2].split('/').collect();
        let memUsage = if memParts.len() >= 2 {
            parse_size_to_bytes(memParts[0])
        } else {
            0
        };
        let memLimit = if memParts.len() >= 2 {
            parse_size_to_bytes(memParts[1])
        } else {
            0
        };

        // NetIO: "15.6kB / 126B"
        let netParts: Vec<&str> = fields[3].split('/').collect();
        let netRx = if netParts.len() >= 2 {
            parse_size_to_bytes(netParts[0])
        } else {
            0
        };
        let netTx = if netParts.len() >= 2 {
            parse_size_to_bytes(netParts[1])
        } else {
            0
        };

        map.insert(
            name,
            ContainerStats {
                cpu_pct: cpuPct,
                memory_usage_bytes: memUsage,
                memory_limit_bytes: memLimit,
                net_rx_bytes: netRx,
                net_tx_bytes: netTx,
            },
        );
    }

    map
}

async fn inspect_container(id: &str) -> ContainerInspect {
    let output = match tokio::process::Command::new("docker")
        .args([
            "inspect",
            id,
            "--format",
            "{{.HostConfig.Runtime}}\t{{.HostConfig.RestartPolicy.Name}}\t{{json .Mounts}}",
        ])
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            warn!("failed to run docker inspect for {id}: {e}");
            return ContainerInspect {
                runtime: String::new(),
                restart_policy: String::new(),
                mounts: Vec::new(),
            };
        }
    };

    if !output.status.success() {
        warn!(
            "docker inspect failed for {id}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return ContainerInspect {
            runtime: String::new(),
            restart_policy: String::new(),
            mounts: Vec::new(),
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();
    let fields: Vec<&str> = line.splitn(3, '\t').collect();

    let runtime = fields.first().unwrap_or(&"").to_string();
    let restartPolicy = fields.get(1).unwrap_or(&"").to_string();
    let mountsJson = fields.get(2).unwrap_or(&"[]").to_string();

    let mounts = parse_mounts_json(&mountsJson);

    ContainerInspect {
        runtime,
        restart_policy: restartPolicy,
        mounts,
    }
}

fn parse_mounts_json(json: &str) -> Vec<String> {
    // Parse JSON array of mount objects, extract Source:Destination pairs
    let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(json);
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

pub async fn collect() -> Vec<ContainerSummary> {
    let mut containers = match collect_ps().await {
        Ok(c) => c,
        Err(e) => {
            warn!("docker not available: {e}");
            return Vec::new();
        }
    };

    // Collect names of running containers for stats
    let runningNames: Vec<String> = containers
        .iter()
        .filter(|c| c.status == ContainerStatus::Running)
        .map(|c| c.name.clone())
        .collect();

    let statsMap = collect_stats(&runningNames).await;

    // Inspect each container for runtime, restart policy, mounts
    let mut inspectResults = Vec::new();
    for c in &containers {
        inspectResults.push(inspect_container(&c.id).await);
    }

    for (i, container) in containers.iter_mut().enumerate() {
        // Merge stats if running
        if let Some(stats) = statsMap.get(&container.name) {
            container.cpu_pct = stats.cpu_pct;
            container.memory_usage_bytes = stats.memory_usage_bytes;
            container.memory_limit_bytes = stats.memory_limit_bytes;
            container.net_rx_bytes = stats.net_rx_bytes;
            container.net_tx_bytes = stats.net_tx_bytes;
        }

        // Merge inspect data
        if let Some(inspect) = inspectResults.get(i) {
            container.runtime = inspect.runtime.clone();
            container.restart_policy = inspect.restart_policy.clone();
            container.mounts = inspect.mounts.clone();
        }
    }

    containers
}

pub async fn execute_action(containerId: &str, action: &str) -> ContainerActionResult {
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
        .args([cmd, containerId])
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            return ContainerActionResult {
                success: false,
                message: format!("failed to execute docker {cmd}: {e}"),
            };
        }
    };

    if output.status.success() {
        ContainerActionResult {
            success: true,
            message: format!("container {action} successful"),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        ContainerActionResult {
            success: false,
            message: stderr,
        }
    }
}
