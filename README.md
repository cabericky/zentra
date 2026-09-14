# Zentra - Screen Studio for Windows

**Zentra** is a high-performance, offline-only Windows desktop application for nonstop screen recording with a post-recording "zoom on click" export feature — bringing the polished, cinematic zoom-and-pan experience of Screen Studio to Windows.

---

## Key Features

- **100% Offline & Private**: Zero cloud uploads, zero telemetry. All video encoding, database operations, and export rendering happen strictly locally on your machine.
- **Native Hardware Acceleration**:
  - **Capture**: `Windows.Graphics.Capture` (WGC WinRT API) with `Direct3D11CaptureFramePool::CreateFreeThreaded` for smooth 60 FPS primary monitor recording.
  - **Encoding**: Native Windows Media Foundation H.264 hardware encoder (`IMFSinkWriter`) with zero-copy DXGI surface buffers.
  - **Compositing**: Direct2D 1.1 / Direct3D 11 affine zoom and pan transformations with high-quality linear/bicubic interpolation.
- **Crash-Resilient Rolling Segments**: Video is automatically partitioned into rolling 5-minute chunks during long nonstop recording sessions. If your PC loses power or crashes, all completed segments remain fully playable and intact.
- **Unified High-Precision Clock (QPC)**: Frame presentation timestamps and mouse click events are captured against Windows `QueryPerformanceCounter` (QPC), guaranteeing sub-millisecond click-to-frame reconciliation without drift.
- **Disappearing App & Floating Draggable Widget**:
  - **App Disappearance**: When recording begins, the main desktop application completely disappears from view so your screen is clear.
  - **Draggable Overlay Widget**: A sleek frosted pill widget floats on top of your screen, displaying a pulsing red dot, a live elapsed timer (`00:00:00`), and a red **Stop** button. You can drag the widget anywhere on your monitor.
  - **Hardware Screen-Capture Exclusion (`WDA_EXCLUDEFROMCAPTURE`)**: Using Win32 `SetWindowDisplayAffinity`, the widget is rendered only to your physical monitor; Windows DWM completely omits the widget from screen capture, ensuring it **never appears in the recorded or exported video**.
  - **Pre-Recording Countdown Timers (3s, 5s, 10s, Off)**: Configure countdown delays directly on the floating widget or from the main dashboard before recording starts. Features amber pulsing visual indicators, live second-by-second countdown, and subtle audible synthetic chimes (Web Audio API) for preparation in the background. Abort or cancel anytime by clicking **Cancel** or pressing <kbd>Esc</kbd>.
  - **Stop Controls**: Stop recording anytime by clicking the widget Stop button or pressing <kbd>F9</kbd> anywhere. Zentra hides the widget, restores the main application, and focuses the window.
