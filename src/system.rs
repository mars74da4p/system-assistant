use std::collections::HashMap;

use sysinfo::{Disks, Process, System};

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: usize,
    pub name: String,
    pub cpu: f32,
    pub memory_mb: u64,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub total_mb: u64,
    pub used_mb: u64,
}

#[derive(Clone, Debug)]
pub struct SystemSnapshot {
    pub cpu_percent: f32,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub memory_free_mb: u64,
    pub uptime_human: String,
    pub processes: Vec<ProcessInfo>,
    pub disks: Vec<DiskInfo>,
    pub hostname: String,
}

pub fn snapshot() -> SystemSnapshot {
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let free_memory = sys.free_memory();

    let processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32() as usize,
            name: process.name().to_string_lossy().to_string(),
            cpu: process.cpu_usage(),
            memory_mb: process.memory() / 1024 / 1024,
            status: match process.status() {
                sysinfo::ProcessStatus::Run => "RUN".to_string(),
                sysinfo::ProcessStatus::Sleep => "SLEEP".to_string(),
                sysinfo::ProcessStatus::Idle => "IDLE".to_string(),
                sysinfo::ProcessStatus::Stop => "STOP".to_string(),
                sysinfo::ProcessStatus::Zombie => "ZOMBIE".to_string(),
                sysinfo::ProcessStatus::Dead => "DEAD".to_string(),
                _ => "OTHER".to_string(),
            },
        })
        .collect();

    let mut sorted = processes;
    sorted.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));
    sorted.truncate(12);

    let mut disks = Vec::new();
    let mut disk_list = Disks::new_with_refreshed_list();
    disk_list.refresh();
    for disk in &disk_list {
        let total = disk.total_space() / 1024 / 1024;
        let used = (disk.total_space() - disk.available_space()) / 1024 / 1024;
        disks.push(DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            mount: disk.mount_point().to_string_lossy().to_string(),
            total_mb: total,
            used_mb: used,
        });
    }

    let uptime_seconds = sys.uptime();
    let uptime_human = format_duration(uptime_seconds);

    SystemSnapshot {
        cpu_percent: sys.global_cpu_info().cpu_usage(),
        memory_total_mb: total_memory / 1024 / 1024,
        memory_used_mb: used_memory / 1024 / 1024,
        memory_free_mb: free_memory / 1024 / 1024,
        uptime_human,
        processes: sorted,
        disks,
        hostname: hostname(),
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| "linux-host".to_string())
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
