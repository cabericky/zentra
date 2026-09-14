#![allow(non_snake_case)]

use windows::core::HRESULT;

// COM Interop Interface for Windows.Graphics.Capture.GraphicsCaptureItem
#[windows::core::interface("3628e81b-3cac-4c60-b7f4-23ce0e0c3356")]
pub unsafe trait IGraphicsCaptureItemInterop: windows::core::IUnknown {
    pub unsafe fn CreateForWindow(
        &self,
        window: windows::Win32::Foundation::HWND,
        riid: *const windows::core::GUID,
        result: *mut *mut std::ffi::c_void,
    ) -> HRESULT;
    pub unsafe fn CreateForMonitor(
        &self,
        monitor: windows::Win32::Graphics::Gdi::HMONITOR,
        riid: *const windows::core::GUID,
        result: *mut *mut std::ffi::c_void,
    ) -> HRESULT;
}

// COM Interop Interface for IDirect3DDxgiInterfaceAccess
#[windows::core::interface("a9b3d012-3df2-4ee3-b8d1-8695f457d3c1")]
pub unsafe trait IDirect3DDxgiInterfaceAccess: windows::core::IUnknown {
    pub unsafe fn GetInterface(
        &self,
        riid: *const windows::core::GUID,
        p: *mut *mut std::ffi::c_void,
    ) -> HRESULT;
}

// Windows native function for wrapping DXGI Device into WinRT IDirect3DDevice
#[link(name = "d3d11")]
extern "system" {
    pub fn CreateDirect3D11DeviceFromDXGIDevice(
        dxgidevice: *mut std::ffi::c_void,
        graphicsdevice: *mut *mut std::ffi::c_void,
    ) -> HRESULT;
}