- **Live Screen Zoom & Auto-Pan**:
  - **Live Monitor Zoom**: Smoothly zooms in to **1.8x** on the physical monitor using Windows Magnification API (`MagSetFullscreenTransform`).
  - **Zero-Delay Screen Dragging**: While zoomed in, moving your mouse freely and smoothly glides/drags the screen view along at 60 FPS in 1:1 real-time lockstep.
  - **Keyboard Controls**: Press <kbd>`</kbd> (Backtick) to zoom in on cursor, and press <kbd>Esc</kbd> to zoom out / reset to 1.0x.
  - **Continuous Trajectory Recording**: The 60 FPS trajectory is continuously sampled into SQLite (`pan_samples`).
- **Smooth Post-Recording Zoom & Pan Reproduction**:
  - **Faithfully reproduces continuous screen dragging**: Export interpolates the exact camera position frame-for-frame, following the exact path moved during live dragging.
  - **CFR 60 FPS Real-Time Playback (1.0x Speed)**: Constant frame rate time-paced presentation loop preserves exact real-time duration without fast-forwarding static desktop pauses.
  - Generates smooth cubic easing transitions ($3t^2 - 2t^3$) with intelligent viewport clamping (preventing off-screen black borders).
- **Production Packaging & Installers**:
  - Standard Windows installation packages: **NSIS installer (`.exe`)** and **WiX installer (`.msi`)**.
  - Creates dedicated Windows Start Menu shortcuts under the **Zentra** program group.
  - Clean uninstaller integration via Windows Settings / Control Panel.
  - User-level per-machine or per-user installation without mandatory UAC elevation prompts.
- **Seamless In-App Background Auto-Updater**:
  - Powered by `@tauri-apps/plugin-updater` and cryptographic minisign signature verification.
  - Silent background checks upon application launch.
  - Manual "Check for Updates" trigger in top navigation bar.
  - Live in-app streaming download progress metrics with passive installation execution.
  - Recording safety guard that prevents application restarts while a recording session is active.
- **Unobtrusive Normal Clicks**: Standard left/right clicks operate normally without interfering with zoom triggers.

---

## Architectural Architecture

```mermaid
flowchart TD
    subgraph Capture Pipeline
        WGC["Windows.Graphics.Capture (WinRT)"] -->|ID3D11Texture2D| FrameQueue["Bounded DXGI Frame Queue (60 frames)"]
        FrameQueue -->|DXGI Surface Buffer| MFEncoder["Media Foundation H.264 Encoder"]
        MFEncoder -->|Rolling 5-Min Chunks| MP4Segments["Raw Video Segments (.mp4)"]
    end

    subgraph Mouse Telemetry
        LLHook["WH_MOUSE_LL Low-Level Hook Thread"] -->|QPC Ticks, X, Y| SQLiteChannel["Crossbeam Batch Channel"]
        SQLiteChannel -->|Batch Writes| SQLiteDB[("SQLite Database (zentra.db)")]
    end

    subgraph QPC Clock Sync
        QPC["QueryPerformanceCounter"] -.->|Presentation Timestamp| WGC
        QPC -.->|Click Timestamp| LLHook
    end

    subgraph Post-Recording Export Pipeline
        MP4Segments -->|IMFSourceReader| VideoReader["Hardware Frame Decoder"]
        SQLiteDB -->|Click Coordinates & Times| KeyframeGen["Cubic Keyframe Timeline Engine"]
        VideoReader --> Compositor["Direct2D Affine Zoom/Pan Compositor"]
        KeyframeGen -->|Transform Matrix (M11, M22, M31, M32)| Compositor
        Compositor --> ExportSink["Media Foundation Sink Writer"]
        ExportSink --> FinalMP4["Exported Video with Zoom (.mp4)"]
    end

    subgraph Production & Updates
        TauriUpdater["@tauri-apps/plugin-updater"] -->|Check & Download| GitHubReleases["GitHub Releases / Static CDN"]
        GitHubReleases -->|Passive Install| NSISInstaller["NSIS / MSI Installer"]
    end
