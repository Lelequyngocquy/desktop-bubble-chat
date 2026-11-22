pub struct LinuxDialogManager;

impl LinuxDialogManager {
    /// Shows native Linux confirmation dialog asking user to confirm exit
    pub fn show_confirm_exit(title: &str, text: &str) -> bool {
        #[cfg(target_os = "linux")]
        {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Warning)
                .set_title(title)
                .set_description(text)
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                == rfd::MessageDialogResult::Yes
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
