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
