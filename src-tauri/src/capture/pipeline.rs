#![allow(non_snake_case)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crossbeam_channel::Sender;
use windows::core::{Interface, HSTRING};
use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
    D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_CREATE_DEVICE_FLAG, D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_DEFAULT,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Gdi::{MonitorFromPoint, MONITOR_DEFAULTTOPRIMARY};
use windows::Win32::System::Performance::QueryPerformanceCounter;
use windows::Win32::System::WinRT::RoGetActivationFactory;

use super::interop::{CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess, IGraphicsCaptureItemInterop};
use crate::platform::is_windows_build_at_least;

/// Represents a captured video frame on the GPU.
pub struct CapturedFrame {
    pub texture: ID3D11Texture2D,
    pub qpc_timestamp: i64,
    pub frame_index: u64,
}

pub struct CapturePipeline {
    pub d3d11_device: ID3D11Device,
    pub d3d11_context: ID3D11DeviceContext,
    pub width: u32,
    pub height: u32,
    session: GraphicsCaptureSession,
    frame_pool: Direct3D11CaptureFramePool,
    is_capturing: Arc<AtomicBool>,
}

impl CapturePipeline {
    /// Creates and starts the screen capture pipeline for the specified monitor (or primary if None).
    pub fn new(
        monitor: Option<windows::Win32::Graphics::Gdi::HMONITOR>,
        frame_sender: Sender<CapturedFrame>,
        is_capturing: Arc<AtomicBool>,
    ) -> windows::core::Result<Self> {
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
            let _ = windows::Win32::System::WinRT::RoInitialize(
                windows::Win32::System::WinRT::RO_INIT_MULTITHREADED,
            );
        }

        // 1. Create D3D11 Device and Context with BGRA and Video support
        let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT;
        let mut d3d11_device = None;
        let mut d3d11_context = None;
        let feature_levels = [D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0];

        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_FLAG(flags.0),
                Some(&feature_levels),
                windows::Win32::Graphics::Direct3D11::D3D11_SDK_VERSION,
                Some(&mut d3d11_device),
                None,
                Some(&mut d3d11_context),
            )?;
        }

        let d3d11_device = d3d11_device.unwrap();
        let d3d11_context = d3d11_context.unwrap();

        // 2. Wrap D3D11 Device into WinRT IDirect3DDevice
        let dxgi_device: IDXGIDevice = d3d11_device.cast()?;
        let winrt_device = unsafe {
            let mut inspectable_raw: *mut std::ffi::c_void = std::ptr::null_mut();
            let hr = CreateDirect3D11DeviceFromDXGIDevice(
                dxgi_device.as_raw(),
                &mut inspectable_raw,
            );
            if hr.is_err() {
                return Err(windows::core::Error::from(hr));
            }
            IDirect3DDevice::from_raw(inspectable_raw)
        };

        // 3. Obtain monitor GraphicsCaptureItem via Interop
        let hmonitor = monitor.unwrap_or_else(|| unsafe {
            MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY)
        });

        let interop_factory: IGraphicsCaptureItemInterop = {
            let class_name = HSTRING::from("Windows.Graphics.Capture.GraphicsCaptureItem");
            unsafe { RoGetActivationFactory(&class_name)? }
        };

        let capture_item: GraphicsCaptureItem = unsafe {
            let mut item_raw: *mut std::ffi::c_void = std::ptr::null_mut();
            let hr = interop_factory.CreateForMonitor(
                hmonitor,
                &GraphicsCaptureItem::IID,
                &mut item_raw,
            );
            if hr.is_err() {
                return Err(windows::core::Error::from(hr));
            }
            GraphicsCaptureItem::from_raw(item_raw)
        };

        let size = capture_item.Size()?;
        let width = size.Width as u32;
        let height = size.Height as u32;

        log::info!("Screen capture initialized for monitor: {}x{}", width, height);

        // 4. Create free-threaded capture frame pool
        let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &winrt_device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            2,
            size,
        )?;

        // 5. Create capture session
        let session = frame_pool.CreateCaptureSession(&capture_item)?;
        let _ = session.SetIsCursorCaptureEnabled(true);

        // Suppress the bright yellow perimeter border drawn by Windows DWM (Windows 10 build 20348+ / Windows 11)
        if is_windows_build_at_least(20348) {
            match session.SetIsBorderRequired(false) {
                Ok(()) => log::info!("Capture yellow border disabled (SetIsBorderRequired: false)"),
                Err(e) => log::warn!("Failed to disable capture yellow border: {:?}", e),
            }
        }

        // 6. Pre-allocate staging textures for double/triple buffering
        let texture_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };

        let num_buffers = 4;
        let mut buffer_pool: Vec<ID3D11Texture2D> = Vec::with_capacity(num_buffers);
        for _ in 0..num_buffers {
            let mut tex = None;
            unsafe {
                d3d11_device.CreateTexture2D(&texture_desc, None, Some(&mut tex))?;
            }
            buffer_pool.push(tex.unwrap());
        }

        let context_clone = d3d11_context.clone();
        let is_capturing_clone = Arc::clone(&is_capturing);
        let mut buffer_idx = 0usize;
        let mut frame_count = 0u64;

        // 7. Subscribe to FrameArrived event
        frame_pool.FrameArrived(&windows::Foundation::TypedEventHandler::new(
            move |pool: &Option<Direct3D11CaptureFramePool>, _| {
                if !is_capturing_clone.load(Ordering::Relaxed) {
                    return Ok(());
                }

                if let Some(pool) = pool {
                    if let Ok(frame) = pool.TryGetNextFrame() {
                        let mut qpc_now = 0i64;
                        unsafe {
                            let _ = QueryPerformanceCounter(&mut qpc_now);
                        }

                        let surface = frame.Surface()?;
                        let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;
                        let mut src_tex_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                        unsafe {
                            let hr = access.GetInterface(&ID3D11Texture2D::IID, &mut src_tex_ptr);
                            if hr.is_ok() {
                                let src_tex = ID3D11Texture2D::from_raw(src_tex_ptr);

                                let target_tex = &buffer_pool[buffer_idx % buffer_pool.len()];
                                buffer_idx += 1;

                                context_clone.CopyResource(target_tex, &src_tex);

                                frame_count += 1;
                                let captured = CapturedFrame {
                                    texture: target_tex.clone(),
                                    qpc_timestamp: qpc_now,
                                    frame_index: frame_count,
                                };

                                if let Err(_) = frame_sender.try_send(captured) {
                                    log::warn!("Encoder queue full, dropped frame {}", frame_count);
                                }
                            }
                        }
                    }
                }
                Ok(())
            },
        ))?;

        session.StartCapture()?;

        Ok(Self {
            d3d11_device,
            d3d11_context,
            width,
            height,
            session,
            frame_pool,
            is_capturing,
        })
    }

    pub fn stop(&mut self) {
        self.is_capturing.store(false, Ordering::SeqCst);
        let _ = self.session.Close();
        let _ = self.frame_pool.Close();
    }
}

impl Drop for CapturePipeline {
    fn drop(&mut self) {
        self.stop();
    }
}