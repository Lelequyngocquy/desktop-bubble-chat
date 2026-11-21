pub struct LinuxSoundPlayer;

impl LinuxSoundPlayer {
    /// Plays static in-memory WAV buffer asynchronously via Linux audio backend
    pub fn play_wav(wav: &'static [u8]) {
        #[cfg(target_os = "linux")]
        {
            if wav.is_empty() {
                return;
            }
            use std::io::Write;
            let spawned = std::process::Command::new("paplay")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .or_else(|_| {
                    std::process::Command::new("aplay")
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn()
                });
            if let Ok(mut child) = spawned {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(wav);
                }
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
            }
        }
        #[cfg(not(target_os = "linux"))]
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
        LinuxSoundPlayer::play_wav(&[]);
        LinuxSoundPlayer::play_wav(&[0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00]);
    }
}
