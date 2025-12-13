# FluxBridge - Cross-Platform Continuity Engine

A Rust + Tauri application for seamless device connectivity featuring P2P discovery, universal clipboard sync, and file transfer.

## 🎯 What It Does:
FluxBridge creates a local mesh network between your devices, enabling instant clipboard synchronization and file transfers without any configuration. Copy text on your laptop, paste it on your desktop. Drag a file into the app, and it appears on your other device—all happening locally, encrypted, and in real-time.

## Features

- **ZZ8🔍 P2P Discovery**: Zero-config device discovery via mDNS
- **📋 Universal Clipboard**: Automatic clipboard synchronization across devices
- **📁 File Transfer**: Drag-and-drop file sharing

## Quick Start

### Prerequisites
- Node.js (v20.19+ or v22.12+)
- Rust (latest stable)
- npm

### Running the Application

```bash
cd fluxbridge
npm install
npm run tauri dev
```

### How to Use

1. **Launch the App**: Run `npm run tauri dev` - a window will open showing "FluxBridge"

2. **Discover Peers**: 
   - The app automatically scans for other FluxBridge instances on your network
   - Discovered peers appear in the "Discovered Peers" list

3. **Connect to a Peer**:
   - Click the "Connect" button next to a peer's name
   - Check the terminal for "Created PeerConnection" confirmation

4. **Test Clipboard Sync**:
   - Copy text on one device
   - Check the terminal logs to see clipboard changes detected
   - The text will be synced to connected peers

5. **Transfer Files**:
   - Drag and drop a file into the "Drag & Drop" zone
   - The file will be chunked and sent to connected peers

## Terminal Output

The terminal shows important events:
- `Clipboard changed locally: [text]` - Local clipboard detected
- `Received clipboard data: [text]` - Remote clipboard received
- `Created PeerConnection` - WebRTC connection established
- `Peer discovered: [name]` - New peer found on network

## Troubleshooting

- **No peers found**: Ensure devices are on the same Wi-Fi network
- **Port conflicts**: Kill processes on port 1420: `lsof -ti:1420 | xargs kill -9`
- **Build errors**: Update Rust: `rustup update`

## Architecture

- **Backend**: Rust (mDNS, WebRTC, clipboard monitoring)
- **Frontend**: React + TypeScript + Vite
- **Framework**: Tauri v2
- **Networking**: WebRTC Data Channels for P2P communication

## Project Structure

```
fluxbridge/
├── src/                    # React frontend
│   ├── App.tsx            # Main UI component
│   └── App.css            # Styles
├── src-tauri/             # Rust backend
│   └── src/
│       ├── lib.rs         # Main application logic
│       ├── discovery.rs   # mDNS service
│       ├── signaling.rs   # WebRTC signaling
│       ├── connection.rs  # WebRTC connections
│       └── clipboard.rs   # Clipboard manager
└── README.md
```
⚡ **Key Features:**
- Automatic device discovery using mDNS (like Apple's Bonjour)
- Real-time clipboard synchronization across all connected devices
- **Drag-and-drop** file transfer with chunking for large files
- **Direct peer-to-peer** connections via WebRTC
- **Cross-platform support** (Windows, macOS, Linux)
- Zero configuration required
- Works completely **offline** (only needs local Wi-Fi)

## Development

- **Dev Mode**: `npm run tauri dev`
- **Build**: `npm run tauri build`
- **Check Rust**: `cd src-tauri && cargo check`

