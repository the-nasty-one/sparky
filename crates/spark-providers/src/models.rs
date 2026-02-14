#![allow(non_snake_case)]

use spark_types::ModelEntry;
use tokio::fs;
use tracing::warn;

const DEFAULT_MODEL_DIRS: &[&str] = &[
    "/opt/models",
    "/home/auxidus-spark/.cache/huggingface/hub",
    "/home/auxidus-spark/.ollama/models",
];

const MODEL_EXTENSIONS: &[&str] = &[
    "gguf", "safetensors", "bin", "pt", "pth", "onnx", "ckpt",
];

pub async fn collect() -> Vec<ModelEntry> {
    let mut entries = Vec::new();
    for dir in DEFAULT_MODEL_DIRS {
        if let Err(e) = scan_dir(dir, &mut entries).await {
            warn!("failed to scan {dir}: {e}");
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

async fn scan_dir(dir: &str, entries: &mut Vec<ModelEntry>) -> Result<(), String> {
    let mut stack = vec![std::path::PathBuf::from(dir)];

    while let Some(path) = stack.pop() {
        let mut readDir = match fs::read_dir(&path).await {
            Ok(rd) => rd,
            Err(_) => continue,
        };

        while let Ok(Some(entry)) = readDir.next_entry().await {
            let entryPath = entry.path();
            if entryPath.is_dir() {
                stack.push(entryPath);
                continue;
            }

            let ext = entryPath
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            if !MODEL_EXTENSIONS.contains(&ext) {
                continue;
            }

            let metadata = match fs::metadata(&entryPath).await {
                Ok(m) => m,
                Err(_) => continue,
            };

            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| {
                            let secs = d.as_secs();
                            format!("{secs}")
                        })
                })
                .unwrap_or_default();

            entries.push(ModelEntry {
                name: entryPath
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                path: entryPath.to_string_lossy().to_string(),
                size_bytes: metadata.len(),
                format: ext.to_uppercase(),
                modified,
            });
        }
    }

    Ok(())
}
