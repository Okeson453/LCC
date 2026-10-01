//! Shared integration-test harness.

use std::process::Command;

/// Helper: spin up the docker-compose stack for integration tests.
pub fn compose_up() {
    let _ = Command::new("docker")
        .args(["compose", "-f", "infra/docker/docker-compose.yml", "up", "-d"])
        .status();
}

/// Helper: tear down the docker-compose stack.
pub fn compose_down() {
    let _ = Command::new("docker")
        .args(["compose", "-f", "infra/docker/docker-compose.yml", "down"])
        .status();
}

/// Helper: wait for a service to be healthy (poll /healthz).
pub async fn wait_healthy(url: &str, timeout_secs: u64) -> bool {
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < timeout_secs {
        if let Ok(resp) = client.get(url).send().await {
            if resp.status().is_success() {
                return true;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    false
}
