mod hyprsocket;
mod events;

use hyprsocket::Hyprsocket;
use tokio::task;
use std::error::Error;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Hyprsocket::new().await?;
    let client = Arc::new(client);

    let client_clone = Arc::clone(&client);
    task::spawn(async move {
        if let Err(e) = client_clone.listen_for_events(move |event, data| {
            let client = Arc::clone(&client);
            tokio::spawn(async move {
                match event.as_str() {
                    "activewindow" => events::activewindow::handle(client, data).await,
                    "workspacev2" => events::workspacev2::handle(client, data).await,
                    // _ => println!("Unknown event: {}", event),
                    _ => {},
                }
            });
        })
        .await {
            eprintln!("Error listening for events: {}", e);
        }
    }).await?;

    Ok(())
}

