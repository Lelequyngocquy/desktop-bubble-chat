#[link(name = "winmm")]
unsafe extern "system" {
    fn PlaySoundW(psz_sound: *const u16, hmod: usize, fdw_sound: u32) -> i32;
}

const SND_ASYNC: u32 = 0x0001;
const SND_NODEFAULT: u32 = 0x0002;
const SND_MEMORY: u32 = 0x0004;

pub struct WindowsSoundPlayer;

impl WindowsSoundPlayer {
    /// Plays in-memory WAV buffer asynchronously via Win32 PlaySoundW
    pub fn play_wav(wav: &[u8]) {
        unsafe {
            let _ = PlaySoundW(wav.as_ptr() as *const u16, 0, SND_MEMORY | SND_ASYNC | SND_NODEFAULT);
        }
    }
}