```

---

## Clock Synchronization & Timestamp Reconciliation

A common pitfall in screen recorders is audio/video/mouse drift caused by using wall-clock time (`SystemTime`) or tick counts with variable granularity.

Zentra avoids this entirely using the hardware `QueryPerformanceCounter`:
1. When recording starts, `QpcClock::now()` captures the baseline tick count $T_0$ and queries `QueryPerformanceFrequency()` ($F$, ticks per second).
2. For every captured screen frame, the presentation timestamp is calculated as:
   $$\Delta t = \frac{T_{\text{frame}} - T_0}{F} \times 10{,}000{,}000 \quad (\text{in 100ns Media Foundation units})$$
3. On an independent OS thread with its own Win32 message pump (`GetMessageW`), the low-level mouse hook (`WH_MOUSE_LL`) queries `QpcClock::now()` immediately on `WM_LBUTTONDOWN`, `WM_RBUTTONDOWN`, and `WM_MBUTTONDOWN`.
4. During export, each frame's exact timestamp $t$ (in seconds from session start) is reconciled against the SQLite click timeline. If $t$ falls within a click's zoom window:
   $$t_{\text{rel}} = \frac{t - t_{\text{start}}}{D_{\text{ease}}}$$
   $$k = 3(t_{\text{rel}})^2 - 2(t_{\text{rel}})^3 \quad (\text{cubic ease-in-out})$$
   $$\text{scale} = 1.0 + k \times (\text{target\_scale} - 1.0)$$
   $$\text{viewport} = \text{clamp}\left(\text{center} - \frac{\text{dims}}{2 \times \text{scale}}, 0, \text{bounds}\right)$$

---

## Technical Trade-off: Media Foundation vs. FFmpeg

| Evaluation Factor | Native Windows Media Foundation (Chosen) | Bundled FFmpeg CLI / libav |
| :--- | :--- | :--- |
| **Zero-Copy DXGI Pipeline** | **Direct**: D3D11 textures from WGC pass directly to encoder via `MFCreateDXGISurfaceBuffer`. GPU VRAM stays on GPU. | **Indirect**: Requires copying D3D11 textures to system RAM or configuring complex D3D11VA/NVENC hardware filters. |
| **Distribution Size** | **Zero bytes added**: Pre-installed on every Windows 10/11 system. Final app is lean (~15 MB). | Adds **80 MB – 150 MB** of static binaries or dynamic libraries. |
| **Licensing** | Native Windows OS SDK. No GPL/LGPL complications. | LGPL/GPL licensing compliance and redistribution requirements. |
| **Compositing Integration** | Native Direct2D `ID2D1DeviceContext` interop with D3D11 render targets. | Requires pixel format conversions or piping raw frames over pipes/sockets. |
| **Portability** | Windows-only (aligns strictly with project specification). | Cross-platform (Linux/macOS/Windows). |
| **API Ergonomics** | Verbose COM interfaces requiring explicit attribute GUIDs and reference management. | High-level CLI or unified C API. |

---

## Building & Packaging

### Prerequisites
1. **Windows 10 / 11** (64-bit)
2. **Rust Toolchain**: `stable-x86_64-pc-windows-gnu` or `stable-x86_64-pc-windows-msvc`
   ```powershell
   rustup default stable-x86_64-pc-windows-gnu
   ```
3. **C/C++ Compiler & Binutils**: MinGW-w64 (`gcc.exe`, `dlltool.exe`, `as.exe`, `windres.exe`).
   Installed automatically via WinLibs or MSYS2:
   ```powershell
   winget install BrechtSanders.WinLibs.POSIX.UCRT
   ```
4. **WebView2 Runtime**: Pre-installed on Windows 10/11.
5. **Node.js**: v18+ (for frontend package coordination and Tauri CLI).

### Build Steps

1. **Configure Environment PATH** (if using MinGW/WinLibs):
   ```powershell
   $env:PATH = "C:\Users\AVITA\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin;$env:USERPROFILE\.cargo\bin;$env:PATH"
   ```

2. **Check Rust Compilation**:
   ```powershell
   cargo check --manifest-path src-tauri\Cargo.toml
   ```

3. **Development Mode**:
   ```powershell
   npx @tauri-apps/cli dev
   ```

4. **Production Build & Packaging (NSIS & MSI)**:
   ```powershell
   npx @tauri-apps/cli build
   ```
   Installers are generated in:
   - **NSIS Setup (`.exe`)**: `src-tauri\target\release\bundle\nsis\zentra_0.1.0_x64-setup.exe`
   - **WiX Installer (`.msi`)**: `src-tauri\target\release\bundle\msi\zentra_0.1.0_x64_en-US.msi`

5. **Signing Updates for Production**:
   Before building update artifacts, set your private signing key:
   ```powershell
   $env:TAURI_SIGNING_PRIVATE_KEY = "<path-or-content-of-private-key>"
   $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<optional-password>"
   npx @tauri-apps/cli build
   ```
   Tauri will produce the `.sig` signature files required for seamless in-app background auto-updates.

---

## Project Structure

```
Zentra/
├── src-tauri/                     # Rust Backend (Tauri v2)
│   ├── Cargo.toml                 # Dependencies (tauri 2, tauri-plugin-updater 2, rusqlite, windows 0.58)
│   ├── tauri.conf.json            # Desktop window config, bundle settings, and updater endpoints
│   ├── build.rs                   # Tauri build script
│   ├── capabilities/              # Security permissions (updater:default, core:default)
│   └── src/
│       ├── main.rs                # Windows entry point
│       ├── lib.rs                 # Tauri builder, plugins, command handlers, lifecycle
│       ├── clock.rs               # QPC clock frequency and delta calculations
│       ├── audio/                 # WASAPI loopback & microphone capture
│       ├── capture/               # Windows.Graphics.Capture pipeline & D3D11 frame pool
│       ├── commands/              # Thin Tauri command RPC handlers (recording, folders, sessions, hotkeys)
│       ├── encoder/               # Hardware Media Foundation H.264 rolling sink writer & segment_writer
│       ├── export/                # Compositor, decoder, audio remuxer, device setup, and keyframe timeline
│       ├── folders/               # Designated folder management domain service
│       ├── live_magnifier/        # Windows Magnification API live zoom & 60 FPS auto-pan
│       ├── mouse_hook/            # Low-level WH_MOUSE_LL hook thread & batch channel
│       ├── monitors/              # Multi-monitor enumeration and geometry
│       ├── storage/               # SQLite database manager (sessions, telemetry, hotkeys, settings)
│       └── tray/                  # System tray icon, notifications, and menu actions
├── ui/                            # Frontend Desktop Interface
│   ├── index.html                 # UI layout (timer hero, telemetry, sessions list, export modal)
│   ├── widget.html                # Draggable floating overlay layout
│   ├── style.css                  # Master stylesheet importing modular feature styles
│   ├── app.js                     # Main application coordinator
│   ├── widget.js                  # Overlay widget coordinator
│   ├── assets/                    # SVG icon sprites (icons.svg)
│   ├── css/                       # Modular feature stylesheets
│   └── js/                        # Modular feature controllers
│       ├── events.js              # Decoupled domain event bus
│       ├── updater.js             # Auto-Updater coordinator
│       ├── updater/               # Updater submodules (plugin-updater, ui, service)
│       ├── sessions.js            # Recorded sessions coordinator
│       ├── sessions/              # Sessions submodules (actions, render, service)
│       ├── export.js              # Video export coordinator
│       ├── export/                # Export submodules (ui, service)
│       ├── folders.js             # Designated folders coordinator
│       ├── folders/               # Folders submodules (service, view, dialogs, panels)
│       ├── recording.js           # Recording console coordinator
│       ├── recording/             # Recording submodules (monitors, options, telemetry)
│       ├── hotkeys.js             # Hotkeys coordinator
│       ├── hotkeys/               # Hotkeys submodules (dialog, formatter, recorder)
│       ├── widget/                # Overlay widget submodules (countdown, drag, sound)
│       ├── formatters.js          # Date, time, duration, and byte formatters
│       ├── icons.js               # SVG sprite injector
│       ├── ipc.js                 # Tauri IPC bridge with browser mock fallback
│       └── state.js               # Reactive global frontend state
├── package.json                   # Frontend dependencies (@tauri-apps/plugin-updater, @tauri-apps/api)
├── .gitignore                     # Git exclusion rules
└── README.md                      # Documentation
```

---

## Usage Guide

1. **Start Recording or Open Widget Mode**:
   - Click **Start Recording** to begin recording immediately (main app hides, widget appears recording).
   - Or click **Widget Mode** in the content grid: the main app hides and the floating widget appears in **Ready Mode** with a green **Record** button, allowing you to position and drag the widget on your screen *before* starting! Click **Record** on the widget when ready.
2. **Monitor & Drag Widget**: Glance at the floating widget to monitor live elapsed time (`00:00:00`). Drag the pill anywhere across your screen by clicking and moving it. *(The widget is hardware-excluded via `WDA_EXCLUDEFROMCAPTURE` and will never appear in your video).*
3. **Zoom In (<kbd>`</kbd>)**: Press the backtick key (<kbd>`</kbd>) at any time to zoom into 1.8x centered directly at your cursor.
4. **Drag / Pan Screen**: With zoom active, moving your mouse freely and smoothly drags the screen view at 60 FPS in 1:1 real-time lockstep across windows, code, or documents.
5. **Zoom Out (<kbd>Esc</kbd>)**: Press the <kbd>Esc</kbd> key to smoothly zoom back out to 1.0x full screen.
6. **Stop Recording**: Simply click the red **Stop** button on the floating widget (or press <kbd>F9</kbd> from any window). The widget disappears, and the main Zentra window automatically re-appears in focus with your new session ready!
7. **Export with Zoom**: Click **Export with Zoom** on any completed session. The video will be re-rendered at **exact 1.0x real-time speed** (CFR 60 FPS) with all live zoom-ins, auto-pans, screen drags, and zoom-outs faithfully reproduced. Once complete, click **Open Exported Video** to highlight the video in Windows Explorer.
8. **Check for Updates**: Click **Check for Updates** in the top navigation bar, or let Zentra check silently in the background on startup. When an update is ready, an unobtrusive banner lets you download and passively install the new version.
9. **Open Storage Directory**: Click **Open Folder** in the top navigation bar to open `%USERPROFILE%\Videos\Zentra`.

---

## License

Internal proprietary software developed for high-performance offline Windows screen recording.