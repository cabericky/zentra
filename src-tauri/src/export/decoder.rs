use std::path::Path;
use windows::core::HSTRING;
use windows::Win32::Media::MediaFoundation::*;

/// Opens and configures a Media Foundation Source Reader for a recorded segment.
/// Returns the configured reader and whether stereo PCM audio is available.
pub fn open_segment_reader(
    seg_path: &Path,
    enable_audio: bool,
) -> Result<(IMFSourceReader, bool), String> {
    let seg_hstring = HSTRING::from(seg_path.to_string_lossy().as_ref());

    unsafe {
        let mut reader_attrs: Option<IMFAttributes> = None;
        MFCreateAttributes(&mut reader_attrs, 3)
            .map_err(|e| format!("Reader attributes failed: {:?}", e))?;
        let r_attrs = reader_attrs.unwrap();
        let _ = r_attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1);
        let _ = r_attrs.SetUINT32(&MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, 1);

        let reader = MFCreateSourceReaderFromURL(&seg_hstring, &r_attrs)
            .map_err(|e| format!("Failed to open segment source reader {}: {:?}", seg_path.display(), e))?;

        // Request uncompressed RGB32 frames from reader
        if let Ok(rgb_t) = MFCreateMediaType() {
            let _ = rgb_t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video);
            let _ = rgb_t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32);
            let _ = reader.SetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32, None, &rgb_t);
        }

        // Request uncompressed 16-bit PCM stereo audio if export sink writer has an audio stream
        let has_audio = if enable_audio {
            if let Ok(pcm_t) = MFCreateMediaType() {
                let _ = pcm_t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio);
                let _ = pcm_t.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_PCM);
                let _ = pcm_t.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, 2);
                let _ = pcm_t.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, 48000);
                let _ = pcm_t.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16);
                let _ = pcm_t.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, 4);
                let _ = pcm_t.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, 192000);
                reader.SetCurrentMediaType(MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32, None, &pcm_t).is_ok()
            } else {
                false
            }
        } else {
            false
        };

        Ok((reader, has_audio))
    }
}