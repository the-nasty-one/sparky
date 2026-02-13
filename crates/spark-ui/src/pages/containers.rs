use leptos::prelude::*;
use spark_types::{ContainerActionResult, ContainerStatus, ContainerSummary};

#[server]
async fn get_containers() -> Result<Vec<ContainerSummary>, ServerFnError> {
    Ok(spark_providers::docker::collect().await)
}

#[server]
async fn container_action(
    container_id: String,
    action: String,
) -> Result<ContainerActionResult, ServerFnError> {
    Ok(spark_providers::docker::execute_action(&container_id, &action).await)
}

fn format_bytes_net(bytes: u64) -> String {
    const KB: f64 = 1000.0;
    const MB: f64 = 1_000_000.0;
    const GB: f64 = 1_000_000_000.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn format_bytes_mem(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.1} GiB", b / GIB)
    } else {
        format!("{:.1} MiB", b / MIB)
    }
}

fn status_class(status: &ContainerStatus) -> &'static str {
    match status {
        ContainerStatus::Running => "status-running",
        ContainerStatus::Stopped => "status-stopped",
        _ => "status-other",
    }
}

fn status_label(status: &ContainerStatus) -> &'static str {
    match status {
        ContainerStatus::Running => "Running",
        ContainerStatus::Stopped => "Stopped",
        ContainerStatus::Restarting => "Restarting",
        ContainerStatus::Paused => "Paused",
        ContainerStatus::Dead => "Dead",
        ContainerStatus::Unknown => "Unknown",
    }
}

