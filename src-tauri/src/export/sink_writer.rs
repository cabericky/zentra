use std::path::Path;
use windows::core::{Interface, HSTRING};
use windows::Win32::Foundation::BOOL;
use windows::Win32::Graphics::Direct3D11::ID3D11Texture2D;
use windows::Win32::Media::MediaFoundation::*;

/// Configures and begins writing an MP4 video file using Media Foundation hardware transforms.
pub fn create_export_sink_writer(
    export_path: &Path,
    width: u32,
    height: u32,
    fps: u32,
    bitrate: u32,
    d3d_manager: Option<&IMFDXGIDeviceManager>,
) -> Result<(IMFSinkWriter, u32, Option<u32>), String> {
    let path_str = export_path.to_string_lossy().to_string();
    let path_hstring = HSTRING::from(&path_str);

    unsafe {
        let mut attributes: Option<IMFAttributes> = None;
        MFCreateAttributes(&mut attributes, 4)
            .map_err(|e| format!("Attributes creation failed: {:?}", e))?;
        let attrs = attributes.unwrap();
        let _ = attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1);
        let _ = attrs.SetUINT32(&MF_LOW_LATENCY, 1);
        if let Some(manager) = d3d_manager {
            let _ = attrs.SetUnknown(&MF_SINK_WRITER_D3D_MANAGER, manager);
        }

        let writer = MFCreateSinkWriterFromURL(&path_hstring, None::<&IMFByteStream>, &attrs)
            .map_err(|e| format!("Failed to create SinkWriter for {}: {:?}", path_str, e))?;

        // Output Media Type (H.264 MP4)
        let out_t = MFCreateMediaType()
            .map_err(|e| format!("MediaType creation failed: {:?}", e))?;
        out_t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
            .map_err(|e| format!("{:?}", e))?;
        out_t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)
            .map_err(|e| format!("{:?}", e))?;
        out_t.SetUINT32(&MF_MT_AVG_BITRATE, bitrate)
            .map_err(|e| format!("{:?}", e))?;
        out_t.SetUINT64(&MF_MT_FRAME_SIZE, ((width as u64) << 32) | (height as u64))
            .map_err(|e| format!("{:?}", e))?;
        out_t.SetUINT64(&MF_MT_FRAME_RATE, ((fps as u64) << 32) | 1)
            .map_err(|e| format!("{:?}", e))?;
        out_t.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1)
            .map_err(|e| format!("{:?}", e))?;
        out_t.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
            .map_err(|e| format!("{:?}", e))?;

        let video_stream_idx = writer.AddStream(&out_t)
            .map_err(|e| format!("AddStream video failed: {:?}", e))?;

        // Input Media Type (RGB32 / ARGB32 for D3D11 surface input)
        let in_t = MFCreateMediaType()
            .map_err(|e| format!("InType creation failed: {:?}", e))?;
        in_t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
            .map_err(|e| format!("{:?}", e))?;
        in_t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_ARGB32)
            .map_err(|e| format!("{:?}", e))?;
        in_t.SetUINT64(&MF_MT_FRAME_SIZE, ((width as u64) << 32) | (height as u64))
            .map_err(|e| format!("{:?}", e))?;
        in_t.SetUINT64(&MF_MT_FRAME_RATE, ((fps as u64) << 32) | 1)
            .map_err(|e| format!("{:?}", e))?;
        in_t.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1)
            .map_err(|e| format!("{:?}", e))?;
        in_t.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
            .map_err(|e| format!("{:?}", e))?;

        writer.SetInputMediaType(video_stream_idx, &in_t, None)
            .map_err(|e| format!("SetInputMediaType video failed: {:?}", e))?;

        // Setup Output Audio Media Type (AAC 48kHz Stereo 192kbps)
        let audio_stream_idx = crate::audio::mf::configure_audio_stream(&writer).ok();

        writer.BeginWriting()
            .map_err(|e| format!("BeginWriting failed: {:?}", e))?;

        Ok((writer, video_stream_idx, audio_stream_idx))
    }
}

/// Encapsulates a rendered D3D11 texture into a Media Foundation DXGI surface buffer sample and writes it.
pub fn write_video_frame(
    sink_writer: &IMFSinkWriter,
    stream_idx: u32,
    dst_texture: &ID3D11Texture2D,
    sample_time: i64,
    hns_frame_duration: i64,
    width: u32,
    height: u32,
) -> Result<(), String> {
    unsafe {
        let out_dxgi_buf = MFCreateDXGISurfaceBuffer(
            &ID3D11Texture2D::IID,
            dst_texture,
            0,
            BOOL(0),
        ).map_err(|e| format!("DXGISurfaceBuffer creation failed: {:?}", e))?;

        out_dxgi_buf.SetCurrentLength(width * height * 4)
            .map_err(|e| format!("SetCurrentLength failed: {:?}", e))?;

        let sample = MFCreateSample()
            .map_err(|e| format!("MFCreateSample failed: {:?}", e))?;
        sample.AddBuffer(&out_dxgi_buf)
            .map_err(|e| format!("AddBuffer failed: {:?}", e))?;
        sample.SetSampleTime(sample_time)
            .map_err(|e| format!("SetSampleTime failed: {:?}", e))?;
        sample.SetSampleDuration(hns_frame_duration)
            .map_err(|e| format!("SetSampleDuration failed: {:?}", e))?;

        sink_writer.WriteSample(stream_idx, &sample)
            .map_err(|e| format!("WriteSample failed: {:?}", e))?;

        Ok(())
    }
}