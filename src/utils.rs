use crate::models::{SystemData, SystemMonitor};
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;
use sysinfo::Disks;

pub fn convert_from_bytes(bytes: u64, value: i32) -> f64 {
    bytes as f64 / f64::powf(1024., value as f64)
}

pub fn calculate_disk_usage(disks: &Disks) -> (u64, u64) {
    let total_disk = disks.iter().fold(0, |acc, disk| acc + disk.total_space());
    let used_disk = disks.iter().fold(0, |acc, disk| {
        acc + (disk.total_space() - disk.available_space())
    });
    (used_disk, total_disk)
}

pub fn log_metrics(system_monitor: &SystemMonitor) {
    let data = SystemData {
        timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        cpu_usage_percent: system_monitor.cpu_usage,
        memory_usage_byte: system_monitor.memory_usage,
        swap_memory_usage_byte: system_monitor.swap_memory_usage,
        disk_usage_byte: system_monitor.disk_usage,
        network_sent_byte: system_monitor.network_sent,
        network_received_byte: system_monitor.network_received,
    };

    let serialized = serde_json::to_string(&data).expect("Failed to serialize system data");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("system_log.json")
        .expect("Failed to open log file");

    writeln!(file, "{}", serialized).expect("Failed to write data to log file");
}
