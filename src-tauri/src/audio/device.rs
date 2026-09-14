use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL};

const KSDATAFORMAT_SUBTYPE_PCM_GUID: windows::core::GUID =
    windows::core::GUID::from_u128(0x00000001_0000_0010_8000_00aa00389b71);
const KSDATAFORMAT_SUBTYPE_IEEE_FLOAT_GUID: windows::core::GUID =
    windows::core::GUID::from_u128(0x00000003_0000_0010_8000_00aa00389b71);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    Float32,
    Pcm16,
    Pcm24,
    Pcm32,
}

#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
    pub channels: usize,
    pub sample_rate: usize,
    pub bytes_per_sample: usize,
    pub format: SampleFormat,
}

impl AudioStreamInfo {
    /// Decodes a raw byte slice into normalized f32 samples in the range [-1.0, 1.0].
    pub fn decode_to_f32(&self, raw: &[u8], out: &mut Vec<f32>) {
        let bpf = self.bytes_per_sample;
        let num_samples = raw.len() / bpf;
        out.reserve(num_samples);

        match self.format {
            SampleFormat::Float32 => {
                for chunk in raw.chunks_exact(4) {
                    let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    out.push(sample);
                }
            }
            SampleFormat::Pcm16 => {
                for chunk in raw.chunks_exact(2) {
                    let val = i16::from_le_bytes([chunk[0], chunk[1]]);
                    out.push(val as f32 / 32768.0);
                }
            }
            SampleFormat::Pcm24 => {
                for chunk in raw.chunks_exact(3) {
                    let val = i32::from_le_bytes([0, chunk[0], chunk[1], chunk[2]]) >> 8;
                    out.push(val as f32 / 8388608.0);
                }
            }
            SampleFormat::Pcm32 => {
                for chunk in raw.chunks_exact(4) {
                    let val = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    out.push(val as f32 / 2147483648.0);
                }
            }
        }
    }
}

pub struct WasapiCaptureDevice {
    pub audio_client: IAudioClient,
    pub capture_client: IAudioCaptureClient,
    pub stream_info: AudioStreamInfo,
    pub is_loopback: bool,
}

impl WasapiCaptureDevice {
    /// Initializes Windows WASAPI loopback capture on the default audio rendering endpoint.
    pub fn new_loopback() -> Option<Self> {
        Self::init_device(eRender, eConsole, AUDCLNT_STREAMFLAGS_LOOPBACK, true)
    }

    /// Initializes Windows WASAPI standard capture mode on the default microphone endpoint.
    pub fn new_microphone() -> Option<Self> {
        // First try communications role, then fallback to console role
        Self::init_device(eCapture, eCommunications, 0, false)
            .or_else(|| Self::init_device(eCapture, eConsole, 0, false))
    }

    fn init_device(
        flow: EDataFlow,
        role: ERole,
        stream_flags: u32,
        is_loopback: bool,
    ) -> Option<Self> {
        unsafe {
            let enumerator: IMMDeviceEnumerator = match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                Ok(e) => e,
                Err(err) => {
                    log::warn!("CoCreateInstance MMDeviceEnumerator failed: {:?}", err);
                    return None;
                }
            };

            let device: IMMDevice = match enumerator.GetDefaultAudioEndpoint(flow, role) {
                Ok(d) => d,
                Err(err) => {
                    log::info!(
                        "No default audio endpoint for flow {:?} (role {:?}): {:?}",
                        flow,
                        role,
                        err
                    );
                    return None;
                }
            };

            let audio_client: IAudioClient = match device.Activate(CLSCTX_ALL, None) {
                Ok(c) => c,
                Err(err) => {
                    log::warn!("Failed to activate IAudioClient: {:?}", err);
                    return None;
                }
            };

            let mix_format_ptr = match audio_client.GetMixFormat() {
                Ok(fmt) => fmt,
                Err(err) => {
                    log::warn!("GetMixFormat failed: {:?}", err);
                    return None;
                }
            };

            if mix_format_ptr.is_null() {
                return None;
            }

            let stream_info = match parse_waveformat(mix_format_ptr) {
                Some(info) => info,
                None => {
                    CoTaskMemFree(Some(mix_format_ptr as *const _));
                    return None;
                }
            };

            // 10,000,000 hns = 1 second buffer duration
            let buffer_duration_hns = 10_000_000i64;
            let init_hr = audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                stream_flags,
                buffer_duration_hns,
                0,
                mix_format_ptr,
                None,
            );

