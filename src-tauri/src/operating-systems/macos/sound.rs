pub struct MacOSSoundPlayer;

impl MacOSSoundPlayer {
    /// Plays in-memory WAV buffer asynchronously via AppKit NSSound
    pub fn play_wav(wav: &[u8]) {
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::{id, nil};
            use objc::{class, msg_send, sel, sel_impl};
            if wav.is_empty() { return; }
            unsafe {
                let ns_data_class = class!(NSData);
                let data: id = msg_send![ns_data_class, dataWithBytes:wav.as_ptr() length:wav.len()];
                if data != nil {
                    let sound_class = class!(NSSound);
                    let alloc_sound: id = msg_send![sound_class, alloc];
                    let sound: id = msg_send![alloc_sound, initWithData:data];
                    if sound != nil {
                        let _: bool = msg_send![sound, play];
                    }
                }
            }
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
