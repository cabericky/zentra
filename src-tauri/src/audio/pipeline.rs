use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use crossbeam_channel::{bounded, Receiver, Sender};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

use crate::clock::QpcClock;
use super::device::WasapiCaptureDevice;
use super::mixer::AudioMixer;
use super::types::{AudioChunk, AUDIO_FRAMES_PER_PACKET, AUDIO_PACKET_DURATION_MS};

pub struct AudioPipeline {
    worker_handle: Option<JoinHandle<()>>,
    is_running: Arc<AtomicBool>,
}

impl AudioPipeline {
    /// Spawns the dedicated audio capture and mixing worker thread.
    /// Returns the pipeline handle and a bounded receiver for 48 kHz stereo AudioChunks.
    pub fn start(
        qpc_clock: QpcClock,
        qpc_session_start: i64,
        record_system_audio: bool,
        record_mic: bool,
    ) -> (Self, Receiver<AudioChunk>) {
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_clone = Arc::clone(&is_running);

        // Capacity of 100 packets = 2.0 seconds of audio buffer
        let (sender, receiver): (Sender<AudioChunk>, Receiver<AudioChunk>) = bounded(100);

        let worker_handle = thread::Builder::new()
            .name("zentra-audio-capture".to_string())
            .spawn(move || {
                unsafe {
                    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                }

                let loopback_dev = if record_system_audio {
                    let dev = WasapiCaptureDevice::new_loopback();
                    if dev.is_none() {
                        log::warn!("System audio loopback device could not be initialized.");
                    }
                    dev
                } else {
                    log::info!("System audio capture disabled by user.");
                    None
                };

                let mic_dev = if record_mic {
                    let dev = WasapiCaptureDevice::new_microphone();
                    if dev.is_none() {
                        log::info!("Microphone capture device could not be initialized or is absent.");
                    }
                    dev
                } else {
                    log::info!("Microphone audio capture disabled by user.");
                    None
                };

                let mut mixer = AudioMixer::new();
                let mut sys_buf = Vec::with_capacity(4096);
                let mut mic_buf = Vec::with_capacity(4096);

                let packet_duration_hns = (AUDIO_PACKET_DURATION_MS as i64) * 10_000;
                let mut next_packet_qpc = qpc_session_start;

                while is_running_clone.load(Ordering::Relaxed) {
                    // 1. Ingest available audio from system loopback
                    if let Some(ref dev) = loopback_dev {
                        sys_buf.clear();
                        dev.read_available(&mut sys_buf);
                        if !sys_buf.is_empty() {
                            mixer.push_system_audio(
                                &sys_buf,
                                dev.stream_info.channels,
                                dev.stream_info.sample_rate,
                            );
                        }
                    }

                    // 2. Ingest available audio from microphone
                    if let Some(ref dev) = mic_dev {
                        mic_buf.clear();
                        dev.read_available(&mut mic_buf);
                        if !mic_buf.is_empty() {
                            mixer.push_mic_audio(
                                &mic_buf,
                                dev.stream_info.channels,
                                dev.stream_info.sample_rate,
                            );
                        }
                    }

                    // 3. Dispatch audio packets aligned with QPC clock
                    let current_qpc = qpc_clock.now();
                    let elapsed_hns = qpc_clock.to_hns(current_qpc - next_packet_qpc);

                    if elapsed_hns >= packet_duration_hns {
                        let pcm_data = mixer.extract_packet();
                        let chunk = AudioChunk {
                            pcm_data,
                            qpc_timestamp: next_packet_qpc,
                            sample_count: AUDIO_FRAMES_PER_PACKET,
                        };

                        let _ = sender.try_send(chunk);

                        // Advance next_packet_qpc by exactly one packet duration in QPC ticks
                        let hns_advance = packet_duration_hns;
                        let qpc_advance = if qpc_clock.frequency > 0 {
                            (hns_advance as i128 * qpc_clock.frequency as i128 / 10_000_000i128) as i64
                        } else {
                            0
                        };
                        next_packet_qpc += qpc_advance;

                        // Prevent drift if worker was stalled
                        if current_qpc - next_packet_qpc > qpc_advance * 5 {
                            next_packet_qpc = current_qpc;
                        }
                    }

                    thread::sleep(Duration::from_millis(5));
                }

                // Clean up capture devices
                if let Some(ref dev) = loopback_dev {
                    dev.stop();
                }
                if let Some(ref dev) = mic_dev {
                    dev.stop();
                }

                unsafe {
                    CoUninitialize();
                }
                log::info!("Audio capture worker thread stopped.");
            })
            .expect("failed to spawn audio capture thread");

        (
            Self {
                worker_handle: Some(worker_handle),
                is_running,
            },
            receiver,
        )
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AudioPipeline {
    fn drop(&mut self) {
        self.stop();
    }
}