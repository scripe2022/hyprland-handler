use std::sync::Arc;
use crate::Hyprsocket;
use tokio::process::Command;

pub async fn handle(_client: Arc<Hyprsocket>, data: String) {
    let (_, workspace_name) = data.split_once(',').unwrap();
    Command::new("eww")
        .args(&["update", format!("CURWS={}", workspace_name).as_str()])
        .spawn()
        .expect("Failed to start command");
}

