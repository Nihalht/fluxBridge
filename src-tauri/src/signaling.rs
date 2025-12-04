use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum SignalingMessage {
    Offer(String),
    Answer(String),
    Candidate(String),
}

pub struct SignalingService;

impl SignalingService {
    pub fn new() -> Self {
        Self
    }

    pub async fn start_server() -> anyhow::Result<(u16, mpsc::Receiver<(SignalingMessage, SocketAddr)>)> {
        let listener = TcpListener::bind("0.0.0.0:0").await?;
        let local_addr = listener.local_addr()?;
        let port = local_addr.port();
        
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let mut framed = Framed::new(stream, LinesCodec::new());
                    while let Some(Ok(line)) = framed.next().await {
                        if let Ok(msg) = serde_json::from_str::<SignalingMessage>(&line) {
                            let _ = tx.send((msg, addr)).await;
                        }
                    }
                });
            }
        });

        Ok((port, rx))
    }
    
    pub async fn connect(ip: &str, port: u16) -> anyhow::Result<mpsc::Sender<SignalingMessage>> {
        let addr = format!("{}:{}", ip, port);
        let stream = TcpStream::connect(addr).await?;
        let (mut sink, _) = Framed::new(stream, LinesCodec::new()).split();
        let (tx, mut rx) = mpsc::channel::<SignalingMessage>(32);
        
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(line) = serde_json::to_string(&msg) {
                    let _ = sink.send(line).await;
                }
            }
        });
        
        Ok(tx)
    }
}
