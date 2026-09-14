use std::path::{Path, PathBuf};
use windows::core::{Interface, HSTRING};
use windows::Win32::Foundation::BOOL;
use windows::Win32::Graphics::Direct3D11::ID3D11Texture2D;
use windows::Win32::Media::MediaFoundation::*;

use crate::audio::AudioChunk;

pub struct SegmentWriter {
    writer: IMFSinkWriter,
    video_stream_index: u32,
    audio_stream_index: u32,
    file_path: PathBuf,
}

impl SegmentWriter {
    /// Opens and initializes a new Media Foundation SinkWriter for a video segment.
    pub fn open(
        output_dir: &Path,
        seg_idx: u32,
        width: u32,
        height: u32,
        fps: u32,
        bitrate: u32,
        d3d_manager: Option<&IMFDXGIDeviceManager>,
    ) -> Option<Self> {
        let file_name = format!("segment_{:04}.mp4", seg_idx);
        let file_path = output_dir.join(&file_name);
        let path_str = file_path.to_string_lossy().to_string();
        let path_hstring = HSTRING::from(&path_str);

        unsafe {
            let mut attributes: Option<IMFAttributes> = None;
            if MFCreateAttributes(&mut attributes, 4).is_err() {
                return None;
            }
            let attrs = attributes.unwrap();
            let _ = attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1);
            let _ = attrs.SetUINT32(&MF_LOW_LATENCY, 1);
            if let Some(manager) = d3d_manager {
                let _ = attrs.SetUnknown(&MF_SINK_WRITER_D3D_MANAGER, manager);
            }

            let writer = match MFCreateSinkWriterFromURL(&path_hstring, None::<&IMFByteStream>, &attrs) {
                Ok(w) => w,
                Err(e) => {
                    log::error!("Failed to create SinkWriter for {}: {:?}", path_str, e);
                    return None;
                }
            };

            // Setup Output Video Media Type (H.264)
            let out_t = match MFCreateMediaType() {
                Ok(t) => t,
                Err(_) => return None,
            };
            out_t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video).ok()?;
            out_t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264).ok()?;
            out_t.SetUINT32(&MF_MT_AVG_BITRATE, bitrate).ok()?;
            out_t.SetUINT64(&MF_MT_FRAME_SIZE, ((width as u64) << 32) | (height as u64)).ok()?;
            out_t.SetUINT64(&MF_MT_FRAME_RATE, ((fps as u64) << 32) | 1).ok()?;
            out_t.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1).ok()?;
            out_t.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32).ok()?;

            let video_stream_index = match writer.AddStream(&out_t) {
                Ok(idx) => idx,
                Err(e) => {
                    log::error!("AddStream video failed: {:?}", e);
                    return None;
                }
            };

            // Setup Input Video Media Type (RGB32 from GPU surface)
            let in_t = match MFCreateMediaType() {
                Ok(t) => t,
                Err(_) => return None,
            };
            in_t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video).ok()?;
            in_t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_ARGB32).ok()?;
            in_t.SetUINT64(&MF_MT_FRAME_SIZE, ((width as u64) << 32) | (height as u64)).ok()?;
            in_t.SetUINT64(&MF_MT_FRAME_RATE, ((fps as u64) << 32) | 1).ok()?;
            in_t.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1).ok()?;
            in_t.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32).ok()?;

            writer.SetInputMediaType(video_stream_index, &in_t, None).ok()?;

            // Setup Audio Stream (AAC 48kHz Stereo 192kbps)
            let audio_stream_index = match crate::audio::mf::configure_audio_stream(&writer) {
                Ok(idx) => idx,
                Err(e) => {
                    log::error!("configure_audio_stream failed: {:?}", e);
                    return None;
                }
            };

            writer.BeginWriting().ok()?;

            log::info!("Started recording segment {} -> {}", seg_idx, path_str);
            Some(Self {
                writer,
                video_stream_index,
                audio_stream_index,
                file_path,
            })
        }
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Writes a single video surface texture to the segment.
    pub fn write_video_frame(
        &self,
        texture: &ID3D11Texture2D,
        sample_time_hns: i64,
        duration_hns: i64,
        width: u32,
        height: u32,
    ) -> bool {
        unsafe {
            match MFCreateDXGISurfaceBuffer(&ID3D11Texture2D::IID, texture, 0, BOOL(0)) {
                Ok(buffer) => {
                    let _ = buffer.SetCurrentLength(width * height * 4);
                    match MFCreateSample() {
                        Ok(sample) => {
                            let _ = sample.AddBuffer(&buffer);
                            let _ = sample.SetSampleTime(sample_time_hns);
                            let _ = sample.SetSampleDuration(duration_hns);
                            self.writer.WriteSample(self.video_stream_index, &sample).is_ok()
                        }
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            }
        }
    }

    /// Writes an audio chunk to the segment.
    pub fn write_audio_chunk(&self, chunk: &AudioChunk, sample_time_hns: i64, duration_hns: i64) {
        let _ = crate::audio::mf::write_audio_sample(
            &self.writer,
            self.audio_stream_index,
            &chunk.pcm_data,
            sample_time_hns,
            duration_hns,
        );
    }

    /// Finalizes the segment file and returns its path.
    pub fn finalize(self) -> PathBuf {
        unsafe {
            let _ = self.writer.Finalize();
        }
        self.file_path
    }
}