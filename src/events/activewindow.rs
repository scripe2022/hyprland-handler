use std::sync::Arc;
use crate::Hyprsocket;

pub async fn handle(client: Arc<Hyprsocket>, data: String) {
    let cur_vmware = data.ends_with(" - VMware Workstation");
    let last_vmware = client.get_vmware_active().await;
    if cur_vmware {
        if let Some((_, title)) = data.split_once(',') {
            let submap = "dispatch submap passthru";
            let control_escape = format!("dispatch sendshortcut CONTROL,CONTROL_L,title:{}", title);
            match client.sends_silent(&[&submap, &control_escape]).await {
                Ok(_) => {},
                Err(e) => eprintln!("Error handling activewindow: {}", e),
            }
        }
    }
    else if last_vmware {
        match client.sends_silent(&["dispatch submap reset"]).await {
            Ok(_) => {},
            Err(e) => eprintln!("Error handling activewindow: {}", e),
        }
    }
    client.set_vmware_active(cur_vmware).await;
}