#[component]
pub fn ContainersPage() -> impl IntoView {
    #[allow(unused_variables)]
    let (containers, setContainers) = signal(Option::<Result<Vec<ContainerSummary>, String>>::None);
    #[allow(unused_variables)]
    let (actionLoading, setActionLoading) = signal(Option::<String>::None);

    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen_futures::spawn_local;

        let fetch = move || {
            spawn_local(async move {
                let result = get_containers().await.map_err(|e| e.to_string());
                setContainers.set(Some(result));
            });
        };

        // Initial fetch on mount
        fetch();

        // Poll every 5 seconds
        set_interval(fetch, std::time::Duration::from_secs(5));
    }

    let handleAction = move |containerId: String, action: String| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen_futures::spawn_local;
            setActionLoading.set(Some(containerId.clone()));
            let containerId2 = containerId.clone();
            spawn_local(async move {
                let _result = container_action(containerId2, action).await;
                setActionLoading.set(None);
                // Refetch containers
                let result = get_containers().await.map_err(|e| e.to_string());
                setContainers.set(Some(result));
            });
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = (containerId, action);
        }
    };

    view! {
        <div class="dashboard-header">
            <h1>"Containers"</h1>
            <p class="subtitle">"Docker containers and live stats"</p>
        </div>
        {move || {
            let handleAction = handleAction.clone();
            match containers.get() {
                None => {
                    view! {
                        <div class="loading">
                            <div class="spinner"></div>
                            "Loading containers..."
                        </div>
                    }
                        .into_any()
                }
                Some(Ok(list)) => {
                    if list.is_empty() {
                        view! {
                            <div class="container-empty">
                                <p>"No containers found"</p>
                            </div>
                        }
                            .into_any()
                    } else {
                        let currentAction = actionLoading.get();
                        view! {
                            <div class="container-list">
                                {list
                                    .into_iter()
                                    .map(|c| {
                                        let handleAction = handleAction.clone();
                                        let isLoading = currentAction
                                            .as_ref()
                                            .map(|id| id == &c.id)
                                            .unwrap_or(false);
                                        view! {
                                            <ContainerCard
                                                container=c
                                                on_action=handleAction
                                                is_loading=isLoading
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                            .into_any()
                    }
                }
                Some(Err(e)) => {
                    view! {
                        <div class="card">
                            <p class="login-error">"Failed to load containers: " {e}</p>
                        </div>
                    }
                        .into_any()
                }
            }
        }}
    }
}

#[component]
fn ContainerCard<F>(container: ContainerSummary, on_action: F, is_loading: bool) -> impl IntoView
where
    F: Fn(String, String) + Clone + 'static,
{
    let isRunning = container.status == ContainerStatus::Running;
    let isStopped = container.status == ContainerStatus::Stopped;
    let statusCls = status_class(&container.status);
    let statusText = status_label(&container.status);

    let containerId = container.id.clone();
    let containerName = container.name.clone();
    let containerImage = container.image.clone();
    let cpuPct = container.cpu_pct;
    let memUsage = container.memory_usage_bytes;
    let memLimit = container.memory_limit_bytes;
    let netRx = container.net_rx_bytes;
    let netTx = container.net_tx_bytes;
    let runtime = container.runtime.clone();
    let restartPolicy = container.restart_policy.clone();
    let ports = container.ports.clone();
    let mounts = container.mounts.clone();

    let (detailsOpen, setDetailsOpen) = signal(false);

    let startId = containerId.clone();
    let startAction = on_action.clone();
    let stopId = containerId.clone();
    let stopAction = on_action.clone();
    let restartId = containerId.clone();
    let restartAction = on_action.clone();

    view! {
        <div class="container-card card">
            <div class="container-header">
                <div class="container-name-row">
                    <span class=format!("status-badge {statusCls}")></span>
                    <strong class="container-name">{containerName}</strong>
                    <span class="container-status-text">{statusText}</span>
                </div>
            </div>
            <div class="container-image">{containerImage}</div>

            {if isRunning {
                view! {
                    <div class="container-stats">
                        <div class="stat-pair">
                            <span class="stat-label">"CPU"</span>
                            <span class="stat-value">{format!("{:.1}%", cpuPct)}</span>
                        </div>
                        <div class="stat-pair">
                            <span class="stat-label">"Memory"</span>
                            <span class="stat-value">
                                {format!(
                                    "{} / {}",
                                    format_bytes_mem(memUsage),
                                    format_bytes_mem(memLimit),
                                )}
                            </span>
                        </div>
                        <div class="stat-pair">
                            <span class="stat-label">"Net I/O"</span>
                            <span class="stat-value">
                                {format!(
                                    "{} / {}",
                                    format_bytes_net(netRx),
                                    format_bytes_net(netTx),
                                )}
                            </span>
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <div class="container-details-toggle">
                <button
                    class="btn btn-ghost btn-sm"
                    on:click=move |_| setDetailsOpen.set(!detailsOpen.get())
                >
                    {move || if detailsOpen.get() { "Hide details" } else { "Show details" }}
                </button>
            </div>

            {move || {
                if detailsOpen.get() {
                    let portsList = ports.clone();
                    let mountsList = mounts.clone();
                    view! {
                        <div class="container-details">
                            <div class="stat-pair">
                                <span class="stat-label">"Runtime"</span>
                                <span class="stat-value">{runtime.clone()}</span>
                            </div>
                            <div class="stat-pair">
                                <span class="stat-label">"Restart Policy"</span>
                                <span class="stat-value">{restartPolicy.clone()}</span>
                            </div>
                            {if !portsList.is_empty() {
                                view! {
                                    <div class="stat-pair">
                                        <span class="stat-label">"Ports"</span>
                                        <span class="stat-value">{portsList.join(", ")}</span>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}
                            {if !mountsList.is_empty() {
                                view! {
                                    <div class="stat-pair">
                                        <span class="stat-label">"Mounts"</span>
                                        <span class="stat-value detail-mounts">
                                            {mountsList.join(", ")}
                                        </span>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}
                        </div>
                    }
                        .into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            <div class="container-actions">
                <button
                    class="btn btn-ghost btn-sm"
                    disabled=move || isRunning || is_loading
                    on:click={
                        let startAction = startAction.clone();
                        let startId = startId.clone();
                        move |_| {
                            startAction(startId.clone(), "start".to_string());
                        }
                    }
                >
                    {if is_loading { "Starting..." } else { "Start" }}
                </button>
                <button
                    class="btn btn-ghost btn-sm"
                    disabled=move || isStopped || is_loading
                    on:click={
                        let stopAction = stopAction.clone();
                        let stopId = stopId.clone();
                        move |_| {
                            stopAction(stopId.clone(), "stop".to_string());
                        }
                    }
                >
                    {if is_loading { "Stopping..." } else { "Stop" }}
                </button>
                <button
                    class="btn btn-ghost btn-sm"
                    disabled=move || !isRunning || is_loading
                    on:click={
                        let restartAction = restartAction.clone();
                        let restartId = restartId.clone();
                        move |_| {
                            restartAction(restartId.clone(), "restart".to_string());
                        }
                    }
                >
                    {if is_loading { "Restarting..." } else { "Restart" }}
                </button>
            </div>
        </div>
    }
}
