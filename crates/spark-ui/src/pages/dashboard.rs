use leptos::prelude::*;
use spark_types::{GpuProcess, SystemMetrics};

use crate::components::gauge::Gauge;
use crate::components::metric_card::MetricCard;

#[server]
async fn get_system_metrics() -> Result<SystemMetrics, ServerFnError> {
    use spark_providers::collect_system_metrics;
    Ok(collect_system_metrics().await)
}

fn format_bytes(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const TIB: f64 = GIB * 1024.0;
    let bytesF64 = bytes as f64;
    if bytesF64 >= TIB {
        format!("{:.1} TiB", bytesF64 / TIB)
    } else {
        format!("{:.1} GiB", bytesF64 / GIB)
    }
}

fn format_mib(mib: u64) -> String {
    if mib >= 1024 {
        format!("{:.1} GiB", mib as f64 / 1024.0)
    } else {
        format!("{mib} MiB")
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    format!("{days}d {hours}h {minutes}m")
}

fn gauge_color(value: f32) -> &'static str {
    if value >= 90.0 {
        "#ef4444"
    } else if value >= 70.0 {
        "#f59e0b"
    } else {
        "#76b900"
    }
}

fn temp_gauge_color(tempC: u32) -> &'static str {
    if tempC >= 80 {
        "#ef4444"
    } else if tempC >= 65 {
        "#f59e0b"
    } else {
        "#76b900"
    }
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    // Hold latest metrics in a signal — never re-enters loading after first data arrives.
    #[allow(unused_variables)]
    let (metrics, setMetrics) = signal(Option::<Result<SystemMetrics, String>>::None);

    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen_futures::spawn_local;

        let fetch = move || {
            spawn_local(async move {
                let result = get_system_metrics().await.map_err(|e| e.to_string());
                setMetrics.set(Some(result));
            });
        };

        // Initial fetch on mount
        fetch();

        // Poll every 2 seconds — updates the signal in place, no flicker
        let handle = set_interval_with_handle(fetch, std::time::Duration::from_secs(2))
            .expect("failed to set interval");
        on_cleanup(move || handle.clear());
    }

    view! {
        <div class="dashboard-header">
            <h1>"System Dashboard"</h1>
            <p class="subtitle">"DGX Spark real-time metrics"</p>
        </div>
        {move || {
            match metrics.get() {
                None => {
                    view! {
                        <div class="loading">
                            <div class="spinner"></div>
                            "Loading system metrics..."
                        </div>
                    }
                        .into_any()
                }
                Some(Ok(m)) => {
                    view! { <DashboardContent metrics=m /> }.into_any()
                }
                Some(Err(e)) => {
                    view! {
                        <div class="card">
                            <p class="login-error">"Failed to load metrics: " {e}</p>
                        </div>
                    }
                        .into_any()
                }
            }
        }}
    }
}

