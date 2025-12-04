pub mod discovery;
pub mod signaling;
pub mod connection;
pub mod clipboard;

use tauri::{Manager, Emitter};
use crate::discovery::{DiscoveryService, Peer};
use crate::clipboard::ClipboardManager;

use crate::signaling::SignalingService;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use crate::connection::ConnectionManager;

#[tauri::command]
async fn connect_to_peer(state: tauri::State<'_, ConnectionManager>, ip: String, port: u16) -> Result<(), String> {
    println!("Connecting to peer at {}:{}", ip, port);
    // Test creating a PC
    let _pc = state.create_pc().await.map_err(|e| e.to_string())?;
    println!("Created PeerConnection");
    
    // TODO: Signaling exchange
    Ok(())
}

#[tauri::command]
async fn send_file(state: tauri::State<'_, ConnectionManager>, path: String) -> Result<(), String> {
    println!("Request to send file: {}", path);
    state.send_file(std::path::PathBuf::from(path)).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ConnectionManager::new())
        .setup(|app| {
            
            // Start clipboard manager
            let clipboard_manager = ClipboardManager::new().expect("Failed to init clipboard");
            clipboard_manager.start_listener();
            let mut clipboard_rx = clipboard_manager.subscribe();
            
            let connection_manager = app.state::<ConnectionManager>();
            let cm_clone = connection_manager.inner().clone();
            
            tauri::async_runtime::spawn(async move {
                while let Ok(text) = clipboard_rx.recv().await {
                    println!("Clipboard changed locally: {}", text);
                    cm_clone.send_clipboard_data(text).await;
                }
            });


            // Start signaling server asynchronously
            let handle_for_discovery = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let (port, mut signaling_rx) = SignalingService::start_server().await.expect("Failed to start signaling");
                
                let discovery = DiscoveryService::new();
                let name = format!("FluxBridge-{}", uuid::Uuid::new_v4());
                discovery.register(&name, port).expect("Failed to register");
                
                let mut rx = discovery.browse();
                
                // Spawn discovery listener
                let handle_clone = handle_for_discovery.clone();
                tauri::async_runtime::spawn(async move {
                    while let Some(info) = rx.recv().await {
                        let peer = Peer::from(info);
                        let _ = handle_clone.emit("peer-discovered", peer);
                    }
                });
                
                // Handle signaling messages
                while let Some((msg, addr)) = signaling_rx.recv().await {
                    println!("Received signaling from {}: {:?}", addr, msg);
                    // TODO: Handle offer/answer
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, connect_to_peer, send_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
