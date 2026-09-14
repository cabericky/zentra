# Zentra - Technical Architecture & Design Specification

This document details the underlying system architecture, hardware clock synchronization, mathematical reconciliation algorithms, and technical trade-offs implemented in Zentra.

---

## 1. System Architecture Pipeline

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

## 2. Clock Synchronization & Timestamp Reconciliation

A primary technical challenge in screen recording with cursor-driven post-effects is audio/video/mouse drift caused by wall-clock time (`SystemTime`) or variable-granularity tick counts.

Zentra guarantees sub-millisecond precision by synchronizing all telemetry against the Windows hardware `QueryPerformanceCounter` (QPC):

1. **Baseline Clock Initialization**:
   When recording begins, `QpcClock::now()` captures the baseline tick count $T_0$ and queries `QueryPerformanceFrequency()` ($F$, ticks per second).

2. **Frame Presentation Timestamping**:
   For every screen frame captured via `Windows.Graphics.Capture`, the presentation timestamp is calculated in 100-nanosecond Media Foundation units:
   $$\Delta t = \frac{T_{\text{frame}} - T_0}{F} \times 10{,}000{,}000$$

3. **Low-Level Telemetry Hook**:
   An independent OS thread running a Win32 message pump (`GetMessageW`) hosts the low-level mouse hook (`WH_MOUSE_LL`). It queries `QpcClock::now()` immediately upon `WM_LBUTTONDOWN`, `WM_RBUTTONDOWN`, and cursor movements, sending timestamped events across a Crossbeam batch channel to SQLite.

4. **Export Reconciliation & Viewport Clamping**:
   During post-recording rendering, each frame's exact timestamp $t$ (seconds from session start) is evaluated against the recorded timeline:
   $$t_{\text{rel}} = \frac{t - t_{\text{start}}}{D_{\text{ease}}}$$
   $$k = 3(t_{\text{rel}})^2 - 2(t_{\text{rel}})^3 \quad (\text{cubic ease-in-out})$$
   $$\text{scale} = 1.0 + k \times (\text{target\_scale} - 1.0)$$
   $$\text{viewport} = \text{clamp}\left(\text{center} - \frac{\text{dims}}{2 \times \text{scale}}, 0, \text{bounds}\right)$$

---

## 3. Technical Trade-off: Media Foundation vs. FFmpeg

| Evaluation Factor | Native Windows Media Foundation (Chosen) | Bundled FFmpeg CLI / libav |
| :--- | :--- | :--- |
| **Zero-Copy DXGI Pipeline** | **Direct**: D3D11 textures from WGC pass directly to encoder via `MFCreateDXGISurfaceBuffer`. GPU VRAM stays on GPU. | **Indirect**: Requires copying D3D11 textures to system RAM or configuring complex D3D11VA/NVENC hardware filters. |
| **Distribution Size** | **Zero bytes added**: Pre-installed on Windows 10/11. Final app installer is lean (~15 MB). | Adds **80 MB – 150 MB** of static binaries or dynamic libraries. |
| **Licensing** | Native Windows OS SDK. No GPL/LGPL complications. | LGPL/GPL licensing compliance and redistribution requirements. |
| **Compositing Integration** | Native Direct2D `ID2D1DeviceContext` interop with D3D11 render targets. | Requires pixel format conversions or piping raw frames over pipes/sockets. |
| **Portability** | Windows-only (aligns with project platform target). | Cross-platform (Linux/macOS/Windows). |
| **API Ergonomics** | Verbose COM interfaces requiring explicit attribute GUIDs and reference management. | High-level CLI or unified C API. |

---

## 4. Hardware Capture Exclusion

To prevent the floating overlay widget from appearing in recorded videos, Zentra applies the Win32 `SetWindowDisplayAffinity` API with `WDA_EXCLUDEFROMCAPTURE`:
- Windows Desktop Window Manager (DWM) omits the widget window from any capture session initiated by `Windows.Graphics.Capture` or BitBlt.
- The widget remains fully interactive and visible to the user on the physical display while remaining completely absent from the encoded video.