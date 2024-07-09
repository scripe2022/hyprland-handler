use tokio::net::UnixStream;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::task;
use tokio::sync::{mpsc, Mutex};
use std::error::Error;
use std::env;
use std::sync::Arc;

pub struct Hyprsocket {
    command_socket_path: String,
    event_socket_path: String,
    vmware_active: Arc<Mutex<bool>>,
}

impl Hyprsocket {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR")?;
        let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")?;

        let command_socket_path = format!("{}/hypr/{}/.socket.sock", xdg_runtime_dir, hyprland_instance_signature);
        let event_socket_path = format!("{}/hypr/{}/.socket2.sock", xdg_runtime_dir, hyprland_instance_signature);

        Ok(Self {
            command_socket_path,
            event_socket_path,
            vmware_active: Arc::new(Mutex::new(false)),
        })
    }

    pub async fn listen_for_events(&self, handler: impl Fn(String, String) + Send + Sync + 'static) -> Result<(), Box<dyn Error>> {
        let (tx, mut rx) = mpsc::channel::<(String, String)>(100);
        let event_socket_path = self.event_socket_path.clone();

        task::spawn(async move {
            let stream = UnixStream::connect(event_socket_path).await.unwrap();
            let mut reader = BufReader::new(stream);

            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        if let Some((event, data)) = parse_event(&line) {
                            if tx.send((event, data)).await.is_err() {
                                break;
                            }
                        }
                    },
                    Err(_) => break,
                }
            }
        });

        while let Some((event, data)) = rx.recv().await {
            handler(event, data);
        }

        Ok(())
    }

    // pub async fn send(&self, command: &str) -> Result<String, Box<dyn Error>> {
    //     let mut stream = UnixStream::connect(&self.command_socket_path).await?;
    //     stream.write_all(command.as_bytes()).await?;
    //
    //     let mut reader = BufReader::new(stream);
    //     let mut response = String::new();
    //     reader.read_to_string(&mut response).await?;
    //
    //     Ok(response)
    // }

    pub async fn sends_silent(&self, commands: &[&str]) -> Result<(), Box<dyn Error>> {
        let mut stream = UnixStream::connect(&self.command_socket_path).await?;
        let command = if commands.len() > 1 {
            format!("[[BATCH]]{}", commands.join(";"))
        } else {
            commands.get(0).map_or(String::new(), |&cmd| cmd.to_string())
        };

        if !command.is_empty() {
            stream.write_all(command.as_bytes()).await?;
        }

        Ok(())
    }

    pub async fn get_vmware_active(&self) -> bool {
        let vmware_active = self.vmware_active.lock().await;
        *vmware_active
    }

    pub async fn set_vmware_active(&self, value: bool) {
        let mut vmware_active = self.vmware_active.lock().await;
        *vmware_active = value;
    }
}

fn parse_event(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.trim().split(">>").collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    }
    else {
        None
    }
}

