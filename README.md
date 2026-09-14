# Zentra - Screen Studio for Windows

**Zentra** is a high-performance, offline-only Windows desktop application for nonstop screen recording with cinematic zoom-and-pan export — bringing the polished Screen Studio experience to Windows.

---

## Features

- **100% Offline & Private**: Zero telemetry and zero cloud uploads. Video encoding, database storage, and rendering happen strictly on your local machine.
- **Hardware-Accelerated 60 FPS Capture**: Built on `Windows.Graphics.Capture` (WGC) and native Windows Media Foundation H.264 hardware encoding with zero-copy DXGI surface buffers.
- **Live Screen Zoom & Auto-Pan**: Real-time 1.8x monitor zoom via the Windows Magnification API with smooth 60 FPS cursor tracking and fluid screen dragging.
- **Cinematic Export with Zoom**: Faithfully reproduces continuous live screen pans and zooms using smooth cubic easing transitions ($3t^2 - 2t^3$) at constant 60 FPS.
- **Disappearing UI & Draggable Widget**: The main app hides automatically upon recording. A floating pill timer appears with pre-recording countdowns (3s, 5s, 10s) and is excluded from the captured video via `WDA_EXCLUDEFROMCAPTURE`.
- **Crash-Resilient Rolling Segments**: Automatically partitions long recordings into rolling 5-minute MP4 chunks so no footage is lost if your PC unexpectedly shuts down.
- **Multi-Monitor & Storage Management**: Record any connected display, configure custom storage directories, and manage past recordings in a unified dashboard.
- **In-App Background Auto-Updater**: Cryptographically verified background update checks and passive installations via `@tauri-apps/plugin-updater`.

---

## Keyboard Shortcuts & Controls

| Shortcut / Action | Function |
| :--- | :--- |
| <kbd>`</kbd> *(Backtick)* | Zoom in (1.8x) centered at cursor location |
| <kbd>Esc</kbd> | Zoom out / reset to 1.0x view |
| <kbd>F9</kbd> | Stop recording and restore the main window |
| **Mouse Drag** | Fluidly pans the zoomed viewport in real-time |

---

## Quick Start

### Prerequisites

1. **Windows 10 / 11** (64-bit)
2. **Rust Toolchain**: `stable-x86_64-pc-windows-msvc` or `stable-x86_64-pc-windows-gnu`
3. **Node.js**: v18+
4. **C/C++ Build Tools**: MSVC C++ Build Tools or MinGW-w64 (WinLibs / MSYS2)

### Development

```powershell
# Install frontend dependencies
npm install

# Launch in Tauri development mode
npm run dev
```

### Production Build

```powershell
# Build NSIS (.exe) and WiX (.msi) installers
npm run build
```

Generated installer packages are located in `src-tauri/target/release/bundle/nsis/` and `src-tauri/target/release/bundle/msi/`.

---

## Project Structure

```text
Zentra/
├── docs/                          # Architecture & technical deep-dives
│   └── architecture.md            # Hardware QPC clock sync, Direct2D pipeline, trade-offs
├── src-tauri/                     # Rust backend (Tauri v2)
│   ├── Cargo.toml                 # Dependencies (tauri 2, rusqlite, windows-rs)
│   ├── tauri.conf.json            # Desktop window configuration & permissions
│   └── src/
│       ├── capture/               # WGC capture pipeline & D3D11 frame pool
│       ├── encoder/               # Media Foundation H.264 rolling sink writer
│       ├── export/                # Direct2D compositor & video export pipeline
│       ├── live_magnifier/        # Windows Magnification API live zoom & pan
│       ├── mouse_hook/            # Low-level mouse hook & QPC telemetry
│       └── storage/               # SQLite database manager
├── ui/                            # Frontend desktop interface
│   ├── index.html                 # Main dashboard layout
│   ├── widget.html                # Floating overlay widget layout
│   ├── js/                        # Modular controllers (recording, export, sessions)
│   └── css/                       # Modular feature stylesheets
├── package.json                   # NPM package scripts & Tauri dependencies
└── README.md
```

---

## Architecture & Technical Details

For in-depth explanations of the system pipelines, hardware `QueryPerformanceCounter` (QPC) clock reconciliation formulas, and the Media Foundation vs. FFmpeg trade-off analysis, see the [Architecture Documentation](docs/architecture.md).

---

## License

Internal proprietary software developed for high-performance offline Windows screen recording.