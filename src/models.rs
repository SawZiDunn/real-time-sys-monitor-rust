use serde::{Deserialize, Serialize};
use sysinfo::{Disks, Networks, System};

// Messages for the application
#[derive(Debug, Clone)]
pub enum Message {
    IntervalChanged(String),
    Tick,
    ToggleMonitoring,
    ToggleSaveToFile(bool),
    LogToFile,
}

// Struct for serializing and deserializing system data
#[derive(Serialize, Deserialize)]
pub struct SystemData {
    pub timestamp: String,
    pub cpu_usage_percent: f32,
    pub memory_usage_byte: (u64, u64),
    pub swap_memory_usage_byte: (u64, u64),
    pub disk_usage_byte: (u64, u64),
    pub network_sent_byte: u64,
    pub network_received_byte: u64,
}

// Basic system info
#[derive(Debug, Clone)]
pub struct SystemBaseInfo {
    pub system_name: String,
    pub kernal_version: String,
    pub os_version: String,
    pub host_name: String,
}

// Info for each disk
#[derive(Debug, Clone)]
pub struct DisksInfo {
    pub name: String,
    pub kind: String,
    pub mount: String,
    pub total_disk: u64,
    pub free_disk: u64,
    pub used_disk_percent: f64,
}

// Info for each process
#[derive(Debug, Clone)]
pub struct Process {
    pub id: u32,
    pub name: String,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
}

// SystemMonitor struct holding all system information
#[derive(Debug)]
pub struct SystemMonitor {
    pub system: System,
    pub disks: Disks,
    pub networks: Networks,
    pub system_base_info: SystemBaseInfo,

    // CPU info
    pub cpu_usage: f32,
    pub no_of_processes: u32,
    pub physical_cores: u32,
    pub logical_processors: u32,
    pub processors_info: Vec<(String, f32)>,

    // Memory
    pub memory_usage: (u64, u64),
    pub swap_memory_usage: (u64, u64),

    // Disk
    pub disk_usage: (u64, u64),
    pub disks_info: Vec<DisksInfo>,

    // Network
    pub network_sent: u64,
    pub network_received: u64,

    // Processes
    pub processes: Vec<Process>,

    // Other
    pub is_monitoring: bool,
    pub save_to_file: bool,
    pub interval_in_secs: String,
}
