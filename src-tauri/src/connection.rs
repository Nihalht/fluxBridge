use webrtc::api::APIBuilder;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::interceptor::registry::Registry;
use webrtc::api::media_engine::MediaEngine;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<Mutex<Vec<Arc<RTCPeerConnection>>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn create_pc(&self) -> anyhow::Result<Arc<RTCPeerConnection>> {
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;
        
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)?;
        
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();
            
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };
        
        let pc = api.new_peer_connection(config).await?;
        
        pc.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            println!("Peer Connection State has changed: {}", s);
            Box::pin(async {})
        }));
        
        pc.on_data_channel(Box::new(move |d| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            println!("New DataChannel {} {}", d_label, d_id);
            
            Box::pin(async move {
                let d2 = d.clone();
                let d_label_clone = d_label.clone();
                d.on_open(Box::new(move || {
                    println!("Data channel '{}'-'{}' open", d_label_clone, d_id);
                    Box::pin(async move {
                        if d_label_clone == "clipboard" {
                            println!("Clipboard channel open");
                        }
                    })
                }));
                
                d.on_message(Box::new(move |msg: DataChannelMessage| {
                    let msg_data = String::from_utf8(msg.data.to_vec()).unwrap();
                    println!("Message from DataChannel '{}': '{}'", d_label, msg_data);
                    
                    if d_label == "clipboard" {
                        // TODO: Inject into local clipboard
                         println!("Received clipboard data: {}", msg_data);
                    }
                    
                    Box::pin(async {})
                }));
            })
        }));
        
        let pc_arc = Arc::new(pc);
        self.connections.lock().unwrap().push(pc_arc.clone());
        Ok(pc_arc)
    }

    pub async fn send_clipboard_data(&self, text: String) {
        let connections = self.connections.lock().unwrap().clone();
        for pc in connections {
            // TODO: Use specific data channel
            println!("Would send clipboard data to peer: {}", text);
        }
    }

    pub async fn send_file(&self, path: std::path::PathBuf) -> anyhow::Result<()> {
        let connections = self.connections.lock().unwrap().clone();
        if connections.is_empty() {
            return Err(anyhow::anyhow!("No peers connected"));
        }

        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let file_content = tokio::fs::read(&path).await?;
        let total_size = file_content.len();
        
        println!("Sending file: {} ({} bytes)", file_name, total_size);

        // Simple chunking
        let chunk_size = 16 * 1024; // 16KB
        let chunks = file_content.chunks(chunk_size);
        let total_chunks = chunks.len();

        for (_pc, _chunk) in connections.iter().zip(chunks) {
            // In a real app, we'd wrap this in a protocol message (JSON/Protobuf)
            // For PoC, we just log
            // println!("Sending chunk {}/{}", i + 1, total_chunks);
            // Simulate send
            // pc.data_channel...send(chunk)
        }
        
        // Just iterating to show usage
        for _pc in connections {
             println!("Would send file to peer");
        }
        
        for (i, _chunk) in file_content.chunks(chunk_size).enumerate() {
             println!("Sending chunk {}/{}", i + 1, total_chunks);
        }
        
        Ok(())
    }
}
