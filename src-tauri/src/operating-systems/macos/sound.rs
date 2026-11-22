pub struct MacOSSoundPlayer;

#[cfg(target_os = "macos")]
static CACHED_SOUND_PTR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl MacOSSoundPlayer {
    /// Plays static in-memory WAV buffer asynchronously via retained AppKit NSSound
    pub fn play_wav(wav: &'static [u8]) {
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::{id, nil, BOOL};
            use objc::{class, msg_send, sel, sel_impl};
            use std::sync::atomic::Ordering;
            if wav.is_empty() {
                return;
            }
            super::bubble::window::ffi::run_on_macos_main_thread(move || unsafe {
                let mut sound = CACHED_SOUND_PTR.load(Ordering::Acquire) as id;
                if sound == nil {
                    let ns_data_class = class!(NSData);
                    let data: id = msg_send![ns_data_class, dataWithBytes: wav.as_ptr() length: wav.len()];
                    if data != nil {
                        let alloc_sound: id = msg_send![class!(NSSound), alloc];
                        sound = msg_send![alloc_sound, initWithData: data];
                        if sound != nil {
                            CACHED_SOUND_PTR.store(sound as usize, Ordering::Release);
                        }
                    }
                }
                if sound != nil {
                    let _: BOOL = msg_send![sound, stop];
                    let _: BOOL = msg_send![sound, play];
                }
            });
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = wav;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_player_safety() {
        MacOSSoundPlayer::play_wav(&[]);
        MacOSSoundPlayer::play_wav(&[0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00]);
    }
}
