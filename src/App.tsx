import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface Peer {
  name: string;
  ip: string[];
  port: number;
}

function App() {
  const [peers, setPeers] = useState<Peer[]>([]);

  useEffect(() => {
    const unlisten = listen<Peer>("peer-discovered", (event) => {
      console.log("Peer discovered:", event.payload);
      setPeers((prev) => {
        // Avoid duplicates
        if (prev.find(p => p.name === event.payload.name)) return prev;
        return [...prev, event.payload];
      });
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <main className="container">
      <h1>FluxBridge</h1>
      <div className="row">
        <h2>Discovered Peers</h2>
      </div>
      <div className="peers-list">
        {peers.length === 0 ? <p>Scanning for peers...</p> : null}
        {peers.map((peer, i) => (
          <div key={i} className="peer-item">
            <span>{peer.name} ({peer.ip[0]})</span>
            <button onClick={() => invoke("connect_to_peer", { ip: peer.ip[0], port: peer.port })}>
              Connect
            </button>
          </div>
        ))}
      </div>

      <div
        className="drop-zone"
        onDragOver={(e) => e.preventDefault()}
        onDrop={(e) => {
          e.preventDefault();
          const files = Array.from(e.dataTransfer.files);
          if (files.length > 0) {
            // In a real web app, we can't get full path easily due to security.
            // But in Tauri, we can listen to the window file drop event or use the dialog API.
            // For this PoC, we'll assume the user drops a file and we get the path via Tauri's file-drop event.
            console.log("Dropped files (web API):", files);
          }
        }}
      >
        <p>Drag & Drop files here to send to connected peers</p>
      </div>
    </main>
  );
}

// We need to listen to Tauri's global file drop event for paths
listen("tauri://drag-drop", (event: any) => {
  console.log("File dropped:", event.payload);
  if (event.payload.paths && event.payload.paths.length > 0) {
    const path = event.payload.paths[0];
    invoke("send_file", { path });
  }
});

export default App;
