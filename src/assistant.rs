use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::system::SystemSnapshot;

#[derive(Debug, Default)]
pub struct Assistant {
    pub history: Vec<String>,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl Assistant {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn answer_local(&self, user_input: &str, snapshot: &SystemSnapshot) -> String {
        let input = user_input.trim();
        if input.is_empty() {
            return "Введите команду или вопрос для ассистента.".to_string();
        }

        let lowered = input.to_lowercase();

        if lowered.contains("cpu") || lowered.contains("load") {
            return format!(
                "Сейчас загрузка CPU: {:.1}%. Если она высокая, проверьте top, htop, и сервисы, которые потребляют много CPU.",
                snapshot.cpu_percent
            );
        }

        if lowered.contains("memory") || lowered.contains("ram") {
            return format!(
                "Используется RAM: {} MB из {} MB. Свободно: {} MB.",
                snapshot.memory_used_mb,
                snapshot.memory_total_mb,
                snapshot.memory_free_mb
            );
        }

        if lowered.contains("disk") || lowered.contains("storage") {
            let disk_summary = snapshot
                .disks
                .iter()
                .take(3)
                .map(|d| format!("{} {}MB/{}", d.name, d.used_mb, d.total_mb))
                .collect::<Vec<_>>()
                .join(" | ");
            return format!("Дисковая нагрузка: {}", disk_summary);
        }

        if lowered.contains("restart") || lowered.contains("start") || lowered.contains("stop") {
            return "Безопасное действие: сначала проверьте status сервиса, затем выполните restart/stop/start с подтверждением.".to_string();
        }

        if lowered.contains("top") || lowered.contains("process") {
            let top = snapshot
                .processes
                .iter()
                .take(5)
                .map(|p| format!("{}:{}MB", p.name, p.memory_mb))
                .collect::<Vec<_>>()
                .join(", ");
            return format!("Топ процессов по памяти: {}", top);
        }

        if lowered.contains("log") || lowered.contains("error") {
            return "Проверьте journalctl -xe, systemctl status, и tail -f /var/log/syslog. Если нужно — я могу помочь проанализировать логи вручную.".to_string();
        }

        if lowered.contains("help") {
            return "Команды: cpu, memory, disk, restart, top, process, log, status".to_string();
        }

        if lowered.contains("status") {
            return format!(
                "Система работает {}. CPU: {:.1}%, RAM: {}MB/{}MB.",
                snapshot.uptime_human,
                snapshot.cpu_percent,
                snapshot.memory_used_mb,
                snapshot.memory_total_mb
            );
        }

        format!("Запрос: \"{}\". Я могу помочь с диагностикой CPU, RAM, дисков, процессов и сервисов Linux.", user_input)
    }

    pub async fn ask_ollama(&self, prompt: &str) -> Result<String> {
        let client = Client::new();
        let payload = OllamaRequest {
            model: "llama3.2".to_string(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let resp = client
            .post("http://127.0.0.1:11434/api/generate")
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("Ollama endpoint is unavailable"));
        }

        let data: OllamaResponse = resp.json().await?;
        Ok(data.response.trim().to_string())
    }
}
