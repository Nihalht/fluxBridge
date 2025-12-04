use mdns_sd::{ServiceDaemon, ServiceInfo, ServiceEvent};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct DiscoveryService {
    mdns: ServiceDaemon,
    service_type: &'static str,
    peers: Arc<Mutex<HashMap<String, ServiceInfo>>>,
}

impl DiscoveryService {
    pub fn new() -> Self {
        let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
        Self {
            mdns,
            service_type: "_fluxbridge._tcp.local.",
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&self, instance_name: &str, port: u16) -> anyhow::Result<()> {
        let mut hostname = hostname::get()?.into_string().unwrap_or_else(|_| "fluxbridge".to_string());
        
        // Ensure hostname ends with .local.
        if !hostname.ends_with(".local.") {
            if hostname.ends_with(".local") {
                hostname.push('.');
            } else {
                hostname = format!("{}.local.", hostname);
            }
        }
        
        let properties = [("name", instance_name)];
        
        let my_service = ServiceInfo::new(
            self.service_type,
            instance_name,
            &hostname,
            "", // ip, let mdns-sd handle it
            port,
            &properties[..],
        )?.enable_addr_auto();

        self.mdns.register(my_service)?;
        Ok(())
    }

    pub fn browse(&self) -> mpsc::Receiver<ServiceInfo> {
        let (tx, rx) = mpsc::channel(32);
        let receiver = self.mdns.browse(self.service_type).expect("Failed to browse");
        
        let peers = self.peers.clone();
        
        // Spawn a thread or task to bridge the mdns receiver to our tokio channel
        // mdns-sd receiver is blocking, so we use a standard thread or spawn_blocking
        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        // Avoid holding the lock across await
                        {
                            let mut peers_guard = peers.lock().unwrap();
                            peers_guard.insert(info.get_fullname().to_string(), info.clone());
                        }
                        let _ = tx.blocking_send(info);
                    }
                    ServiceEvent::ServiceRemoved(_service_type, fullname) => {
                        let mut peers_guard = peers.lock().unwrap();
                        peers_guard.remove(&fullname);
                    }
                    _ => {}
                }
            }
        });
        
        rx
    }
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct Peer {
    pub name: String,
    pub hostname: String,
    pub ip: Vec<String>,
    pub port: u16,
}

impl From<ServiceInfo> for Peer {
    fn from(info: ServiceInfo) -> Self {
        Peer {
            name: info.get_fullname().to_string(),
            hostname: info.get_hostname().to_string(),
            ip: info.get_addresses().iter().map(|ip| ip.to_string()).collect(),
            port: info.get_port(),
        }
    }
}
