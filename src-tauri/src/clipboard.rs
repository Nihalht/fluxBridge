use arboard::Clipboard;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

pub struct ClipboardManager {
    clipboard: Arc<Mutex<Clipboard>>,
    tx: broadcast::Sender<String>,
}

impl ClipboardManager {
    pub fn new() -> anyhow::Result<Self> {
        let clipboard = Clipboard::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let (tx, _) = broadcast::channel(16);
        
        Ok(Self {
            clipboard: Arc::new(Mutex::new(clipboard)),
            tx,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    pub fn set_text(&self, text: String) -> anyhow::Result<()> {
        let mut clipboard = self.clipboard.lock().unwrap();
        clipboard.set_text(text).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(())
    }

    pub fn start_listener(&self) {
        let clipboard = self.clipboard.clone();
        let tx = self.tx.clone();
        
        std::thread::spawn(move || {
            let mut last_text = String::new();
            
            // Initial read
            if let Ok(mut cb) = clipboard.lock() {
                if let Ok(text) = cb.get_text() {
                    last_text = text;
                }
            }

            loop {
                std::thread::sleep(Duration::from_millis(500));
                
                let current_text = {
                    let mut cb = match clipboard.lock() {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    cb.get_text().unwrap_or_default()
                };

                if current_text != last_text && !current_text.is_empty() {
                    println!("Clipboard changed: {}", current_text);
                    last_text = current_text.clone();
                    // Broadcast change
                    let _ = tx.send(last_text.clone());
                }
            }
        });
    }
}
