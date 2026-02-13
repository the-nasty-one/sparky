use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SystemMetrics {
    pub gpu: GpuMetrics,
    pub memory: MemoryMetrics,
    pub cpu: CpuMetrics,
    pub disk: DiskMetrics,
    pub uptime: UptimeMetrics,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GpuMetrics {
    pub name: String,
    pub utilization_pct: f32,
    pub temperature_c: u32,
    pub memory_used_mib: u64,
    pub memory_total_mib: u64,
    pub power_draw_w: f32,
    pub processes: Vec<GpuProcess>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GpuProcess {
    pub pid: u32,
    pub name: String,
    pub memory_mib: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CpuMetrics {
    pub load_1m: f32,
    pub load_5m: f32,
    pub load_15m: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DiskMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub mount_point: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UptimeMetrics {
    pub seconds: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            gpu: GpuMetrics::default(),
            memory: MemoryMetrics::default(),
            cpu: CpuMetrics::default(),
            disk: DiskMetrics::default(),
            uptime: UptimeMetrics::default(),
        }
    }
}

impl Default for GpuMetrics {
    fn default() -> Self {
        Self {
            name: "No GPU detected".into(),
            utilization_pct: 0.0,
            temperature_c: 0,
            memory_used_mib: 0,
            memory_total_mib: 0,
            power_draw_w: 0.0,
            processes: Vec::new(),
        }
    }
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            used_bytes: 0,
            available_bytes: 0,
            swap_total_bytes: 0,
            swap_used_bytes: 0,
        }
    }
}

impl Default for CpuMetrics {
    fn default() -> Self {
        Self {
            load_1m: 0.0,
            load_5m: 0.0,
            load_15m: 0.0,
        }
    }
}

impl Default for DiskMetrics {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            used_bytes: 0,
            available_bytes: 0,
            mount_point: "/".into(),
        }
    }
}

impl Default for UptimeMetrics {
    fn default() -> Self {
        Self { seconds: 0 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContainerSummary {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: ContainerStatus,
    pub state_text: String,
    pub cpu_pct: f64,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub ports: Vec<String>,
    pub runtime: String,
    pub restart_policy: String,
    pub created: String,
    pub mounts: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ContainerStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Dead,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContainerAction {
    pub container_id: String,
    pub action: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContainerActionResult {
    pub success: bool,
    pub message: String,
}

impl Default for ContainerSummary {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            image: String::new(),
            status: ContainerStatus::default(),
            state_text: String::new(),
            cpu_pct: 0.0,
            memory_usage_bytes: 0,
            memory_limit_bytes: 0,
            net_rx_bytes: 0,
            net_tx_bytes: 0,
            ports: Vec::new(),
            runtime: String::new(),
            restart_policy: String::new(),
            created: String::new(),
            mounts: Vec::new(),
        }
    }
}

impl Default for ContainerStatus {
    fn default() -> Self {
        ContainerStatus::Unknown
    }
}
