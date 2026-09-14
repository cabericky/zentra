/// Audio stream configuration constants matching the Media Foundation AAC encoder specifications.
pub const AUDIO_SAMPLE_RATE: u32 = 48000;
pub const AUDIO_CHANNELS: u32 = 2;
pub const AUDIO_BITS_PER_SAMPLE: u32 = 16;
pub const AUDIO_BLOCK_ALIGN: u32 = (AUDIO_CHANNELS * AUDIO_BITS_PER_SAMPLE) / 8; // 4 bytes
pub const AUDIO_BITRATE: u32 = 192000; // 192 kbps
pub const AUDIO_AVG_BYTES_PER_SEC: u32 = AUDIO_SAMPLE_RATE * AUDIO_BLOCK_ALIGN; // 192,000 B/s

/// Duration in milliseconds for each audio dispatch packet (20 ms = 960 frames @ 48 kHz).
pub const AUDIO_PACKET_DURATION_MS: u32 = 20;
pub const AUDIO_FRAMES_PER_PACKET: u32 = (AUDIO_SAMPLE_RATE * AUDIO_PACKET_DURATION_MS) / 1000; // 960 frames
pub const AUDIO_BYTES_PER_PACKET: usize = (AUDIO_FRAMES_PER_PACKET * AUDIO_BLOCK_ALIGN) as usize; // 3,840 bytes

/// Represents an uncompressed 16-bit stereo PCM chunk synchronized against QPC monotonic clock.
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// Raw 16-bit signed integer stereo PCM bytes (interleaved Left, Right, little-endian).
    pub pcm_data: Vec<u8>,
    /// QPC tick count at the start of this audio packet.
    pub qpc_timestamp: i64,
    /// Number of audio frames in this packet (e.g. 960).
    pub sample_count: u32,
}