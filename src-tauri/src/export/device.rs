use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
    D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_FLAG, D3D11_CREATE_DEVICE_VIDEO_SUPPORT,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Media::MediaFoundation::{IMFDXGIDeviceManager, MFCreateDXGIDeviceManager};

/// Initializes the hardware Direct3D 11 device, immediate context, and DXGI Device Manager for export.
pub fn create_export_d3d_device() -> Result<(ID3D11Device, ID3D11DeviceContext, Option<IMFDXGIDeviceManager>), String> {
    let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT;
    let mut d3d_device: Option<ID3D11Device> = None;
    let mut d3d_context: Option<ID3D11DeviceContext> = None;
    let feature_levels = [D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0];

    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_FLAG(flags.0),
            Some(&feature_levels),
            windows::Win32::Graphics::Direct3D11::D3D11_SDK_VERSION,
            Some(&mut d3d_device),
            None,
            Some(&mut d3d_context),
        )
        .map_err(|e| format!("D3D11 device creation failed: {:?}", e))?;
    }

    let d3d_device = d3d_device.unwrap();
    let d3d_context = d3d_context.unwrap();

    let mut reset_token = 0u32;
    let mut d3d_manager: Option<IMFDXGIDeviceManager> = None;
    unsafe {
        if MFCreateDXGIDeviceManager(&mut reset_token, &mut d3d_manager).is_ok() {
            if let Some(ref manager) = d3d_manager {
                let _ = manager.ResetDevice(&d3d_device, reset_token);
            }
        }
    }

    Ok((d3d_device, d3d_context, d3d_manager))
}

/// Allocates source and destination textures for zoom compositing.
pub fn create_export_textures(
    d3d_device: &ID3D11Device,
    width: u32,
    height: u32,
) -> Result<(ID3D11Texture2D, ID3D11Texture2D), String> {
    let tex_desc = D3D11_TEXTURE2D_DESC {
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

    let mut src_texture = None;
    let mut dst_texture = None;
    unsafe {
        d3d_device
            .CreateTexture2D(&tex_desc, None, Some(&mut src_texture))
            .map_err(|e| format!("Source texture creation failed: {:?}", e))?;
        d3d_device
            .CreateTexture2D(&tex_desc, None, Some(&mut dst_texture))
            .map_err(|e| format!("Dest texture creation failed: {:?}", e))?;
    }

    Ok((src_texture.unwrap(), dst_texture.unwrap()))
}