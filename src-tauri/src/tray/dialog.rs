/// Displays confirmation dialog before quitting to prevent accidental exits
pub fn confirm_exit_dialog(locale: &str) -> bool {
    let tray_i18n = crate::utils::i18n::get_tray_locale(locale);
    crate::operating_systems::current::DialogManager::show_confirm_exit(&tray_i18n.confirm_exit_title, &tray_i18n.confirm_exit_text)
}
