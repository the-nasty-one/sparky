use leptos::prelude::*;
use spark_types::{ContainerActionResult, ContainerStatus, ContainerSummary};

#[server]
async fn get_containers() -> Result<Vec<ContainerSummary>, ServerFnError> {
    spark_providers::docker::collect()
        .await
        .map_err(|e| ServerFnError::new(e))
}

#[server]
async fn container_action(
    container_id: String,
    action: String,
) -> Result<ContainerActionResult, ServerFnError> {
    Ok(spark_providers::docker::execute_action(&container_id, &action).await)
}

fn format_net_bytes(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= 1_000_000_000.0 {
        format!("{:.1} GB", b / 1_000_000_000.0)
    } else if b >= 1_000_000.0 {
        format!("{:.1} MB", b / 1_000_000.0)
    } else if b >= 1_000.0 {
        format!("{:.1} KB", b / 1_000.0)
    } else {
        format!("{bytes} B")
    }
}

fn format_mem_bytes(bytes: u64) -> String {
    const GIB: f64 = 1_073_741_824.0;
    const MIB: f64 = 1_048_576.0;
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
    let (containers, setContainers) =
        signal(Option::<Result<Vec<ContainerSummary>, String>>::None);
    #[allow(unused_variables)]
    let (pendingAction, setPendingAction) = signal(Option::<String>::None);
    #[allow(unused_variables)]
    let (actionError, setActionError) = signal(Option::<String>::None);
    #[allow(unused_variables)]
    let (expandedIds, setExpandedIds) = signal(Vec::<String>::new());

    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen_futures::spawn_local;

        let fetch = move || {
            spawn_local(async move {
                let result = get_containers().await.map_err(|e| e.to_string());
                setContainers.set(Some(result));
            });
        };

        fetch();
        let handle = set_interval_with_handle(fetch, std::time::Duration::from_secs(5))
            .expect("failed to set interval");
        on_cleanup(move || handle.clear());
    }

    view! {
        <div class="dashboard-header">
            <h1>"Containers"</h1>
            <p class="subtitle">"Docker container management"</p>
        </div>
        {move || {
            actionError.get().map(|msg| {
                view! {
                    <div class="container-action-error">
                        <p>{msg}</p>
                    </div>
                }
            })
        }}
        {move || {
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
                Some(Err(e)) => {
                    view! {
                        <div class="card">
                            <p style="color: var(--danger)">"Failed to load containers: " {e}</p>
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
                        let items = list
                            .into_iter()
                            .map(|c| {
                                let containerId = c.id.clone();
                                let containerName = c.name.clone();
                                let containerImage = c.image.clone();
                                let containerStatus = c.status.clone();
                                let stateText = c.state_text.clone();
                                let cpuPct = c.cpu_pct;
                                let memUsage = c.memory_usage_bytes;
                                let memLimit = c.memory_limit_bytes;
                                let netRx = c.net_rx_bytes;
                                let netTx = c.net_tx_bytes;
                                let ports = c.ports.clone();
                                let runtime = c.runtime.clone();
                                let restartPolicy = c.restart_policy.clone();
                                let created = c.created.clone();
                                let mounts = c.mounts.clone();
                                let isRunning = containerStatus == ContainerStatus::Running;
                                let isStopped = containerStatus == ContainerStatus::Stopped;
                                let statusCls = status_class(&containerStatus);
                                let statusLbl = status_label(&containerStatus);

                                // Clone IDs for each closure that needs them
                                let idForToggle = containerId.clone();

                                let toggleExpand = move |_| {
                                    let id = idForToggle.clone();
                                    setExpandedIds.update(|ids| {
                                        if let Some(pos) = ids.iter().position(|x| x == &id) {
                                            ids.remove(pos);
                                        } else {
                                            ids.push(id);
                                        }
                                    });
                                };

                                #[allow(unused_variables)]
                                let makeAction = {
                                    let containerId = containerId.clone();
                                    move |action: &'static str| {
                                        let cid = containerId.clone();
                                        move |_| {
                                            let cid = cid.clone();
                                            setActionError.set(None);
                                            setPendingAction.set(Some(cid.clone()));
                                            #[cfg(feature = "hydrate")]
                                            {
                                                use wasm_bindgen_futures::spawn_local;
                                                let cid2 = cid.clone();
                                                spawn_local(async move {
                                                    match container_action(
                                                        cid2,
                                                        action.to_string(),
                                                    )
                                                    .await
                                                    {
                                                        Ok(res) if !res.success => {
                                                            setActionError.set(Some(res.message));
                                                        }
                                                        Err(e) => {
                                                            setActionError
                                                                .set(Some(e.to_string()));
                                                        }
                                                        _ => {}
                                                    }
                                                    let result = get_containers()
                                                        .await
                                                        .map_err(|e| e.to_string());
                                                    setContainers.set(Some(result));
                                                    setPendingAction.set(None);
                                                });
                                            }
                                        }
                                    }
                                };

                                let onStart = makeAction("start");
                                let onStop = makeAction("stop");
                                let onRestart = makeAction("restart");

                                let hasDetails = !ports.is_empty()
                                    || !runtime.is_empty()
                                    || !restartPolicy.is_empty()
                                    || !mounts.is_empty();

                                // Clone containerId for each closure that checks pending
                                let idPend1 = containerId.clone();
                                let idPend2 = containerId.clone();
                                let idPend3 = containerId.clone();
                                let idPend4 = containerId.clone();
                                let idPend5 = containerId.clone();
                                let idPend6 = containerId.clone();

                                // Clone containerId for each closure that checks expanded
                                let idExp1 = containerId.clone();
                                let idExp2 = containerId.clone();

                                view! {
                                    <div class="container-card card">
                                        <div class="container-header">
                                            <div class="container-name-row">
                                                <span class=format!(
                                                    "status-badge {statusCls}",
                                                )></span>
                                                <span class="container-name">{containerName}</span>
                                                <span class="container-status-text">{statusLbl}</span>
                                            </div>
                                            <span class="container-state-detail">{stateText}</span>
                                        </div>
                                        <div class="container-image">{containerImage}</div>

                                        {if isRunning {
                                            view! {
                                                <div class="container-stats">
                                                    <div class="stat-pair">
                                                        <span class="stat-label">"CPU"</span>
                                                        <span class="stat-value">
                                                            {format!("{:.1}%", cpuPct)}
                                                        </span>
                                                    </div>
                                                    <div class="stat-pair">
                                                        <span class="stat-label">"Memory"</span>
                                                        <span class="stat-value">
                                                            {format!(
                                                                "{} / {}",
                                                                format_mem_bytes(memUsage),
                                                                format_mem_bytes(memLimit),
                                                            )}
                                                        </span>
                                                    </div>
                                                    <div class="stat-pair">
                                                        <span class="stat-label">"Net I/O"</span>
                                                        <span class="stat-value">
                                                            {format!(
                                                                "{} / {}",
                                                                format_net_bytes(netRx),
                                                                format_net_bytes(netTx),
                                                            )}
                                                        </span>
                                                    </div>
                                                </div>
                                            }
                                                .into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}

                                        <div class="container-actions">
                                            <button
                                                class="btn btn-sm btn-ghost"
                                                disabled=move || {
                                                    isRunning
                                                        || pendingAction.get().as_ref() == Some(&idPend1)
                                                }
                                                on:click=onStart
                                            >
                                                {move || {
                                                    if pendingAction.get().as_ref() == Some(&idPend2) {
                                                        "Starting..."
                                                    } else {
                                                        "Start"
                                                    }
                                                }}
                                            </button>
                                            <button
                                                class="btn btn-sm btn-ghost"
                                                disabled=move || {
                                                    isStopped
                                                        || pendingAction.get().as_ref() == Some(&idPend3)
                                                }
                                                on:click=onStop
                                            >
                                                {move || {
                                                    if pendingAction.get().as_ref() == Some(&idPend4) {
                                                        "Stopping..."
                                                    } else {
                                                        "Stop"
                                                    }
                                                }}
                                            </button>
                                            <button
                                                class="btn btn-sm btn-ghost"
                                                disabled=move || {
                                                    !isRunning
                                                        || pendingAction.get().as_ref() == Some(&idPend5)
                                                }
                                                on:click=onRestart
                                            >
                                                {move || {
                                                    if pendingAction.get().as_ref() == Some(&idPend6) {
                                                        "Restarting..."
                                                    } else {
                                                        "Restart"
                                                    }
                                                }}
                                            </button>
                                            {if hasDetails {
                                                view! {
                                                    <button
                                                        class="btn btn-sm btn-ghost"
                                                        on:click=toggleExpand
                                                    >
                                                        {move || {
                                                            if expandedIds.get().contains(&idExp1) {
                                                                "Hide Details"
                                                            } else {
                                                                "Details"
                                                            }
                                                        }}
                                                    </button>
                                                }
                                                    .into_any()
                                            } else {
                                                view! {}.into_any()
                                            }}
                                        </div>

                                        {if hasDetails {
                                            let ports = ports.clone();
                                            let runtime = runtime.clone();
                                            let restartPolicy = restartPolicy.clone();
                                            let mounts = mounts.clone();
                                            let created = created.clone();
                                            view! {
                                                <div
                                                    class="container-details"
                                                    style=move || {
                                                        if expandedIds.get().contains(&idExp2) {
                                                            "display: block"
                                                        } else {
                                                            "display: none"
                                                        }
                                                    }
                                                >
                                                    {if !runtime.is_empty() {
                                                        view! {
                                                            <div class="detail-row">
                                                                <span class="detail-label">"Runtime"</span>
                                                                <span class="detail-value">
                                                                    {runtime.clone()}
                                                                </span>
                                                            </div>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! {}.into_any()
                                                    }}
                                                    {if !restartPolicy.is_empty() {
                                                        view! {
                                                            <div class="detail-row">
                                                                <span class="detail-label">
                                                                    "Restart Policy"
                                                                </span>
                                                                <span class="detail-value">
                                                                    {restartPolicy.clone()}
                                                                </span>
                                                            </div>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! {}.into_any()
                                                    }}
                                                    {if !created.is_empty() {
                                                        view! {
                                                            <div class="detail-row">
                                                                <span class="detail-label">"Created"</span>
                                                                <span class="detail-value">
                                                                    {created.clone()}
                                                                </span>
                                                            </div>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! {}.into_any()
                                                    }}
                                                    {if !ports.is_empty() {
                                                        let portList = ports
                                                            .iter()
                                                            .map(|p| {
                                                                view! {
                                                                    <div class="detail-tag">
                                                                        {p.clone()}
                                                                    </div>
                                                                }
                                                            })
                                                            .collect_view();
                                                        view! {
                                                            <div class="detail-row">
                                                                <span class="detail-label">"Ports"</span>
                                                                <div class="detail-tags">
                                                                    {portList}
                                                                </div>
                                                            </div>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! {}.into_any()
                                                    }}
                                                    {if !mounts.is_empty() {
                                                        let mountList = mounts
                                                            .iter()
                                                            .map(|m| {
                                                                view! {
                                                                    <div class="detail-tag">
                                                                        {m.clone()}
                                                                    </div>
                                                                }
                                                            })
                                                            .collect_view();
                                                        view! {
                                                            <div class="detail-row">
                                                                <span class="detail-label">"Mounts"</span>
                                                                <div class="detail-tags">
                                                                    {mountList}
                                                                </div>
                                                            </div>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! {}.into_any()
                                                    }}
                                                </div>
                                            }
                                                .into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}
                                    </div>
                                }
                            })
                            .collect_view();
                        view! { <div class="container-list">{items}</div> }.into_any()
                    }
                }
            }
        }}
    }
}
