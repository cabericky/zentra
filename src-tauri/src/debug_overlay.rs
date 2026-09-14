use windows::core::Interface;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_POINT_2F,
};
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::IDXGISurface;

use crate::clock::QpcClock;
use crate::mouse_hook::ClickEvent;

pub struct DebugOverlay {
    factory: ID2D1Factory,
}

impl DebugOverlay {
    pub fn new(_d3d_device: &ID3D11Device, _width: u32, _height: u32) -> windows::core::Result<Self> {
        let factory: ID2D1Factory = unsafe {
            D2D1CreateFactory(
                D2D1_FACTORY_TYPE_SINGLE_THREADED,
                None,
            )?
        };

        Ok(Self { factory })
    }

    /// Renders a red dot overlay on the given frame texture if any click occurred within
    /// a 250ms window of the frame's QPC timestamp.
    pub fn render(
        &mut self,
        texture: &ID3D11Texture2D,
        frame_qpc: i64,
        clicks: &[ClickEvent],
        clock: &QpcClock,
    ) {
        if clicks.is_empty() {
            return;
        }

        let window_seconds = 0.25;
        let active_clicks: Vec<&ClickEvent> = clicks
            .iter()
            .filter(|c| {
                let delta = clock.delta_seconds(c.qpc_timestamp, frame_qpc).abs();
                delta <= window_seconds
            })
            .collect();

        if active_clicks.is_empty() {
            return;
        }

        unsafe {
            let dxgi_surface: IDXGISurface = match texture.cast() {
                Ok(s) => s,
                Err(_) => return,
            };

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

            let rt = match self.factory.CreateDxgiSurfaceRenderTarget(&dxgi_surface, &rt_props) {
                Ok(t) => t,
                Err(_) => return,
            };

            let red_color = D2D1_COLOR_F { r: 1.0, g: 0.1, b: 0.1, a: 0.85 };
            let white_color = D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.95 };

            let red_brush = match rt.CreateSolidColorBrush(&red_color, None) {
                Ok(b) => b,
                Err(_) => return,
            };
            let white_brush = match rt.CreateSolidColorBrush(&white_color, None) {
                Ok(b) => b,
                Err(_) => return,
            };

            rt.BeginDraw();

            for click in active_clicks {
                let delta = clock.delta_seconds(click.qpc_timestamp, frame_qpc).abs();
                let progress = (1.0 - (delta / window_seconds)).max(0.0) as f32;
                let radius = 16.0 + 8.0 * progress;

                let ellipse = D2D1_ELLIPSE {
                    point: D2D_POINT_2F {
                        x: click.x as f32,
                        y: click.y as f32,
                    },
                    radiusX: radius,
                    radiusY: radius,
                };

                // Fill red circle
                rt.FillEllipse(&ellipse, &red_brush);
                // Draw white outer stroke
                rt.DrawEllipse(&ellipse, &white_brush, 3.0, None);

                // Center highlight
                let center_ellipse = D2D1_ELLIPSE {
                    point: ellipse.point,
                    radiusX: 4.0,
                    radiusY: 4.0,
                };
                rt.FillEllipse(&center_ellipse, &white_brush);
            }

            let _ = rt.EndDraw(None, None);
        }
    }
}