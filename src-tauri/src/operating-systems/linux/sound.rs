pub struct LinuxSoundPlayer;

impl LinuxSoundPlayer {
    /// Ordered Linux audio player commands supporting PipeWire, PulseAudio, and ALSA
    pub const AUDIO_BACKENDS: [(&'static str, &'static [&'static str]); 3] = [
        ("pw-play", &["-"]),
        ("paplay", &[]),
        ("aplay", &["-q"]),
    ];

    /// Plays static in-memory WAV buffer asynchronously via Linux audio backend
    pub fn play_wav(wav: &'static [u8]) {
        #[cfg(target_os = "linux")]
        {
            if wav.is_empty() {
                return;
            }
            std::thread::spawn(move || {
                use std::io::Write;
                for (program, args) in Self::AUDIO_BACKENDS {
                    if let Ok(mut child) = std::process::Command::new(program)
                        .args(args)
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn()
                    {
                        if let Some(mut stdin) = child.stdin.take() {
                            let _ = stdin.write_all(wav);
                        }
                        let _ = child.wait();
                        break;
                    }
                }
            });
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