            CoTaskMemFree(Some(mix_format_ptr as *const _));

            if let Err(err) = init_hr {
                log::warn!(
                    "audio_client.Initialize failed (loopback: {}): {:?}",
                    is_loopback,
                    err
                );
                return None;
            }

            let capture_client: IAudioCaptureClient = match audio_client.GetService() {
                Ok(cc) => cc,
                Err(err) => {
                    log::warn!("GetService::<IAudioCaptureClient> failed: {:?}", err);
                    return None;
                }
            };

            if let Err(err) = audio_client.Start() {
                log::warn!("audio_client.Start failed: {:?}", err);
                return None;
            }

            log::info!(
                "Initialized WASAPI device (loopback: {}, rate: {} Hz, channels: {}, format: {:?})",
                is_loopback,
                stream_info.sample_rate,
                stream_info.channels,
                stream_info.format,
            );

            Some(Self {
                audio_client,
                capture_client,
                stream_info,
                is_loopback,
            })
        }
    }

    /// Reads all currently available audio frames from the capture client into normalized f32 samples.
    pub fn read_available(&self, buffer: &mut Vec<f32>) {
        unsafe {
            loop {
                let packet_size = match self.capture_client.GetNextPacketSize() {
                    Ok(s) => s,
                    Err(_) => break,
                };
                if packet_size == 0 {
                    break;
                }

                let mut data_ptr: *mut u8 = std::ptr::null_mut();
                let mut frames_read = 0u32;
                let mut flags = 0u32;

                let hr = self.capture_client.GetBuffer(
                    &mut data_ptr,
                    &mut frames_read,
                    &mut flags,
                    None,
                    None,
                );

                if hr.is_err() || frames_read == 0 || data_ptr.is_null() {
                    break;
                }

                let is_silent = (flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32) != 0;
                let total_samples = frames_read as usize * self.stream_info.channels;

                if is_silent {
                    buffer.resize(buffer.len() + total_samples, 0.0f32);
                } else {
                    let bytes = frames_read as usize * self.stream_info.channels * self.stream_info.bytes_per_sample;
                    let slice = std::slice::from_raw_parts(data_ptr, bytes);
                    self.stream_info.decode_to_f32(slice, buffer);
                }

                let _ = self.capture_client.ReleaseBuffer(frames_read);
            }
        }
    }

    pub fn stop(&self) {
        unsafe {
            let _ = self.audio_client.Stop();
        }
    }
}

unsafe fn parse_waveformat(ptr: *const WAVEFORMATEX) -> Option<AudioStreamInfo> {
    let wfx = &*ptr;
    let channels = wfx.nChannels as usize;
    let sample_rate = wfx.nSamplesPerSec as usize;
    let bits_per_sample = wfx.wBitsPerSample as usize;
    let tag = wfx.wFormatTag as u32;

    if channels == 0 || sample_rate == 0 {
        return None;
    }

    let (format, bytes_per_sample) = if tag == 1 {
        // WAVE_FORMAT_PCM
        match bits_per_sample {
            16 => (SampleFormat::Pcm16, 2),
            24 => (SampleFormat::Pcm24, 3),
            32 => (SampleFormat::Pcm32, 4),
            _ => return None,
        }
    } else if tag == 3 {
        // WAVE_FORMAT_IEEE_FLOAT
        (SampleFormat::Float32, 4)
    } else if tag == 0xFFFE {
        // WAVE_FORMAT_EXTENSIBLE
        let ext = &*(ptr as *const WAVEFORMATEXTENSIBLE);
        let sub_format = std::ptr::addr_of!(ext.SubFormat).read_unaligned();
        if sub_format == KSDATAFORMAT_SUBTYPE_IEEE_FLOAT_GUID {
            (SampleFormat::Float32, 4)
        } else if sub_format == KSDATAFORMAT_SUBTYPE_PCM_GUID {
            match bits_per_sample {
                16 => (SampleFormat::Pcm16, 2),
                24 => (SampleFormat::Pcm24, 3),
                32 => (SampleFormat::Pcm32, 4),
                _ => return None,
            }
        } else {
            log::warn!("Unsupported WAVEFORMATEXTENSIBLE subformat: {:?}", sub_format);
            return None;
        }
    } else {
        log::warn!("Unsupported WAVEFORMATEX tag: {}", tag);
        return None;
    };

    Some(AudioStreamInfo {
        channels,
        sample_rate,
        bytes_per_sample,
        format,
    })
}