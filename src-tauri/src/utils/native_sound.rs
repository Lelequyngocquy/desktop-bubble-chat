use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
#[link(name = "winmm")]
unsafe extern "system" {
    fn PlaySoundW(psz_sound: *const u16, hmod: usize, fdw_sound: u32) -> i32;
}

#[cfg(target_os = "windows")]
const SND_ASYNC: u32 = 0x0001;
#[cfg(target_os = "windows")]
const SND_NODEFAULT: u32 = 0x0002;
#[cfg(target_os = "windows")]
const SND_MEMORY: u32 = 0x0004;

static LAST_PLAY_TIME: AtomicU64 = AtomicU64::new(0);
const THROTTLE_MS: u64 = 1500;

/// Tạo buffer file WAV 44.1kHz 16-bit Mono trong bộ nhớ tạo hiệu ứng âm thanh Bubble Pop
fn generate_bubble_pop_wav() -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let duration_secs: f32 = 0.085;
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let mut pcm_data = Vec::with_capacity(num_samples * 2);

    let mut phase = 0.0f32;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = t / duration_secs;
        // Frequency sweep from 420Hz to 860Hz
        let freq = 420.0 + (860.0 - 420.0) * progress;
        phase += 2.0 * std::f32::consts::PI * freq / sample_rate as f32;

        // Exponential decay envelope
        let decay = (1.0 - progress).powi(2);
        let sample = (phase.sin() * decay * 11000.0) as i16;

        pcm_data.extend_from_slice(&sample.to_le_bytes());
    }

    let data_len = pcm_data.len() as u32;
    let file_len = 36 + data_len;

    let mut wav = Vec::with_capacity(44 + pcm_data.len());
    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_len.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // "fmt " chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (16 for PCM)
    wav.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat (1 = PCM)
    wav.extend_from_slice(&1u16.to_le_bytes());  // NumChannels (1 = Mono)
    wav.extend_from_slice(&sample_rate.to_le_bytes()); // SampleRate
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // ByteRate
    wav.extend_from_slice(&2u16.to_le_bytes());  // BlockAlign
    wav.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample

    // "data" chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(&pcm_data);

    wav
}

static BUBBLE_WAV_CACHE: OnceLock<Vec<u8>> = OnceLock::new();

/// Phát âm thanh bong bóng nổi (Bubble Pop) thuần Native trong Rust Core
pub fn play_bubble_pop() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let last = LAST_PLAY_TIME.load(Ordering::Relaxed);
    if now.saturating_sub(last) < THROTTLE_MS {
        return;
    }
    LAST_PLAY_TIME.store(now, Ordering::Relaxed);

    #[cfg(target_os = "windows")]
    {
        let wav = BUBBLE_WAV_CACHE.get_or_init(generate_bubble_pop_wav);
        unsafe {
            let _ = PlaySoundW(
                wav.as_ptr() as *const u16,
                0,
                SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
            );
        }
    }
}