#[component]
fn DashboardContent(metrics: SystemMetrics) -> impl IntoView {
    let gpuUtilization = metrics.gpu.utilization_pct;
    let gpuTemp = metrics.gpu.temperature_c;
    let gpuMemUsed = metrics.gpu.memory_used_mib;
    let gpuMemTotal = metrics.gpu.memory_total_mib;
    let gpuMemPct = if gpuMemTotal > 0 {
        (gpuMemUsed as f32 / gpuMemTotal as f32) * 100.0
    } else {
        0.0
    };
    let gpuPower = metrics.gpu.power_draw_w;
    let gpuName = metrics.gpu.name.clone();
    let gpuProcesses = metrics.gpu.processes.clone();
    let gpuUnifiedMemory = metrics.gpu.unified_memory;

    // Temperature: normalize to 0-100 scale where 30°C = 0% and 90°C = 100%
    let tempNormalized = ((gpuTemp as f32 - 30.0) / 60.0 * 100.0).clamp(0.0, 100.0);

    let memUsed = metrics.memory.used_bytes;
    let memTotal = metrics.memory.total_bytes;
    let memPct = if memTotal > 0 {
        (memUsed as f64 / memTotal as f64 * 100.0) as f32
    } else {
        0.0
    };

    let diskUsed = metrics.disk.used_bytes;
    let diskTotal = metrics.disk.total_bytes;
    let diskPct = if diskTotal > 0 {
        (diskUsed as f64 / diskTotal as f64 * 100.0) as f32
    } else {
        0.0
    };

    let uptimeFormatted = format_uptime(metrics.uptime.seconds);

    // GPU Memory card: branch on unified memory
    let gpuMemoryCard = if gpuUnifiedMemory {
        view! {
            <MetricCard title="GPU Memory".to_string()>
                <div class="gauge-container">
                    <div class="uptime-display">"Unified Memory"</div>
                    <div class="gauge-label">{format_mib(gpuMemTotal)} " total"</div>
                    <div class="gauge-label">"Per-GPU VRAM tracking not available"</div>
                </div>
            </MetricCard>
        }
            .into_any()
    } else {
        view! {
            <MetricCard title="GPU Memory".to_string()>
                <Gauge
                    value=gpuMemPct
                    label=format!("{} / {} MiB", gpuMemUsed, gpuMemTotal)
                    unit="%".to_string()
                    color=gauge_color(gpuMemPct).to_string()
                />
            </MetricCard>
        }
            .into_any()
    };

    view! {
        <div class="dashboard-grid">
            <MetricCard title="GPU Utilization".to_string()>
                <Gauge
                    value=gpuUtilization
                    label=gpuName.clone()
                    unit="%".to_string()
                    color=gauge_color(gpuUtilization).to_string()
                />
            </MetricCard>

            <MetricCard title="GPU Temperature".to_string()>
                <Gauge
                    value=tempNormalized
                    label="Temperature".to_string()
                    unit="\u{00B0}C".to_string()
                    color=temp_gauge_color(gpuTemp).to_string()
                    display_value=format!("{gpuTemp}")
                />
            </MetricCard>

            {gpuMemoryCard}

            <MetricCard title="GPU Power".to_string()>
                <div class="gauge-container">
                    <div class="uptime-display">{format!("{:.0} W", gpuPower)}</div>
                    <div class="gauge-label">"Power Draw"</div>
                </div>
            </MetricCard>

            <MetricCard title="System Memory".to_string()>
                <Gauge
                    value=memPct
                    label=format!("{} / {}", format_bytes(memUsed), format_bytes(memTotal))
                    unit="%".to_string()
                    color=gauge_color(memPct).to_string()
                />
            </MetricCard>

            <MetricCard title="CPU Load".to_string()>
                <div class="metric-row">
                    <span class="metric-label">"1 min"</span>
                    <span class="metric-value">{format!("{:.2}", metrics.cpu.load_1m)}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">"5 min"</span>
                    <span class="metric-value">{format!("{:.2}", metrics.cpu.load_5m)}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">"15 min"</span>
                    <span class="metric-value">{format!("{:.2}", metrics.cpu.load_15m)}</span>
                </div>
            </MetricCard>

            <MetricCard title="Disk Usage".to_string()>
                <Gauge
                    value=diskPct
                    label=format!(
                        "{} / {}",
                        format_bytes(diskUsed),
                        format_bytes(diskTotal),
                    )
                    unit="%".to_string()
                    color=gauge_color(diskPct).to_string()
                />
            </MetricCard>

            <MetricCard title="Uptime".to_string()>
                <div class="gauge-container">
                    <div class="uptime-display">{uptimeFormatted}</div>
                    <div class="gauge-label">"System Uptime"</div>
                </div>
            </MetricCard>
        </div>

        <GpuProcessTable processes=gpuProcesses />
    }
}

#[component]
fn GpuProcessTable(processes: Vec<GpuProcess>) -> impl IntoView {
    view! {
        <div class="process-section">
            <div class="card">
                <div class="card-title">"GPU Processes"</div>
                <table>
                    <thead>
                        <tr>
                            <th>"PID"</th>
                            <th>"Process"</th>
                            <th>"GPU Memory"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {if processes.is_empty() {
                            view! {
                                <tr>
                                    <td colspan="3">"No GPU processes running"</td>
                                </tr>
                            }
                                .into_any()
                        } else {
                            processes
                                .into_iter()
                                .map(|process| {
                                    view! {
                                        <tr>
                                            <td>{process.pid}</td>
                                            <td>{process.name.clone()}</td>
                                            <td>{format!("{} MiB", process.memory_mib)}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
