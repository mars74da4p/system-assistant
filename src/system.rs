use std::process::Command;

use sysinfo::{Disks, System};

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
pub struct ServiceInfo {
    pub name: String,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct SystemSnapshot {
    pub cpu_percent: f32,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub memory_free_mb: u64,
    pub uptime_human: String,
    pub processes: Vec<ProcessInfo>,
    pub services: Vec<ServiceInfo>,
    pub disks: Vec<DiskInfo>,
    pub hostname: String,
}

pub fn snapshot() -> SystemSnapshot {
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let free_memory = sys.free_memory();

    let mut processes: Vec<ProcessInfo> = sys
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

    processes.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));
    processes.truncate(25);

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

    let services = load_services();

    SystemSnapshot {
        cpu_percent: sys.global_cpu_info().cpu_usage(),
        memory_total_mb: total_memory / 1024 / 1024,
        memory_used_mb: used_memory / 1024 / 1024,
        memory_free_mb: free_memory / 1024 / 1024,
        uptime_human: format_duration(sys.uptime()),
        processes,
        services,
        disks,
        hostname: hostname(),
    }
}

pub fn kill_process(pid: usize) -> anyhow::Result<String> {
    let output = Command::new("kill").args(["-15", &pid.to_string()]).output()?;

    if output.status.success() {
        Ok(format!("SIGTERM sent to PID {}", pid))
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("kill failed: {}", err.trim()))
    }
}

pub fn set_process_priority(pid: usize, niceness: i32) -> anyhow::Result<String> {
    let output = Command::new("renice")
        .args([niceness.to_string(), pid.to_string()])
        .output()?;

    if output.status.success() {
        Ok(format!("Priority updated for PID {} to {}", pid, niceness))
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("renice failed: {}", err.trim()))
    }
}

pub fn execute_service_action(service: &str, action: &str) -> anyhow::Result<String> {
    let output = Command::new("systemctl")
        .args([action, service])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = stdout.trim();
        if !msg.is_empty() {
            Ok(format!("{} {}: {}", action, service, msg.lines().next().unwrap_or("OK")))
        } else {
            Ok(format!("{} {} completed successfully", action, service))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!(
            "systemctl {} {} failed: {}",
            action,
            service,
            stderr.trim()
        ))
    }
}

fn load_services() -> Vec<ServiceInfo> {
    let output = match Command::new("systemctl")
        .args(["list-units", "--type=service", "--all", "--no-pager", "--no-legend", "--plain"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return vec![ServiceInfo {
            name: "systemctl-unavailable".to_string(),
            status: "not available".to_string(),
        }],
    };

    if !output.status.success() {
        return vec![ServiceInfo {
            name: "systemctl-unavailable".to_string(),
            status: "error".to_string(),
        }];
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(12)
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let status = format!("{} {}", parts[2], parts[3]);
                ServiceInfo { name, status }
            } else {
                ServiceInfo {
                    name: "unknown".to_string(),
                    status: "status".to_string(),
                }
            }
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_builds() {
        let snap = snapshot();
        assert!(!snap.hostname.is_empty());
        assert!(snap.processes.len() > 0);
    }
}
