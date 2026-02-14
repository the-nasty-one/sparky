use leptos::prelude::*;
use spark_types::ModelEntry;

#[server]
async fn get_models() -> Result<Vec<ModelEntry>, ServerFnError> {
    let models = spark_providers::models::collect().await;
    Ok(models)
}

fn format_size(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.1} GiB", b / GIB)
    } else {
        format!("{:.1} MiB", b / MIB)
    }
}

const SCANNED_DIRS: &[&str] = &[
    "/opt/models",
    "/home/auxidus-spark/.cache/huggingface/hub",
    "/home/auxidus-spark/.ollama/models",
];

#[component]
pub fn ModelsPage() -> impl IntoView {
    #[allow(unused_variables)]
    let (models, setModels) = signal(Option::<Result<Vec<ModelEntry>, String>>::None);

    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen_futures::spawn_local;

        let fetch = move || {
            spawn_local(async move {
                let result = get_models().await.map_err(|e| e.to_string());
                setModels.set(Some(result));
            });
        };

        fetch();

        let handle = set_interval_with_handle(fetch, std::time::Duration::from_secs(30))
            .expect("failed to set interval");
        on_cleanup(move || handle.clear());
    }

    view! {
        <div class="dashboard-header">
            <h1>"Models"</h1>
            <p class="subtitle">"Local model file inventory"</p>
        </div>
        {move || {
            match models.get() {
                None => {
                    view! {
                        <div class="loading">
                            <div class="spinner"></div>
                            "Scanning for models..."
                        </div>
                    }
                        .into_any()
                }
                Some(Err(e)) => {
                    view! {
                        <div class="card">
                            <p style="color: var(--danger)">"Failed to scan models: " {e}</p>
                        </div>
                    }
                        .into_any()
                }
                Some(Ok(list)) => {
                    if list.is_empty() {
                        view! {
                            <div class="card">
                                <div class="card-title">"No Models Found"</div>
                                <p style="color: var(--text-secondary); margin-bottom: 0.75rem;">
                                    "No model files were found in the scanned directories:"
                                </p>
                                <div style="display: flex; flex-direction: column; gap: 0.25rem;">
                                    {SCANNED_DIRS
                                        .iter()
                                        .map(|dir| {
                                            view! {
                                                <code style="font-size: 0.8125rem; color: var(--text-secondary);">
                                                    {*dir}
                                                </code>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            </div>
                        }
                            .into_any()
                    } else {
                        let count = list.len();
                        view! {
                            <div class="card">
                                <div class="card-title">
                                    {format!("{count} Model{}", if count == 1 { "" } else { "s" })}
                                </div>
                                <table>
                                    <thead>
                                        <tr>
                                            <th>"Name"</th>
                                            <th>"Format"</th>
                                            <th>"Size"</th>
                                            <th>"Path"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {list
                                            .into_iter()
                                            .map(|entry| {
                                                view! {
                                                    <tr>
                                                        <td>{entry.name.clone()}</td>
                                                        <td>{entry.format.clone()}</td>
                                                        <td>{format_size(entry.size_bytes)}</td>
                                                        <td
                                                            style="word-break: break-all; font-size: 0.75rem; color: var(--text-secondary);"
                                                        >
                                                            {entry.path.clone()}
                                                        </td>
                                                    </tr>
                                                }
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        }
                            .into_any()
                    }
                }
            }
        }}
    }
}
