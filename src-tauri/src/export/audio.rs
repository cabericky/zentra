use windows::Win32::Media::MediaFoundation::*;

/// Pumps decoded PCM audio samples from the segment SourceReader to the export SinkWriter,
/// adjusting timestamps to align with session timeline.
/// If `until_time_sec` is `Some(t)`, it only reads up to `t`. If `None`, it drains to EOF.
pub fn pump_segment_audio(
    reader: &IMFSourceReader,
    sink_writer: &IMFSinkWriter,
    audio_stream_idx: u32,
    seg_start_seconds: f64,
    until_time_sec: Option<f64>,
    last_audio_time_sec: &mut f64,
) {
    let seg_start_hns = (seg_start_seconds * 10_000_000.0) as i64;

    unsafe {
        loop {
            if let Some(target) = until_time_sec {
                if *last_audio_time_sec >= target {
                    break;
                }
            }

            let mut a_stream_idx = 0u32;
            let mut a_flags = 0u32;
            let mut a_timestamp = 0i64;
            let mut a_sample: Option<IMFSample> = None;

            let hr = reader.ReadSample(
                MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32,
                0,
                Some(&mut a_stream_idx),
                Some(&mut a_flags),
                Some(&mut a_timestamp),
                Some(&mut a_sample),
            );

            if hr.is_err() || (a_flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32) != 0 {
                break;
            }

            if let Some(audio_sample) = a_sample {
                let adjusted_hns = seg_start_hns + a_timestamp;
                let _ = audio_sample.SetSampleTime(adjusted_hns);
                let _ = sink_writer.WriteSample(audio_stream_idx, &audio_sample);
                *last_audio_time_sec = (a_timestamp as f64) / 10_000_000.0;
            } else {
                break;
            }
        }
    }
}