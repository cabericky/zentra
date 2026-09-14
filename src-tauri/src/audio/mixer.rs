use std::collections::VecDeque;
use super::types::{AUDIO_BYTES_PER_PACKET, AUDIO_FRAMES_PER_PACKET, AUDIO_SAMPLE_RATE};

pub struct AudioMixer {
    system_fifo: VecDeque<[f32; 2]>,
    mic_fifo: VecDeque<[f32; 2]>,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            system_fifo: VecDeque::with_capacity(9600),
            mic_fifo: VecDeque::with_capacity(9600),
        }
    }

    /// Appends incoming system audio samples, converting to 48 kHz stereo.
    pub fn push_system_audio(&mut self, samples: &[f32], channels: usize, sample_rate: usize) {
        Self::resample_and_enqueue(samples, channels, sample_rate, &mut self.system_fifo);
    }

    /// Appends incoming microphone audio samples, converting to 48 kHz stereo.
    pub fn push_mic_audio(&mut self, samples: &[f32], channels: usize, sample_rate: usize) {
        Self::resample_and_enqueue(samples, channels, sample_rate, &mut self.mic_fifo);
    }

    /// Drains up to `AUDIO_FRAMES_PER_PACKET` (960) frames, summing system and mic audio,
    /// and formats them into a 16-bit stereo PCM byte buffer.
    /// If fewer than 960 frames are buffered, pads with silence to maintain continuous 48 kHz output.
    pub fn extract_packet(&mut self) -> Vec<u8> {
        let mut pcm_bytes = Vec::with_capacity(AUDIO_BYTES_PER_PACKET);

        for _ in 0..AUDIO_FRAMES_PER_PACKET {
            let sys = self.system_fifo.pop_front().unwrap_or([0.0, 0.0]);
            let mic = self.mic_fifo.pop_front().unwrap_or([0.0, 0.0]);

            let mixed_l = (sys[0] + mic[0]).clamp(-1.0, 1.0);
            let mixed_r = (sys[1] + mic[1]).clamp(-1.0, 1.0);

            let pcm_l = (mixed_l * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
            let pcm_r = (mixed_r * 32767.0).round().clamp(-32768.0, 32767.0) as i16;

            pcm_bytes.extend_from_slice(&pcm_l.to_le_bytes());
            pcm_bytes.extend_from_slice(&pcm_r.to_le_bytes());
        }

        pcm_bytes
    }

    /// Converts raw interleaved input samples into 48 kHz stereo frames and pushes into the FIFO.
    fn resample_and_enqueue(
        samples: &[f32],
        channels: usize,
        sample_rate: usize,
        fifo: &mut VecDeque<[f32; 2]>,
    ) {
        if samples.is_empty() || channels == 0 || sample_rate == 0 {
            return;
        }

        // 1. Map input frames to stereo [L, R]
        let num_input_frames = samples.len() / channels;
        let mut stereo_frames: Vec<[f32; 2]> = Vec::with_capacity(num_input_frames);

        match channels {
            1 => {
                for &s in samples.iter().take(num_input_frames) {
                    stereo_frames.push([s, s]);
                }
            }
            2 => {
                for chunk in samples.chunks_exact(2) {
                    stereo_frames.push([chunk[0], chunk[1]]);
                }
            }
            _ => {
                for chunk in samples.chunks_exact(channels) {
                    let l = chunk[0];
                    let r = chunk[1];
                    let center = if channels > 2 { chunk[2] * 0.707 } else { 0.0 };
                    stereo_frames.push([(l + center).clamp(-1.0, 1.0), (r + center).clamp(-1.0, 1.0)]);
                }
            }
        }

        // 2. Resample to target 48000 Hz if needed
        let target_rate = AUDIO_SAMPLE_RATE as f64;
        let source_rate = sample_rate as f64;

        if (source_rate - target_rate).abs() < 1.0 {
            // Already 48 kHz, enqueue directly
            fifo.extend(stereo_frames);
        } else {
            // Linear interpolation resampling
            let ratio = source_rate / target_rate;
            let num_out_frames = ((stereo_frames.len() as f64) / ratio).floor() as usize;

            for i in 0..num_out_frames {
                let pos = i as f64 * ratio;
                let idx = pos.floor() as usize;
                let frac = (pos - idx as f64) as f32;

                let f0 = stereo_frames[idx];
                let f1 = if idx + 1 < stereo_frames.len() {
                    stereo_frames[idx + 1]
                } else {
                    f0
                };

                let l = f0[0] * (1.0 - frac) + f1[0] * frac;
                let r = f0[1] * (1.0 - frac) + f1[1] * frac;

                fifo.push_back([l, r]);
            }
        }

        // Prevent buffer bloat if consumer is delayed (keep max ~500ms in queue)
        let max_backlog = (AUDIO_SAMPLE_RATE as usize) / 2;
        while fifo.len() > max_backlog {
            fifo.pop_front();
        }
    }
}