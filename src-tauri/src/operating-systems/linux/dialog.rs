pub struct LinuxDialogManager;

impl LinuxDialogManager {
    /// Shows native Linux confirmation dialog asking user to confirm exit
    pub fn show_confirm_exit(title: &str, text: &str) -> bool {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("zenity")
                .args(&["--question", &format!("--title={}", title), &format!("--text={}", text)])
                .output()
            {
                return output.status.success();
            }
            true
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (title, text);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_exit_fallback() {
        #[cfg(not(target_os = "linux"))]
        assert!(LinuxDialogManager::show_confirm_exit("Exit", "Are you sure?"));
    }
}
