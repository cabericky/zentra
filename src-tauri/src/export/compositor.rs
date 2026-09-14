use windows::core::Interface;
use windows::Foundation::Numerics::Matrix3x2;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::IDXGISurface;

use super::timeline::TransformState;

pub struct ZoomCompositor {
    factory: ID2D1Factory,
    width: u32,
    height: u32,
}

impl ZoomCompositor {
    pub fn new(_d3d_device: &ID3D11Device, width: u32, height: u32) -> windows::core::Result<Self> {
        let factory: ID2D1Factory = unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?
        };

        Ok(Self {
            factory,
            width,
            height,
        })
    }

    /// Renders the source frame texture into the destination output texture,
    /// applying the zoom scale and pan transform centered on the evaluated keyframe point.
    pub fn composite(
        &mut self,
        source_texture: &ID3D11Texture2D,
        dest_texture: &ID3D11Texture2D,
        transform: &TransformState,
    ) -> windows::core::Result<()> {
        unsafe {
            let src_surface: IDXGISurface = source_texture.cast()?;
            let dst_surface: IDXGISurface = dest_texture.cast()?;

            let rt_props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };

            let rt = self.factory.CreateDxgiSurfaceRenderTarget(&dst_surface, &rt_props)?;

            let bitmap_props = D2D1_BITMAP_PROPERTIES {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
            };

            let mut bitmap_raw: Option<ID2D1Bitmap> = None;
            rt.CreateSharedBitmap(
                &IDXGISurface::IID,
                src_surface.as_raw(),
                Some(&bitmap_props),
                &mut bitmap_raw,
            )?;

            let bitmap = match bitmap_raw {
                Some(b) => b,
                None => return Ok(()),
            };

            rt.BeginDraw();

            // Calculate 3x2 Affine Transformation Matrix
            // 1. Translate (center_x, center_y) to origin: -cx, -cy
            // 2. Scale by transform.scale
            // 3. Translate origin back to screen center: width / 2, height / 2
            let s = transform.scale;
            let cx = transform.center_x;
            let cy = transform.center_y;
            let half_w = (self.width as f32) / 2.0;
            let half_h = (self.height as f32) / 2.0;

            let matrix = Matrix3x2 {
                M11: s,
                M12: 0.0,
                M21: 0.0,
                M22: s,
                M31: half_w - s * cx,
                M32: half_h - s * cy,
            };

            rt.SetTransform(&matrix);

            // Draw source bitmap with high quality bilinear/bicubic filtering
            rt.DrawBitmap(
                &bitmap,
                None,
                1.0,
                D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                None,
            );

            rt.EndDraw(None, None)?;
        }

        Ok(())
    }
}