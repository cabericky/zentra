use windows::core::Result;
use windows::Win32::Media::MediaFoundation::{
    IMFMediaType, IMFSinkWriter, MFCreateMediaType, MFCreateMemoryBuffer,
    MFCreateSample, MFAudioFormat_AAC, MFAudioFormat_PCM, MFMediaType_Audio,
    MF_MT_AUDIO_AVG_BYTES_PER_SECOND, MF_MT_AUDIO_BITS_PER_SAMPLE, MF_MT_AUDIO_BLOCK_ALIGNMENT,
    MF_MT_AUDIO_NUM_CHANNELS, MF_MT_AUDIO_SAMPLES_PER_SECOND, MF_MT_AVG_BITRATE,
    MF_MT_MAJOR_TYPE, MF_MT_SUBTYPE,
};

use super::types::{AUDIO_CHANNELS, AUDIO_SAMPLE_RATE};

/// Builds the Media Foundation AAC compressed audio output media type (48 kHz Stereo 192 kbps).
pub fn create_aac_output_type() -> Result<IMFMediaType> {
    unsafe {
        let audio_out_t = MFCreateMediaType()?;
        audio_out_t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
        audio_out_t.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_AAC)?;
        audio_out_t.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, AUDIO_CHANNELS)?;
        audio_out_t.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, AUDIO_SAMPLE_RATE)?;
        audio_out_t.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)?;
        audio_out_t.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, 24000)?;
        audio_out_t.SetUINT32(&MF_MT_AVG_BITRATE, 192000)?;
        Ok(audio_out_t)
    }
}

/// Builds the Media Foundation uncompressed 16-bit PCM stereo input media type.
pub fn create_pcm_input_type() -> Result<IMFMediaType> {
    unsafe {
        let audio_in_t = MFCreateMediaType()?;
        audio_in_t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
        audio_in_t.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_PCM)?;
        audio_in_t.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, AUDIO_CHANNELS)?;
        audio_in_t.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, AUDIO_SAMPLE_RATE)?;
        audio_in_t.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)?;
        audio_in_t.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, 4)?;
        audio_in_t.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, 192000)?;
        Ok(audio_in_t)
    }
}

/// Configures an AAC stream on the specified IMFSinkWriter and binds its PCM input type.
/// Returns the audio stream index assigned by the sink writer.
pub fn configure_audio_stream(writer: &IMFSinkWriter) -> Result<u32> {
    unsafe {
        let audio_out_t = create_aac_output_type()?;
        let audio_stream_idx = writer.AddStream(&audio_out_t)?;
        let audio_in_t = create_pcm_input_type()?;
        writer.SetInputMediaType(audio_stream_idx, &audio_in_t, None)?;
        Ok(audio_stream_idx)
    }
}

/// Wraps raw 16-bit PCM bytes into an IMFSample with presentation time and duration, then writes to the stream.
pub fn write_audio_sample(
    writer: &IMFSinkWriter,
    stream_idx: u32,
    pcm_data: &[u8],
    sample_time_hns: i64,
    duration_hns: i64,
) -> Result<()> {
    unsafe {
        let buffer = MFCreateMemoryBuffer(pcm_data.len() as u32)?;
        let mut ptr: *mut u8 = std::ptr::null_mut();
        buffer.Lock(&mut ptr, None, None)?;
        std::ptr::copy_nonoverlapping(pcm_data.as_ptr(), ptr, pcm_data.len());
        let _ = buffer.Unlock();
        buffer.SetCurrentLength(pcm_data.len() as u32)?;

        let sample = MFCreateSample()?;
        sample.AddBuffer(&buffer)?;
        sample.SetSampleTime(sample_time_hns)?;
        sample.SetSampleDuration(duration_hns)?;
        writer.WriteSample(stream_idx, &sample)
    }
}